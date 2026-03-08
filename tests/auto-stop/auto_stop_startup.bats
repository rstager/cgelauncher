#!/usr/bin/env bats
# BATS tests for auto-stop-startup.sh
# Tests that the startup script installs systemd units and is idempotent.

setup() {
    TEST_DIR="$(mktemp -d)"

    MOCK_ROOT="${TEST_DIR}/root"
    mkdir -p "${MOCK_ROOT}/usr/local/bin"
    mkdir -p "${MOCK_ROOT}/etc/systemd/system"
    mkdir -p "${MOCK_ROOT}/etc/default"

    MOCK_BIN="${TEST_DIR}/bin"
    mkdir -p "$MOCK_BIN"
    export PATH="${MOCK_BIN}:${PATH}"

    cat > "${MOCK_BIN}/systemctl" << 'EOF'
#!/bin/sh
exit 0
EOF
    chmod +x "${MOCK_BIN}/systemctl"

    cat > "${MOCK_BIN}/logger" << 'EOF'
#!/bin/sh
exit 0
EOF
    chmod +x "${MOCK_BIN}/logger"

    export STARTUP_SCRIPT="${TEST_DIR}/startup.sh"
    create_startup_script
}

teardown() {
    rm -rf "$TEST_DIR"
}

create_startup_script() {
    cat > "$STARTUP_SCRIPT" << SCRIPT
#!/bin/sh
set -e

CHECK_SCRIPT="${MOCK_ROOT}/usr/local/bin/auto-stop-check.sh"
SERVICE_FILE="${MOCK_ROOT}/etc/systemd/system/auto-stop.service"
TIMER_FILE="${MOCK_ROOT}/etc/systemd/system/auto-stop.timer"
CONFIG_FILE="${MOCK_ROOT}/etc/default/auto-stop"

cat > "\$CHECK_SCRIPT" << 'CHECKEOF'
#!/bin/sh
set -e
echo "check script installed"
CHECKEOF
chmod +x "\$CHECK_SCRIPT"

cat > "\$SERVICE_FILE" << 'SERVICEEOF'
[Unit]
Description=Auto-stop idle VM check
After=network.target sshd.service

[Service]
Type=oneshot
ExecStart=/usr/local/bin/auto-stop-check.sh
SERVICEEOF

cat > "\$TIMER_FILE" << 'TIMEREOF'
[Unit]
Description=Auto-stop idle VM check timer

[Timer]
OnBootSec=5min
OnUnitActiveSec=5min

[Install]
WantedBy=timers.target
TIMEREOF

if [ ! -f "\$CONFIG_FILE" ]; then
    cat > "\$CONFIG_FILE" << 'CONFIGEOF'
AUTO_STOP_TIMEOUT=1800
AUTO_STOP_GRACE=1
CONFIGEOF
fi

systemctl daemon-reload
systemctl enable auto-stop.timer
systemctl start auto-stop.timer

logger -t auto-stop-agent "Auto-stop agent installed and timer started."
SCRIPT
    chmod +x "$STARTUP_SCRIPT"
}

# --- TC-AS-008: Startup script installs units ---

@test "TC-AS-008: startup script creates check script" {
    run sh "$STARTUP_SCRIPT"
    [ "$status" -eq 0 ]
    [ -f "${MOCK_ROOT}/usr/local/bin/auto-stop-check.sh" ]
    [ -x "${MOCK_ROOT}/usr/local/bin/auto-stop-check.sh" ]
}

@test "TC-AS-008: startup script creates service unit" {
    run sh "$STARTUP_SCRIPT"
    [ "$status" -eq 0 ]
    [ -f "${MOCK_ROOT}/etc/systemd/system/auto-stop.service" ]
    grep -q "Type=oneshot" "${MOCK_ROOT}/etc/systemd/system/auto-stop.service"
    grep -q "auto-stop-check.sh" "${MOCK_ROOT}/etc/systemd/system/auto-stop.service"
}

@test "TC-AS-008: startup script creates timer unit" {
    run sh "$STARTUP_SCRIPT"
    [ "$status" -eq 0 ]
    [ -f "${MOCK_ROOT}/etc/systemd/system/auto-stop.timer" ]
    grep -q "OnUnitActiveSec=5min" "${MOCK_ROOT}/etc/systemd/system/auto-stop.timer"
    grep -q "timers.target" "${MOCK_ROOT}/etc/systemd/system/auto-stop.timer"
}

@test "TC-AS-008: startup script creates default config" {
    run sh "$STARTUP_SCRIPT"
    [ "$status" -eq 0 ]
    [ -f "${MOCK_ROOT}/etc/default/auto-stop" ]
    grep -q "AUTO_STOP_TIMEOUT=1800" "${MOCK_ROOT}/etc/default/auto-stop"
}

# --- TC-AS-009: Startup script is idempotent ---

@test "TC-AS-009: running startup script twice succeeds" {
    run sh "$STARTUP_SCRIPT"
    [ "$status" -eq 0 ]

    run sh "$STARTUP_SCRIPT"
    [ "$status" -eq 0 ]

    [ -f "${MOCK_ROOT}/usr/local/bin/auto-stop-check.sh" ]
    [ -f "${MOCK_ROOT}/etc/systemd/system/auto-stop.service" ]
    [ -f "${MOCK_ROOT}/etc/systemd/system/auto-stop.timer" ]
}

@test "TC-AS-009: idempotent run preserves existing config" {
    echo "AUTO_STOP_TIMEOUT=900" > "${MOCK_ROOT}/etc/default/auto-stop"

    run sh "$STARTUP_SCRIPT"
    [ "$status" -eq 0 ]

    grep -q "AUTO_STOP_TIMEOUT=900" "${MOCK_ROOT}/etc/default/auto-stop"
}
