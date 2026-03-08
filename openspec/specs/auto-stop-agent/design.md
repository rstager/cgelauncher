# Auto-Stop Agent - Design Document

schema: rks-sdd
status: draft
created: 2026-03-07

---

## Context

CGE Launcher creates ephemeral VMs from persistent disks. VMs are meant to be short-lived, but users often leave them running after disconnecting SSH. This agent runs on the VM itself to detect idle SSH sessions and auto-shutdown, complementing the desktop app's "delete VM" workflow.

The agent is a POSIX shell script deployed via GCE startup script metadata. It installs a systemd service and timer on first boot. The timer fires a check script every 5 minutes that tracks SSH session state and triggers shutdown after sustained inactivity.

## Goals / Non-Goals

### Goals

- Zero-install idle detection: deployed automatically when VM is created via the launcher.
- Configurable idle timeout via environment variable (default: 30 minutes).
- Manual override to disable auto-shutdown without modifying the service.
- Clean shutdown (not hard power-off) to allow graceful process termination.
- Grace period before shutdown to allow reconnection.

### Non-Goals

- Detecting non-SSH activity (Jupyter kernels, running jobs, HTTP traffic).
- Notifying the launcher app before shutdown (the status monitor will detect the VM going away).
- Supporting VMs not created by the launcher.
- User-facing configuration UI in the launcher (future enhancement).

## Decisions

### 1. Shell script over compiled binary

The agent is a POSIX shell script, not a compiled binary. This avoids cross-compilation concerns, works on any Linux VM image, and is simple to embed as startup script metadata. The logic (check SSH sessions, compare timestamps) is trivial for shell.

### 2. Startup script metadata injection

GCE `--metadata startup-script=<script>` runs on every boot. The script installs the systemd unit files on first run (idempotent) and is the standard GCE mechanism — no SSH provisioning step needed.

### 3. Systemd timer over cron

Systemd timers provide better logging (journalctl), dependency management, and are the standard on modern Linux distributions. The timer fires every 5 minutes; the service unit runs the idle check.

### 4. SSH session detection via `ss`

Use `ss -tnp | grep ':22'` to detect active SSH connections. This is more reliable than checking `who` or `w` (which miss port-forwarded sessions) and more portable than parsing `/proc` directly.

### 5. Idle tracking via timestamp file

A file `/var/run/auto-stop-agent/last-active` stores the epoch timestamp of the last observed SSH session. Each check updates this timestamp if sessions exist. If no sessions exist and `(now - last_active) >= timeout`, shutdown is triggered. This survives timer restarts and avoids in-memory state.

### 6. Grace period with wall message

Before shutdown, the agent writes a `wall` message and schedules `shutdown +1` (1-minute delay). This gives users a window to reconnect and cancel (`shutdown -c` or touch the disable file).

### 7. Disable file convention

Placing `/etc/auto-stop-agent.disabled` (any content) disables the agent. This is discoverable, simple, and can be toggled by users via SSH without modifying systemd units.

## Architecture Overview

```
+---------------------------+
|  GCE VM                   |
|                           |
|  /usr/local/bin/           |
|    auto-stop-check.sh     |  <-- idle detection script
|                           |
|  /etc/systemd/system/     |
|    auto-stop.service      |  <-- oneshot unit (runs check)
|    auto-stop.timer        |  <-- fires every 5 minutes
|                           |
|  /etc/default/auto-stop   |  <-- config: AUTO_STOP_TIMEOUT=1800
|  /var/run/auto-stop-agent/|
|    last-active            |  <-- timestamp file
|                           |
|  /etc/auto-stop-agent.disabled  <-- touch to disable
+---------------------------+

Launcher (Tauri app)
  |
  | gcloud compute instances create ... \
  |   --metadata startup-script="$(cat auto-stop-startup.sh)"
  v
GCE API
  |
  | VM boots -> startup script runs
  | -> installs systemd units
  | -> timer starts
  v
Every 5 min: auto-stop.timer -> auto-stop.service -> auto-stop-check.sh
  |
  | if SSH sessions active: update /var/run/auto-stop-agent/last-active
  | if idle >= timeout: wall + shutdown +1
```

### Launcher integration

The `create_instance` function in `gcloud/instance.rs` adds `--metadata startup-script=<embedded-script>` to the gcloud command. The script content is embedded as a Rust `const` string (or loaded from a bundled resource file).

## Data Models Reference

No new data models in the launcher app. The agent uses only filesystem artifacts:
- `/etc/default/auto-stop` — shell-sourceable config file
- `/var/run/auto-stop-agent/last-active` — epoch timestamp

See `datamodels.yaml` for the `AutoStopConfig` schema (documents the config file format).

## API Contract Reference

No new Tauri commands. The only API change is to `startVm` — the gcloud command gains the metadata flag. See `apis.yaml` for the updated `StartVmRequest`.

## Risks / Trade-offs

| # | Risk | Impact | Mitigation |
|---|------|--------|------------|
| 1 | User has long-running job with no SSH | VM shutdown kills the job | Document in README; user can `touch /etc/auto-stop-agent.disabled` |
| 2 | Startup script fails silently | Agent not installed, VM never auto-stops | Script logs to syslog; idempotent so next boot retries |
| 3 | `ss` not available on minimal images | Detection fails | Fall back to `netstat` or `/proc/net/tcp`; script checks for `ss` availability |
| 4 | User reconnects during grace period | Shutdown proceeds if within 1-min window | Grace period is intentionally short; new SSH session on next timer cycle would cancel future shutdowns |
| 5 | Metadata size limit (256 KB) | Script too large | Shell script is <2 KB; well within limits |
