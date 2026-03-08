# Status Monitoring

**Capability:** Real-time VM status with color indicators

---

## ADDED Requirements

### Requirement: Status Polling

The app SHALL poll VM status every 5 seconds using `gcloud compute instances describe`.

#### Scenario: Poll a running VM

- **WHEN** a VM has been created and is being monitored
- **THEN** the app SHALL invoke `gcloud compute instances describe {vm-name} --zone={zone} --project={project} --format=json` every 5 seconds and update the displayed status

#### Scenario: VM not found during polling

- **WHEN** the gcloud describe command returns a "not found" error
- **THEN** the app SHALL set the VM status to not-found and stop polling for that VM

#### Scenario: Polling stops after VM deletion

- **WHEN** the user deletes a VM through the app
- **THEN** the app SHALL stop the polling loop for that VM

#### Scenario: Transient gcloud error during polling

- **WHEN** a single poll fails due to a transient error (e.g., network timeout) but the VM was previously known
- **THEN** the app SHALL retain the last known status and retry on the next polling interval

---

### Requirement: Color Indicators

The app SHALL display VM status as colored dots: green for RUNNING, yellow with pulse animation for transitional states (PROVISIONING, STAGING, STOPPING, SUSPENDING), and red for terminal states (TERMINATED, not found, STOPPED).

#### Scenario: Running VM indicator

- **WHEN** the polled VM status is RUNNING
- **THEN** the app SHALL display a green dot next to the VM/disk entry

#### Scenario: Provisioning VM indicator

- **WHEN** the polled VM status is PROVISIONING
- **THEN** the app SHALL display a yellow dot with a pulse animation next to the VM/disk entry

#### Scenario: Staging VM indicator

- **WHEN** the polled VM status is STAGING
- **THEN** the app SHALL display a yellow dot with a pulse animation

#### Scenario: Terminated VM indicator

- **WHEN** the polled VM status is TERMINATED
- **THEN** the app SHALL display a red dot next to the VM/disk entry

#### Scenario: Stopped VM indicator

- **WHEN** the polled VM status is STOPPED
- **THEN** the app SHALL display a red dot next to the VM/disk entry

#### Scenario: VM not found indicator

- **WHEN** the VM is not found (deleted externally)
- **THEN** the app SHALL display a red dot and indicate the VM no longer exists

---

### Requirement: Resource Summary

When a VM is running, the app SHALL display the CPU count, memory amount, GPU type and count, and provisioning model.

#### Scenario: Display resource summary for a running VM

- **WHEN** the polled VM status is RUNNING
- **THEN** the app SHALL display the machine type's vCPU count, memory in GB, GPU type and count (if applicable), and whether the VM is spot or on-demand

#### Scenario: Resource summary not shown for non-running VM

- **WHEN** the polled VM status is TERMINATED or STOPPED
- **THEN** the app SHALL NOT display the resource summary section

---

### Requirement: Auto-Discovery

On app launch, the app SHALL check all disks for attached VMs and start monitoring any running instances.

#### Scenario: Discover running VMs on launch

- **WHEN** the app starts and the disk list contains disks attached to VMs
- **THEN** the app SHALL immediately begin polling the status of each attached VM

#### Scenario: No running VMs on launch

- **WHEN** the app starts and no disks are attached to VMs
- **THEN** the app SHALL not start any polling loops and display all disks as unattached

#### Scenario: Multiple running VMs on launch

- **WHEN** the app starts and three disks are each attached to a separate VM
- **THEN** the app SHALL start independent polling loops for all three VMs

---

### Requirement: Event-Driven Updates

The backend SHALL emit Tauri events (`vm-status-update`) for real-time frontend updates.

#### Scenario: Emit status update event

- **WHEN** the backend completes a poll and the VM status has changed
- **THEN** the backend SHALL emit a `vm-status-update` Tauri event containing the VM name, new status, and resource metadata

#### Scenario: Frontend receives status event

- **WHEN** the frontend receives a `vm-status-update` event
- **THEN** the frontend SHALL update the corresponding disk/VM entry's color indicator and resource summary without a full page refresh

#### Scenario: No event emitted when status unchanged

- **WHEN** the backend completes a poll and the VM status has not changed since the last poll
- **THEN** the backend SHALL NOT emit a `vm-status-update` event
