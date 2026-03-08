#!/bin/sh
# auto-stop-check.sh — Idle SSH session detector for GCE VMs.
# Shuts down the VM if no SSH sessions are active for the configured timeout.
#
# Installed by the auto-stop startup script. Runs as a systemd oneshot service
# triggered by auto-stop.timer every 5 minutes.
#
# Override: touch /etc/auto-stop-agent.disabled to prevent shutdown.
# Config:   /etc/default/auto-stop (AUTO_STOP_TIMEOUT=seconds)

set -e

DISABLE_FILE="/etc/auto-stop-agent.disabled"
CONFIG_FILE="/etc/default/auto-stop"
STATE_DIR="/var/run/auto-stop-agent"
LAST_ACTIVE_FILE="${STATE_DIR}/last-active"

# Defaults
AUTO_STOP_TIMEOUT=1800
AUTO_STOP_GRACE=1

# Load config overrides
if [ -f "$CONFIG_FILE" ]; then
    . "$CONFIG_FILE"
fi

# Check disable file
if [ -f "$DISABLE_FILE" ]; then
    exit 0
fi

# Ensure state directory exists
mkdir -p "$STATE_DIR"

NOW=$(date +%s)

# Detect active SSH sessions on port 22
has_ssh_sessions() {
    if command -v ss >/dev/null 2>&1; then
        ss -tnp 2>/dev/null | grep -q ':22[[:space:]]'
    elif command -v netstat >/dev/null 2>&1; then
        netstat -tnp 2>/dev/null | grep -q ':22[[:space:]]'
    else
        # Fallback: check /proc/net/tcp for port 22 (0x0016)
        grep -q ':0016 ' /proc/net/tcp 2>/dev/null
    fi
}

if has_ssh_sessions; then
    # Active session — update timestamp
    echo "$NOW" > "$LAST_ACTIVE_FILE"
    exit 0
fi

# No SSH sessions — check idle duration
if [ ! -f "$LAST_ACTIVE_FILE" ]; then
    # First run with no sessions — initialize timestamp, don't shutdown
    echo "$NOW" > "$LAST_ACTIVE_FILE"
    exit 0
fi

LAST_ACTIVE=$(cat "$LAST_ACTIVE_FILE" 2>/dev/null || echo "$NOW")
IDLE_SECONDS=$((NOW - LAST_ACTIVE))

if [ "$IDLE_SECONDS" -ge "$AUTO_STOP_TIMEOUT" ]; then
    logger -t auto-stop-agent "No SSH sessions for ${IDLE_SECONDS}s (timeout: ${AUTO_STOP_TIMEOUT}s). Shutting down."
    wall "auto-stop-agent: VM idle for ${IDLE_SECONDS}s with no SSH sessions. Shutting down in ${AUTO_STOP_GRACE} minute(s). Reconnect or run 'sudo shutdown -c' to cancel."
    shutdown +"$AUTO_STOP_GRACE"
fi
