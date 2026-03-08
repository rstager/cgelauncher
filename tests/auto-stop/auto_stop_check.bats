#!/usr/bin/env bats
# BATS tests for auto-stop-check.sh
# Tests idle SSH detection, timestamp management, disable file, and config parsing.
# Shutdown-triggering tests are excluded (tested via manual VM validation).

setup() {
    TEST_DIR="$(mktemp -d)"
    export TEST_DIR
    export DISABLE_FILE="${TEST_DIR}/auto-stop-agent.disabled"
    export CONFIG_FILE="${TEST_DIR}/auto-stop.conf"
    export STATE_DIR="${TEST_DIR}/run"
    export LAST_ACTIVE_FILE="${STATE_DIR}/last-active"

    MOCK_BIN="${TEST_DIR}/bin"
    mkdir -p "$MOCK_BIN" "$STATE_DIR"
    export PATH="${MOCK_BIN}:${PATH}"

    # Default: mock ss with no connections
    cat > "${MOCK_BIN}/ss" << 'EOF'
#!/bin/sh
echo "State  Recv-Q Send-Q Local Address:Port Peer Address:Port"
EOF
    chmod +x "${MOCK_BIN}/ss"

    # Mock shutdown/wall/logger as no-ops (never expected to be called)
    for cmd in shutdown wall logger; do
        cat > "${MOCK_BIN}/${cmd}" << 'EOF'
#!/bin/sh
exit 0
EOF
        chmod +x "${MOCK_BIN}/${cmd}"
    done

    export CHECK_SCRIPT="${TEST_DIR}/auto-stop-check.sh"
    create_check_script
}

teardown() {
    rm -rf "$TEST_DIR"
}

create_check_script() {
    cat > "$CHECK_SCRIPT" << 'SCRIPT'
#!/bin/sh
set -e

DISABLE_FILE="${DISABLE_FILE:-/etc/auto-stop-agent.disabled}"
CONFIG_FILE="${CONFIG_FILE:-/etc/default/auto-stop}"
STATE_DIR="${STATE_DIR:-/var/run/auto-stop-agent}"
LAST_ACTIVE_FILE="${LAST_ACTIVE_FILE:-${STATE_DIR}/last-active}"

AUTO_STOP_TIMEOUT=1800
AUTO_STOP_GRACE=1

if [ -f "$CONFIG_FILE" ]; then
    . "$CONFIG_FILE"
fi

if [ -f "$DISABLE_FILE" ]; then
    exit 0
fi

mkdir -p "$STATE_DIR"

NOW=$(date +%s)

has_ssh_sessions() {
    if command -v ss >/dev/null 2>&1; then
        ss -tnp 2>/dev/null | grep -q ':22[[:space:]]'
    elif command -v netstat >/dev/null 2>&1; then
        netstat -tnp 2>/dev/null | grep -q ':22[[:space:]]'
    else
        grep -q ':0016 ' /proc/net/tcp 2>/dev/null
    fi
}

if has_ssh_sessions; then
    echo "$NOW" > "$LAST_ACTIVE_FILE"
    exit 0
fi

if [ ! -f "$LAST_ACTIVE_FILE" ]; then
    echo "$NOW" > "$LAST_ACTIVE_FILE"
    exit 0
fi

LAST_ACTIVE=$(cat "$LAST_ACTIVE_FILE" 2>/dev/null || echo "$NOW")
IDLE_SECONDS=$((NOW - LAST_ACTIVE))

if [ "$IDLE_SECONDS" -ge "$AUTO_STOP_TIMEOUT" ]; then
    logger -t auto-stop-agent "No SSH sessions for ${IDLE_SECONDS}s (timeout: ${AUTO_STOP_TIMEOUT}s). Shutting down."
    wall "auto-stop-agent: VM idle for ${IDLE_SECONDS}s with no SSH sessions. Shutting down in ${AUTO_STOP_GRACE} minute(s)."
    shutdown +"$AUTO_STOP_GRACE"
fi
SCRIPT
    chmod +x "$CHECK_SCRIPT"
}

mock_ssh_active() {
    cat > "${MOCK_BIN}/ss" << 'EOF'
#!/bin/sh
echo "ESTAB  0      0      10.0.0.1:22    192.168.1.1:54321  users:(("sshd",pid=1234,fd=3))"
EOF
    chmod +x "${MOCK_BIN}/ss"
}

mock_ssh_inactive() {
    cat > "${MOCK_BIN}/ss" << 'EOF'
#!/bin/sh
echo "State  Recv-Q Send-Q Local Address:Port Peer Address:Port"
EOF
    chmod +x "${MOCK_BIN}/ss"
}

set_last_active_ago() {
    local seconds_ago=$1
    local now
    now=$(date +%s)
    local past=$((now - seconds_ago))
    echo "$past" > "$LAST_ACTIVE_FILE"
}

# --- TC-AS-001: Active SSH prevents shutdown ---

@test "TC-AS-001: active SSH session updates timestamp and does not shutdown" {
    mock_ssh_active

    run sh "$CHECK_SCRIPT"
    [ "$status" -eq 0 ]
    [ -f "$LAST_ACTIVE_FILE" ]
}

# --- TC-AS-003: Idle within timeout does not shutdown ---

@test "TC-AS-003: idle within timeout does not trigger shutdown" {
    mock_ssh_inactive
    set_last_active_ago 600  # 10 minutes, timeout is 1800s

    run sh "$CHECK_SCRIPT"
    [ "$status" -eq 0 ]
}

# --- TC-AS-004: Disable file prevents shutdown ---

@test "TC-AS-004: disable file prevents shutdown even when idle" {
    mock_ssh_inactive
    set_last_active_ago 3600
    touch "$DISABLE_FILE"

    run sh "$CHECK_SCRIPT"
    [ "$status" -eq 0 ]
}

# --- TC-AS-006: Missing last-active file with SSH active ---

@test "TC-AS-006: first run with active SSH creates timestamp" {
    mock_ssh_active
    rm -f "$LAST_ACTIVE_FILE"

    run sh "$CHECK_SCRIPT"
    [ "$status" -eq 0 ]
    [ -f "$LAST_ACTIVE_FILE" ]
}

# --- TC-AS-007: Missing last-active file with no SSH ---

@test "TC-AS-007: first run with no SSH creates timestamp without shutdown" {
    mock_ssh_inactive
    rm -f "$LAST_ACTIVE_FILE"

    run sh "$CHECK_SCRIPT"
    [ "$status" -eq 0 ]
    [ -f "$LAST_ACTIVE_FILE" ]
}

# --- TC-AS-012: Config defaults when no file ---

@test "TC-AS-012: defaults used when config file is missing" {
    mock_ssh_inactive
    rm -f "$CONFIG_FILE"
    set_last_active_ago 1740  # 29 min, default timeout is 1800s

    run sh "$CHECK_SCRIPT"
    [ "$status" -eq 0 ]
}

# --- Active SSH after idle resets timestamp ---

@test "active SSH after idle resets timestamp" {
    mock_ssh_inactive
    set_last_active_ago 900

    run sh "$CHECK_SCRIPT"
    [ "$status" -eq 0 ]

    mock_ssh_active
    run sh "$CHECK_SCRIPT"
    [ "$status" -eq 0 ]

    local now
    now=$(date +%s)
    local last
    last=$(cat "$LAST_ACTIVE_FILE")
    local diff=$((now - last))
    [ "$diff" -le 5 ]
}

# --- TC-AS-005b: Custom timeout prevents premature shutdown ---

@test "TC-AS-005b: custom timeout prevents premature shutdown" {
    mock_ssh_inactive
    echo "AUTO_STOP_TIMEOUT=600" > "$CONFIG_FILE"
    set_last_active_ago 300  # 5 min idle, timeout is 600s

    run sh "$CHECK_SCRIPT"
    [ "$status" -eq 0 ]
}
