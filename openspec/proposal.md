## Why

Managing Google Compute Engine VMs with persistent disks for ML workloads requires repetitive manual gcloud commands: creating instances with the right machine type, GPU configuration, and spot pricing, then deleting them while preserving disks. There is no simple tool to visualize disk/VM status, estimate costs before launching, or automatically configure SSH for VS Code Remote connections. This app eliminates that friction with a single dashboard.

## What Changes

- New cross-platform desktop application (Tauri + React) for GCE VM management
- Dashboard UI showing persistent disks with real-time VM status indicators (green/yellow/red)
- VM creation from persistent disks with configurable machine type, GPU, and spot/on-demand pricing
- VM deletion with automatic disk preservation (`--keep-disks=all`)
- Estimated spot and on-demand pricing display before VM launch
- Automatic SSH config update via `gcloud compute config-ssh` after VM start
- Configuration persistence for project, zone, and default machine settings
- Support for both gcloud CLI auth and service account key file auth

## Capabilities

### New Capabilities
- `disk-management`: List and select persistent disks in a project/zone
- `vm-lifecycle`: Create and delete VMs from persistent disks with machine/GPU configuration
- `pricing-estimation`: Show estimated spot and on-demand hourly costs before launching
- `status-monitoring`: Real-time VM status polling with color-coded indicators
- `ssh-config`: Automatic SSH config update after VM starts for VS Code Remote SSH
- `configuration`: Persist and manage project, zone, auth, and default machine settings

### Modified Capabilities

## Impact

- New Tauri desktop application (Rust backend + React frontend)
- Depends on `gcloud` CLI being installed and authenticated
- Reads/writes `~/.ssh/config` via `gcloud compute config-ssh`
- Stores user preferences in Tauri app data directory
