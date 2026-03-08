# Disk Management

**Capability:** List and select persistent disks in a project/zone

---

## ADDED Requirements

### Requirement: Disk Listing

The app SHALL list all persistent disks in the configured GCP project and zone by invoking `gcloud compute disks list` via the gcloud CLI. Each disk entry SHALL display the disk name, size in GB, disk type (e.g., pd-standard, pd-ssd, pd-balanced), and whether the disk is currently attached to a VM instance.

#### Scenario: List disks in configured project and zone

- **WHEN** the user opens the disk management view and a valid project and zone are configured
- **THEN** the app SHALL query `gcloud compute disks list` filtered to the configured project and zone, and display all returned disks with name, size, type, and attachment status

#### Scenario: Display attachment status for an attached disk

- **WHEN** the disk list is returned and a disk has a non-empty `users` field
- **THEN** the app SHALL display that disk as "attached" and show the name of the VM it is attached to

#### Scenario: Display attachment status for an unattached disk

- **WHEN** the disk list is returned and a disk has an empty `users` field
- **THEN** the app SHALL display that disk as "unattached"

#### Scenario: No disks found

- **WHEN** the gcloud CLI returns zero disks for the configured project and zone
- **THEN** the app SHALL display a message indicating no persistent disks were found

#### Scenario: gcloud CLI error during listing

- **WHEN** the gcloud CLI command fails (non-zero exit code)
- **THEN** the app SHALL display the error message returned by gcloud and not render a partial disk list

---

### Requirement: Disk Selection

The user SHALL be able to select a disk from the list to configure and launch a VM from. Only one disk MAY be selected at a time.

#### Scenario: Select an unattached disk

- **WHEN** the user clicks on an unattached disk in the list
- **THEN** the app SHALL mark that disk as selected and present VM configuration options for launching a VM from that disk

#### Scenario: Select a disk that is already attached to a VM

- **WHEN** the user clicks on a disk that is currently attached to a VM
- **THEN** the app SHALL show the status of the attached VM rather than VM creation options

#### Scenario: Change selection

- **WHEN** the user selects a different disk while one is already selected
- **THEN** the app SHALL deselect the previous disk and select the new one

---

### Requirement: Disk Refresh

The user SHALL be able to manually refresh the disk list to reflect changes made outside the app.

#### Scenario: Manual refresh

- **WHEN** the user activates the refresh action
- **THEN** the app SHALL re-query `gcloud compute disks list` and update the displayed list with current results

#### Scenario: Refresh while a disk is selected

- **WHEN** the user refreshes the disk list while a disk is selected and that disk still exists
- **THEN** the app SHALL preserve the selection state after the refresh completes

#### Scenario: Refresh when selected disk no longer exists

- **WHEN** the user refreshes the disk list and the previously selected disk no longer exists
- **THEN** the app SHALL clear the selection and display the updated list without a selected disk
