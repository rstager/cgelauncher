# CGE Launcher - Test Plan

schema: rks-sdd
status: draft
created: 2026-03-06

---

## Test Strategy

Testing is organized in four layers, each targeting a different boundary:

1. **Unit tests (Rust)**: Validate gcloud CLI argument construction, JSON response parsing, pricing calculation logic, and configuration serialization. These test the `gcloud/` modules in isolation using the `FakeRunner` trait implementation.

2. **Unit tests (React)**: Validate component rendering, hook behavior, and UI state transitions. Components are tested with mock Tauri invoke responses; hooks are tested for correct state management and event subscription.

3. **Integration tests**: Validate Tauri command handlers (`commands/`) end-to-end through the Rust backend using `FakeRunner` for canned gcloud responses. These verify that commands correctly wire argument validation, gcloud module calls, and response serialization.

4. **E2E tests**: Validate full user workflows from the UI through the backend with a mocked executor. Cover the critical path: open app, select disk, configure VM, launch, monitor status, delete.

## Framework and Tooling

| Layer | Tool | Run Command |
|-------|------|-------------|
| Rust unit + integration | `cargo test` (built-in test framework) | `cargo test` |
| React unit | Vitest + React Testing Library | `npm test` |
| E2E | Playwright or Tauri WebDriver | `npm run test:e2e` |

## Automation Plan

- `cargo test` - Run all Rust unit and integration tests. Must pass before merge.
- `npm test` - Run all frontend unit tests via Vitest. Must pass before merge.
- `npm run test:e2e` - Run E2E smoke tests. Required for release branches.
- CI runs `cargo test` and `npm test` on every PR. E2E tests run on merge to main.

## Coverage and Scope

All six capabilities are covered:

| Capability | Unit (Rust) | Unit (React) | Integration | E2E |
|------------|:-----------:|:------------:|:-----------:|:---:|
| Disk Management | x | x | x | x |
| VM Lifecycle | x | x | x | x |
| Pricing Estimation | x | x | x | |
| Status Monitoring | x | x | x | x |
| SSH Config | x | | x | |
| Configuration | x | x | x | |

## Test Cases

| ID | Name | Purpose | Type | Preconditions | Steps / Input | Expected Results | Priority |
|----|------|---------|------|---------------|---------------|------------------|----------|
| TC-001 | List disks parsing | Verify JSON response from `gcloud compute disks list` is parsed into `Vec<Disk>` | Unit (Rust) | FakeRunner returns valid disk list JSON | Call `list_disks` with FakeRunner returning 3 disks | Returns 3 `Disk` structs with correct name, size, type, and users fields | High |
| TC-002 | Empty disk list | Verify empty JSON array is handled | Unit (Rust) | FakeRunner returns `[]` | Call `list_disks` | Returns empty `Vec<Disk>` without error | High |
| TC-003 | Disk selection | Verify clicking an unattached disk marks it selected and shows config panel | Unit (React) | DiskList rendered with 3 disks, one unattached | Click the unattached disk | Disk shows selected state; ConfigPanel renders | High |
| TC-004 | Disk refresh | Verify refresh re-fetches disk list and preserves selection | Unit (React) | DiskList rendered with selected disk | Click refresh button | `list_disks` invoked again; previously selected disk remains selected if still present | Medium |
| TC-005 | Attached disk display | Verify attached disk shows VM name and status badge | Unit (React) | DiskList rendered with a disk whose `users` field contains a VM name | Render DiskItem | DiskItem displays "attached" label with VM name and VmStatusBadge | High |
| TC-006 | Create N1 VM with GPU args | Verify gcloud command includes `--accelerator` and `--maintenance-policy` for N1 + GPU | Unit (Rust) | FakeRunner captures command args | Call `create_instance` with machine_type=`n1-standard-8`, gpu_type=`nvidia-tesla-t4`, gpu_count=4 | Command args include `--accelerator=type=nvidia-tesla-t4,count=4` and `--maintenance-policy=TERMINATE` | High |
| TC-007 | Create A2 VM without accelerator | Verify A2 machine type does not include `--accelerator` flag | Unit (Rust) | FakeRunner captures command args | Call `create_instance` with machine_type=`a2-highgpu-1g` | Command args do NOT include `--accelerator` or `--maintenance-policy` | High |
| TC-008 | Spot flag included | Verify spot provisioning adds `--provisioning-model=SPOT` | Unit (Rust) | FakeRunner captures command args | Call `create_instance` with spot=true | Command args include `--provisioning-model=SPOT` | High |
| TC-009 | On-demand flag | Verify on-demand provisioning omits spot flag | Unit (Rust) | FakeRunner captures command args | Call `create_instance` with spot=false | Command args do NOT include `--provisioning-model=SPOT` | High |
| TC-010 | Delete with keep-disks | Verify delete command includes `--keep-disks=all --quiet` | Unit (Rust) | FakeRunner captures command args | Call `delete_instance` with vm_name=`test-vm` | Command args include `--keep-disks=all` and `--quiet` | High |
| TC-011 | VM name derivation | Verify VM name is `{disk-name}-vm` | Unit (Rust) | None | Derive VM name from disk name `ml-workspace` | Result is `ml-workspace-vm` | High |
| TC-012 | Preset selection | Verify preset populates machine type, GPU type, and GPU count | Unit (React) | MachineConfig rendered with presets | Click "ML Training" preset card | Machine type set to `n1-standard-8`, GPU type to `nvidia-tesla-t4`, GPU count to 4 | Medium |
| TC-013 | N1 pricing breakdown | Verify N1 pricing returns separate vCPU, memory, and GPU line items | Unit (Rust) | Embedded pricing table loaded | Call `estimate_pricing` with `n1-standard-8`, `nvidia-tesla-t4`, count=4 | Returns PricingEstimate with vcpu_cost, memory_cost, and gpu_cost fields all > 0 | High |
| TC-014 | A2 all-in pricing | Verify A2 pricing returns a single total line item | Unit (Rust) | Embedded pricing table loaded | Call `estimate_pricing` with `a2-highgpu-1g` | Returns PricingEstimate with a single machine_cost and no separate gpu_cost | High |
| TC-015 | Monthly projection | Verify monthly cost equals hourly cost times 730 | Unit (Rust) | None | Call `estimate_pricing` returning hourly_total=2.50 | monthly_total equals 1825.00 | High |
| TC-016 | Spot vs on-demand pricing | Verify both spot and on-demand rates are returned | Unit (Rust) | Embedded pricing table loaded | Call `estimate_pricing` with spot=true | Returns both spot_hourly and ondemand_hourly; spot_hourly < ondemand_hourly | Medium |
| TC-017 | Zero GPU pricing | Verify N1 without GPU has no GPU cost component | Unit (Rust) | Embedded pricing table loaded | Call `estimate_pricing` with `n1-standard-4`, no GPU | Returns PricingEstimate with gpu_cost=0 or None | Medium |
| TC-018 | Running status green | Verify RUNNING status renders green dot | Unit (React) | VmStatusBadge rendered with status=RUNNING | Render component | Green dot element is present | High |
| TC-019 | Provisioning status yellow | Verify PROVISIONING status renders yellow pulsing dot | Unit (React) | VmStatusBadge rendered with status=PROVISIONING | Render component | Yellow dot element with pulse animation class is present | High |
| TC-020 | Terminated status red | Verify TERMINATED status renders red dot | Unit (React) | VmStatusBadge rendered with status=TERMINATED | Render component | Red dot element is present | High |
| TC-021 | Auto-discovery on launch | Verify app polls status for attached VMs on startup | Integration | FakeRunner returns disk list with 2 attached disks | Initialize app state and trigger startup | Monitor starts polling for both attached VM names | High |
| TC-022 | Event emission on status change | Verify backend emits `vm-status-update` only when status changes | Unit (Rust) | Monitor running with last_status=PROVISIONING | Poll returns status=RUNNING | `vm-status-update` event emitted with new status RUNNING | High |
| TC-023 | Config-ssh invocation after start | Verify `gcloud compute config-ssh` is called after successful VM creation | Unit (Rust) | FakeRunner captures command args; create succeeds | Call `create_instance` then verify ssh config call | `config-ssh` command invoked with correct `--project` flag | High |
| TC-024 | Connection string display | Verify SSH connection string shown for running VM | Unit (React) | VM status is RUNNING, vm_name=`test-vm`, zone=`us-central1-a`, project=`my-project` | Render status bar | Displays `ssh test-vm.us-central1-a.my-project` | Medium |
| TC-025 | Async SSH execution | Verify config-ssh runs asynchronously without blocking VM status transition | Integration | FakeRunner with delayed ssh response | Create VM and check status updates | VM status updates to RUNNING before config-ssh completes | Medium |
| TC-026 | Save and load preferences | Verify preferences round-trip through JSON config file | Unit (Rust) | Temp directory for config file | Save UserPreferences, then load from same path | Loaded preferences match saved values | High |
| TC-027 | Auth status check | Verify `gcloud auth list` output is parsed for active account | Unit (Rust) | FakeRunner returns auth list JSON with active account | Call `check_auth` | Returns active account email | High |
| TC-028 | Service account config | Verify service account key path sets env var on gcloud calls | Unit (Rust) | FakeRunner captures environment | Set service_account_key_path in config, then invoke any gcloud command | `CLOUDSDK_AUTH_CREDENTIAL_FILE_OVERRIDE` env var set to the key path | High |
| TC-029 | Default values applied | Verify saved defaults pre-populate VM configuration | Unit (React) | useConfig returns defaults: machine_type=`n1-standard-8`, spot=true | Render MachineConfig for a selected disk | Machine type field shows `n1-standard-8`; spot toggle is on | Medium |
| TC-030 | Missing config file | Verify app starts with built-in defaults when no config file exists | Unit (Rust) | No config file at expected path | Call `load_preferences` | Returns default UserPreferences without error | High |

## Traceability Matrix

| Spec | Requirement | Scenarios | Test Case IDs |
|------|-------------|-----------|---------------|
| disk-management | Disk Listing | List disks, Attachment status (attached), Attachment status (unattached), No disks found, gcloud error | TC-001, TC-002, TC-005 |
| disk-management | Disk Selection | Select unattached, Select attached, Change selection | TC-003 |
| disk-management | Disk Refresh | Manual refresh, Refresh with selection preserved, Refresh when selected disk removed | TC-004 |
| vm-lifecycle | VM Creation | Create from disk, VM name derivation, Disk preservation, Creation failure | TC-006, TC-007, TC-011 |
| vm-lifecycle | Machine Configuration | N1 with GPU, A2 without accelerator, A3 without accelerator, N1 without GPU | TC-006, TC-007 |
| vm-lifecycle | Spot Provisioning | Default spot, Create with spot, Switch to on-demand | TC-008, TC-009 |
| vm-lifecycle | VM Deletion | Delete running VM, Disk preserved, Deletion failure | TC-010 |
| vm-lifecycle | Configuration Presets | Select preset, Preset overrides manual, Advanced after preset, Advanced without preset | TC-012 |
| pricing-estimation | Cost Estimation | Spot and on-demand display, Cost updates on change, No GPU cost | TC-013, TC-016, TC-017 |
| pricing-estimation | Pricing Breakdown | Breakdown with GPU, Breakdown without GPU, A2/A3 single line | TC-013, TC-014, TC-017 |
| pricing-estimation | Monthly Projection | Monthly calculation, Both models | TC-015 |
| pricing-estimation | Embedded Pricing | Offline availability, Covers all types, Unknown fallback | TC-013, TC-014 |
| status-monitoring | Status Polling | Poll running, Not found, Stop after delete, Transient error | TC-021, TC-022 |
| status-monitoring | Color Indicators | Running green, Provisioning yellow, Staging yellow, Terminated red, Stopped red, Not found red | TC-018, TC-019, TC-020 |
| status-monitoring | Resource Summary | Display for running, Not shown for terminated | TC-018 |
| status-monitoring | Auto-Discovery | Discover on launch, No running VMs, Multiple VMs | TC-021 |
| status-monitoring | Event-Driven Updates | Emit on change, Frontend receives, No emit when unchanged | TC-022 |
| ssh-config | Auto-Update | Config-ssh after creation, Update failure, Not triggered on failure | TC-023, TC-025 |
| ssh-config | Connection Display | Display connection string, Not shown for non-running, Copy to clipboard | TC-024 |
| ssh-config | Notification | Ready notification, Timing | TC-025 |
| configuration | Project/Zone Config | Set project and zone, Missing project, Change project | TC-026 |
| configuration | Auth Config | Default CLI auth, Service account key, Invalid key path | TC-028 |
| configuration | Auth Status | Display active account, No auth, Service account active | TC-027 |
| configuration | Persistence | Save on change, Load on startup, First launch no file, Corrupted file | TC-026, TC-030 |
| configuration | Defaults | Default machine type, Default GPU, Default spot, Persist across restarts | TC-029 |

## Merge Gates

All of the following must pass before a PR can merge:

1. `cargo test` -- all Rust unit and integration tests pass.
2. `npm test` -- all frontend unit tests pass via Vitest.
3. No test case may be skipped or marked `#[ignore]` without a linked tracking issue.
4. New functionality must include corresponding test cases from the table above.
