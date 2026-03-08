# SSH Config

**Capability:** SSH config update after VM start

---

## ADDED Requirements

### Requirement: Auto-Update

After VM creation succeeds, the app SHALL run `gcloud compute config-ssh` asynchronously to update `~/.ssh/config`.

#### Scenario: SSH config updated after VM creation

- **WHEN** the `gcloud compute instances create` command completes successfully
- **THEN** the app SHALL invoke `gcloud compute config-ssh --project={project} --quiet` asynchronously without blocking the UI

#### Scenario: SSH config update failure

- **WHEN** the `gcloud compute config-ssh` command fails
- **THEN** the app SHALL display a warning notification indicating SSH config could not be updated, but SHALL NOT treat it as a VM creation failure

#### Scenario: SSH config not triggered on creation failure

- **WHEN** the VM creation command fails
- **THEN** the app SHALL NOT invoke `gcloud compute config-ssh`

---

### Requirement: Connection Display

The app SHALL display the SSH connection string in a status bar (e.g., `ssh {vm-name}.{zone}.{project}`).

#### Scenario: Display SSH connection string for a running VM

- **WHEN** a VM named `ml-workspace-vm` is running in zone `us-central1-a` for project `my-project`
- **THEN** the app SHALL display `ssh ml-workspace-vm.us-central1-a.my-project` in the status bar

#### Scenario: Connection string not shown for non-running VM

- **WHEN** the VM status is TERMINATED or STOPPED
- **THEN** the app SHALL NOT display the SSH connection string

#### Scenario: Copy connection string

- **WHEN** the user clicks on the displayed SSH connection string
- **THEN** the app SHALL copy the connection string to the system clipboard

---

### Requirement: Notification

The app SHALL show a notification when SSH config is ready.

#### Scenario: SSH config ready notification

- **WHEN** the `gcloud compute config-ssh` command completes successfully
- **THEN** the app SHALL display a success notification indicating SSH config has been updated and the VM is accessible via SSH

#### Scenario: Notification timing

- **WHEN** the SSH config update completes while the user is viewing the app
- **THEN** the notification SHALL appear within 1 second of command completion and auto-dismiss after 5 seconds
