# CGE Launcher - Design Document

schema: rks-sdd
status: draft
created: 2026-03-06

---

## Context

CGE Launcher is a cross-platform desktop application for managing Google Compute Engine virtual machines with persistent disks. The core workflow treats disks as the durable artifact and VMs as ephemeral compute that attaches to them. Users create persistent disks once (outside this app), then use the launcher to spin up VMs with chosen machine type and GPU configurations, monitor VM status in real time, and delete VMs while preserving disks.

The primary use case is ML workloads running on Windows/WSL, accessed via VS Code Remote SSH.

**Stack**: Tauri 2.x (Rust backend) + React (TypeScript frontend) + Tailwind CSS + Vite

## Goals / Non-Goals

### Goals

- Simple dashboard for GCE VM lifecycle management (create, monitor, delete).
- Real-time status monitoring with color-coded indicators.
- Cost estimation before VM launch using an embedded pricing table.
- Automatic SSH config generation for VS Code Remote SSH integration.
- Cross-platform support with Windows/WSL as the primary target.

### Non-Goals

- Not a full GCP console replacement.
- No VM-internal monitoring (CPU/memory utilization from inside the VM).
- No multi-project or multi-zone simultaneous view.
- No disk creation or deletion (managed outside the app).
- No live pricing API integration for MVP (embedded pricing table only).

## Decisions

### 1. gcloud CLI over raw API

All GCE operations shell out to `gcloud` with `--format=json`. This provides simpler auth handling (works with both user credentials and service accounts) and eliminates Google API client library dependencies.

**Trade-off**: Requires gcloud CLI installed on the host.

### 2. Delete vs Stop

Use `gcloud compute instances delete --keep-disks=all` instead of `gcloud compute instances stop`. Stopped VMs still incur costs (IP allocation, resource reservation). The persistent disk is the durable artifact; the VM is disposable.

### 3. VM naming convention

Auto-derive VM name from disk name using the pattern `{disk}-vm`. No user input required for VM naming.

### 4. Trait-based executor

A `GcloudRunner` trait abstracts all CLI calls. Production code uses `CliRunner` (real subprocess execution via `tokio::process::Command`). Tests use `FakeRunner` (canned JSON responses). This is the primary testability boundary.

### 5. Background polling

A tokio task polls `gcloud compute instances describe` every 5 seconds and emits Tauri events to the frontend. Chosen over GCE push notifications for simplicity.

### 6. Embedded pricing

A static pricing table in Rust covers common machine types and GPUs. GCE pricing changes infrequently. No Cloud Billing API dependency for MVP.

### 7. Presets + Advanced configuration

Quick-select preset cards for common machine/GPU configurations, plus an expandable advanced section for full manual configuration.

## Architecture Overview

```
+-----------------------+          +----------------------------+
|   React Frontend      |          |   Tauri Rust Backend       |
|                       |          |                            |
| src/components/       |  invoke  | src-tauri/src/commands/    |
|   Layout              | -------> |   disk.rs                  |
|   DiskList / DiskItem |          |   vm.rs                    |
|   ConfigPanel         |  events  |   config.rs                |
|   MachineConfig       | <------- |   pricing.rs               |
|   PricingDisplay      |          |   auth.rs                  |
|   VmControls          |          |   ssh.rs                   |
|   VmStatusBadge       |          |                            |
|   ResourceSummary     |          | src-tauri/src/gcloud/      |
|   SettingsPanel       |          |   executor.rs (trait)      |
|                       |          |   disk.rs                  |
| src/hooks/            |          |   instance.rs              |
|   useDisks            |          |   pricing.rs               |
|   useVmStatus         |          |   auth.rs                  |
|   usePricing          |          |   ssh.rs                   |
|   useConfig           |          |                            |
|                       |          | src-tauri/src/monitor.rs   |
| src/lib/              |          | src-tauri/src/state.rs     |
|   tauri.ts            |          | src-tauri/src/models/      |
|   types.ts            |          |                            |
+-----------------------+          +----------------------------+
                                              |
                                              | subprocess
                                              v
                                   +---------------------+
                                   |   gcloud CLI        |
                                   |   --format=json     |
                                   +---------------------+
                                              |
                                              v
                                   +---------------------+
                                   |   Google Compute    |
                                   |   Engine API        |
                                   +---------------------+
```

### Backend layers

- **`src-tauri/src/gcloud/executor.rs`** - Core abstraction. `GcloudRunner` trait defines the interface for all gcloud calls. `CliRunner` executes `gcloud {args} --project={project} --format=json` via `tokio::process::Command`. Service account auth supported via `CLOUDSDK_AUTH_CREDENTIAL_FILE_OVERRIDE` env var.
- **`src-tauri/src/gcloud/`** - Business logic modules: `disk.rs` (list and parse disks), `instance.rs` (create, delete, describe VMs), `pricing.rs` (embedded pricing table and cost calculation), `auth.rs` (check authentication state), `ssh.rs` (generate SSH config).
- **`src-tauri/src/commands/`** - Thin `#[tauri::command]` handlers: `disk.rs`, `vm.rs`, `config.rs`, `pricing.rs`, `auth.rs`, `ssh.rs`. These do argument validation and delegate to gcloud modules.
- **`src-tauri/src/models/`** - Serde-serializable types shared between backend and frontend: `Disk`, `Instance`, `MachineConfig`, `PricingEstimate`, `VmStatus`, `UserPreferences`.
- **`src-tauri/src/monitor.rs`** - Background tokio task for VM status polling. Emits `vm-status-update` Tauri events to the frontend.
- **`src-tauri/src/state.rs`** - Tauri managed state holding the executor instance, user configuration, and active monitor handles.

### Frontend layers

- **`src/components/`** - React UI components: `Layout`, `DiskList`, `DiskItem`, `ConfigPanel`, `MachineConfig`, `PricingDisplay`, `VmControls`, `VmStatusBadge`, `ResourceSummary`, `SettingsPanel`.
- **`src/hooks/`** - Custom React hooks: `useDisks` (fetch and cache disk list), `useVmStatus` (subscribe to Tauri `vm-status-update` events), `usePricing` (debounced cost estimation), `useConfig` (user preferences).
- **`src/lib/`** - `tauri.ts` (typed `invoke` wrappers for Tauri commands), `types.ts` (TypeScript types mirroring Rust models).

## Data Models Reference

See `openspec/datamodels.yaml` for full schemas covering:

- `Disk` - Persistent disk metadata (name, zone, size, type, status).
- `VmStatus` - VM lifecycle state (status enum, IP addresses, timestamps).
- `MachineConfig` - VM configuration (machine type, GPU type/count, spot flag, boot disk).
- `PricingEstimate` - Hourly and monthly cost breakdown by resource.
- `UserPreferences` - Saved project, zone, default configs.

## API Contract Reference

See `openspec/apis.yaml` for Tauri command contracts:

- `list_disks` - List persistent disks in the configured project/zone.
- `start_vm` - Create an ephemeral VM attached to a selected disk.
- `stop_vm` - Delete a VM while preserving all attached disks.
- `estimate_pricing` - Calculate cost estimate for a given machine configuration.
- `check_auth` - Verify gcloud authentication state.
- `config_ssh` - Generate SSH config for a running VM.
- `get_preferences` / `set_preferences` - Read and write user configuration.

## Risks / Trade-offs

| # | Risk | Impact | Mitigation |
|---|------|--------|------------|
| 1 | gcloud CLI dependency | App non-functional without gcloud installed | Check for gcloud on startup, show clear error with install instructions if missing |
| 2 | Spot VM preemption | VM terminated without user action | Status monitor detects termination, shows red indicator immediately |
| 3 | Pricing staleness | Embedded pricing table drifts from actual GCE pricing | Table updated with each app release; future enhancement: Cloud Billing API refresh |
| 4 | SSH config races | SSH config written before VM is reachable | Run `gcloud compute config-ssh` only after VM reaches RUNNING state, not during PROVISIONING |
| 5 | WSL path issues | SSH config written to wrong location on Windows | gcloud inside WSL updates WSL's `~/.ssh/config`, which VS Code Remote SSH reads correctly |
