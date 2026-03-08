## Why

VMs left running after SSH sessions end waste cloud compute budget. Users frequently forget to delete VMs after finishing work, especially with spot instances that may have been preempted and recreated. An on-VM agent that detects idle SSH sessions and triggers shutdown eliminates this cost leak automatically, without requiring the user to remember manual cleanup.

## What Changes

- New shell-based agent installed on VMs via `--metadata startup-script` during `gcloud compute instances create`.
- A systemd timer runs a check script every 5 minutes. If no SSH sessions have been active for a configurable idle timeout (default: 30 minutes), the VM initiates a clean shutdown.
- The launcher's existing `create_instance` gcloud command gains a `--metadata` flag to inject the agent script.
- A disable file (`/etc/auto-stop-agent.disabled`) provides a manual override.

## Capabilities

### New Capabilities
- `auto-stop-agent`: Systemd-based idle detection and auto-shutdown agent deployed to VMs via startup script metadata.

### Modified Capabilities
- `vm-lifecycle`: VM creation command adds `--metadata startup-script=...` to inject the auto-stop agent.

## Impact

- `src-tauri/src/gcloud/instance.rs` — create command gains metadata flag.
- New embedded shell script resource for the agent.
- No frontend changes required for MVP (agent runs silently on VM).
- Users can configure idle timeout via `UserPreferences` (future enhancement).
