# VM Lifecycle

**Capability:** Create and delete VMs from persistent disks

---

## ADDED Requirements

### Requirement: VM Creation

The app SHALL create a VM instance using `gcloud compute instances create` with the selected disk attached as the boot disk with `auto-delete=no`. The VM name SHALL be automatically derived as `{disk-name}-vm`.

#### Scenario: Create a VM from a selected disk

- **WHEN** the user selects an unattached disk and confirms VM creation
- **THEN** the app SHALL invoke `gcloud compute instances create {disk-name}-vm --disk=name={disk-name},boot=yes,auto-delete=no` with the configured machine type, zone, and project

#### Scenario: VM name derivation

- **WHEN** a disk named `ml-workspace` is selected for VM creation
- **THEN** the app SHALL create a VM named `ml-workspace-vm`

#### Scenario: Disk preservation on boot disk attachment

- **WHEN** the VM is created with the selected disk
- **THEN** the gcloud command SHALL include `auto-delete=no` to prevent disk deletion when the VM is later deleted

#### Scenario: Creation failure

- **WHEN** the `gcloud compute instances create` command returns a non-zero exit code
- **THEN** the app SHALL display the error message from gcloud and not transition to a running VM state

---

### Requirement: Machine Configuration

The user SHALL be able to configure the machine type. For N1 series machine types, GPU type and count SHALL be configurable separately via the `--accelerator` flag. For A2/A3 series machine types, the GPU is built into the machine type and SHALL NOT present separate GPU configuration.

#### Scenario: Configure N1 machine type with GPU

- **WHEN** the user selects an N1 series machine type (e.g., `n1-standard-8`) and configures a GPU (e.g., type=nvidia-tesla-t4, count=4)
- **THEN** the app SHALL include `--accelerator=type=nvidia-tesla-t4,count=4` and `--maintenance-policy=TERMINATE` in the gcloud create command

#### Scenario: Configure A2 machine type

- **WHEN** the user selects an A2 series machine type (e.g., `a2-highgpu-1g`)
- **THEN** the app SHALL NOT present GPU type/count selectors and SHALL NOT include a separate `--accelerator` flag, since the GPU is inherent to the machine type

#### Scenario: Configure A3 machine type

- **WHEN** the user selects an A3 series machine type
- **THEN** the app SHALL NOT present GPU type/count selectors and SHALL NOT include a separate `--accelerator` flag

#### Scenario: N1 without GPU

- **WHEN** the user selects an N1 series machine type and does not add a GPU
- **THEN** the app SHALL NOT include `--accelerator` or `--maintenance-policy=TERMINATE` in the gcloud command

---

### Requirement: Spot Provisioning

The app SHALL default to spot provisioning (`--provisioning-model=SPOT`). The user SHALL be able to switch to on-demand provisioning.

#### Scenario: Default provisioning model

- **WHEN** the user opens the VM configuration panel for a selected disk
- **THEN** the spot provisioning option SHALL be selected by default

#### Scenario: Create VM with spot provisioning

- **WHEN** the user creates a VM with spot provisioning selected
- **THEN** the gcloud command SHALL include `--provisioning-model=SPOT`

#### Scenario: Switch to on-demand provisioning

- **WHEN** the user toggles the provisioning model to on-demand
- **THEN** the gcloud command SHALL NOT include `--provisioning-model=SPOT`

---

### Requirement: VM Deletion

The app SHALL delete VMs using `gcloud compute instances delete --keep-disks=all --quiet` to preserve the persistent disk.

#### Scenario: Delete a running VM

- **WHEN** the user requests deletion of a running VM
- **THEN** the app SHALL invoke `gcloud compute instances delete {vm-name} --zone={zone} --project={project} --keep-disks=all --quiet`

#### Scenario: Disk preserved after VM deletion

- **WHEN** the VM deletion command completes successfully
- **THEN** the persistent disk SHALL remain in the disk list as unattached and available for future VM creation

#### Scenario: Deletion failure

- **WHEN** the `gcloud compute instances delete` command fails
- **THEN** the app SHALL display the error message and retain the current VM state

---

### Requirement: Configuration Presets

The app SHALL offer curated preset configurations (e.g., "ML Training: 8 vCPU / 30GB / 4x T4") as quick-select buttons. The user SHALL be able to expand an advanced section for full manual configuration.

#### Scenario: Select a preset configuration

- **WHEN** the user clicks a preset button such as "ML Training: 8 vCPU / 30GB / 4x T4"
- **THEN** the app SHALL populate the machine type, GPU type, and GPU count fields with the preset values (e.g., `n1-standard-8`, `nvidia-tesla-t4`, count 4)

#### Scenario: Preset overrides previous manual configuration

- **WHEN** the user has manually configured machine type and GPU settings and then selects a preset
- **THEN** the app SHALL replace all manual configuration values with the preset values

#### Scenario: Expand advanced configuration after preset

- **WHEN** the user selects a preset and then expands the advanced configuration section
- **THEN** the app SHALL show the preset values in the individual fields, and the user SHALL be able to modify them

#### Scenario: Advanced configuration without preset

- **WHEN** the user expands the advanced configuration section without selecting a preset
- **THEN** the app SHALL display all configurable fields with their default or previously saved values
