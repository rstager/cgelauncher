# Auto-Stop Agent for GCE VMs

**Capability:** Auto-shutdown idle VM

---

## Intent

Reduce cloud costs by automatically shutting down a VM when no SSH sessions are active for a configurable idle period (default: 30 minutes).

---

## Requirements

### Requirement: Idle Detection
- The agent SHALL monitor for active SSH sessions (e.g., by checking for `sshd` processes or open port 22 connections).
- The agent SHALL consider the VM idle if no SSH sessions are detected for the configured idle timeout.

### Requirement: Configurable Timeout
- The idle timeout SHALL be configurable via an environment variable or config file (default: 30 minutes).

### Requirement: Safe Shutdown
- When the idle timeout is reached, the agent SHALL initiate a clean system shutdown (`shutdown -h now`).
- The agent SHALL log a message before shutdown and allow a short grace period (e.g., 1 minute) for new SSH connections to appear.

### Requirement: Systemd Integration
- The agent SHALL be installable as a systemd service and timer, running as root or a privileged user.
- The agent SHALL be robust to VM reboots and restarts.

### Requirement: Manual Override
- The agent SHALL provide a way to disable auto-shutdown (e.g., by setting an environment variable or placing a file at `/etc/auto-stop-agent.disabled`).

---

## Scenarios

### Scenario: Idle VM is auto-stopped
- **GIVEN** a running VM with the agent installed
- **WHEN** no SSH sessions are active for 30 minutes
- **THEN** the agent SHALL log a shutdown message and power off the VM

### Scenario: Active SSH session prevents shutdown
- **GIVEN** a running VM with the agent installed
- **WHEN** at least one SSH session is active
- **THEN** the agent SHALL NOT shut down the VM

### Scenario: Timeout is configurable
- **GIVEN** the agent is running with `AUTO_STOP_TIMEOUT=10m`
- **WHEN** no SSH sessions are active for 10 minutes
- **THEN** the agent SHALL shut down the VM

### Scenario: Manual override disables agent
- **GIVEN** the file `/etc/auto-stop-agent.disabled` exists
- **WHEN** the agent runs
- **THEN** the agent SHALL NOT shut down the VM, regardless of idle time

---

## Out of Scope
- Detecting non-SSH activity (e.g., Jupyter, HTTP)
- Notifying users before shutdown (beyond log message)
- Cross-VM coordination
