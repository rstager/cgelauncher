#!/bin/sh
# auto-stop-startup.sh — GCE startup script that installs the auto-stop agent.
# Injected via --metadata startup-script=... during VM creation.
# Idempotent: safe to run on every boot.

set -e

CHECK_SCRIPT="/usr/local/bin/auto-stop-check.sh"
SERVICE_FILE="/etc/systemd/system/auto-stop.service"
TIMER_FILE="/etc/systemd/system/auto-stop.timer"
CONFIG_FILE="/etc/default/auto-stop"

# Write the idle detection check script
cat > "$CHECK_SCRIPT" << 'CHECKEOF'
#!/bin/sh
set -e

DISABLE_FILE="/etc/auto-stop-agent.disabled"
CONFIG_FILE="/etc/default/auto-stop"
STATE_DIR="/var/run/auto-stop-agent"
LAST_ACTIVE_FILE="${STATE_DIR}/last-active"

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
    wall "auto-stop-agent: VM idle for ${IDLE_SECONDS}s with no SSH sessions. Shutting down in ${AUTO_STOP_GRACE} minute(s). Reconnect or run 'sudo shutdown -c' to cancel."
    shutdown +"$AUTO_STOP_GRACE"
fi
CHECKEOF
chmod +x "$CHECK_SCRIPT"

# Write systemd service unit
cat > "$SERVICE_FILE" << 'SERVICEEOF'
[Unit]
Description=Auto-stop idle VM check
After=network.target sshd.service

[Service]
Type=oneshot
ExecStart=/usr/local/bin/auto-stop-check.sh
SERVICEEOF

# Write systemd timer unit
cat > "$TIMER_FILE" << 'TIMEREOF'
[Unit]
Description=Auto-stop idle VM check timer

[Timer]
OnBootSec=5min
OnUnitActiveSec=5min

[Install]
WantedBy=timers.target
TIMEREOF

# Write default config if not present
if [ ! -f "$CONFIG_FILE" ]; then
    cat > "$CONFIG_FILE" << 'CONFIGEOF'
# Auto-stop agent configuration
# AUTO_STOP_TIMEOUT: idle seconds before shutdown (default: 1800 = 30 minutes)
# AUTO_STOP_GRACE: minutes of grace period before shutdown (default: 1)
AUTO_STOP_TIMEOUT=1800
AUTO_STOP_GRACE=1
CONFIGEOF
fi

# Enable and start the timer
systemctl daemon-reload
systemctl enable auto-stop.timer
systemctl start auto-stop.timer

logger -t auto-stop-agent "Auto-stop agent installed and timer started."
