# CGE Launcher - Task Breakdown

schema: rks-sdd
status: draft
created: 2026-03-06

---

## 1. Project Setup

- [ ] 1.1 Initialize Tauri + React + Vite project
- [ ] 1.2 Add Tailwind CSS
- [ ] 1.3 Configure Cargo.toml dependencies (serde, serde_json, tokio, async-trait)
- [ ] 1.4 Create src-tauri/src/ module structure (commands/, gcloud/, models/)

## 2. Backend Core - Executor

- [ ] 2.1 Define GcloudRunner trait and GcloudError type
- [ ] 2.2 Implement CliRunner (real gcloud subprocess)
- [ ] 2.3 Implement FakeRunner (canned responses for tests)
- [ ] 2.4 Write executor unit tests

## 3. Backend - Disk Management

- [ ] 3.1 Implement gcloud/disk.rs (list_disks parsing)
- [ ] 3.2 Implement commands/disk.rs (list_disks Tauri command)
- [ ] 3.3 Write disk listing unit tests

## 4. Backend - VM Lifecycle

- [ ] 4.1 Implement gcloud/instance.rs (create, delete, describe)
- [ ] 4.2 Implement commands/vm.rs (start_vm, stop_vm Tauri commands)
- [ ] 4.3 Implement N1 GPU argument construction
- [ ] 4.4 Implement A2/A3 machine type handling
- [ ] 4.5 Write VM lifecycle unit tests

## 5. Backend - Status Monitoring

- [ ] 5.1 Implement monitor.rs (background polling task)
- [ ] 5.2 Implement Tauri event emission (vm-status-update)
- [ ] 5.3 Implement auto-discovery on startup
- [ ] 5.4 Write monitoring unit tests

## 6. Backend - Pricing

- [ ] 6.1 Implement embedded pricing table (gcloud/pricing.rs)
- [ ] 6.2 Implement pricing calculation (N1 component, A2 all-in)
- [ ] 6.3 Implement commands/pricing.rs (estimate_pricing Tauri command)
- [ ] 6.4 Write pricing unit tests

## 7. Backend - Auth & SSH

- [ ] 7.1 Implement gcloud/auth.rs (check auth, service account)
- [ ] 7.2 Implement gcloud/ssh.rs (config-ssh invocation)
- [ ] 7.3 Implement commands/auth.rs and commands/ssh.rs
- [ ] 7.4 Write auth and SSH unit tests

## 8. Backend - Configuration

- [ ] 8.1 Implement models/config.rs (UserPreferences)
- [ ] 8.2 Implement state.rs (Tauri managed state)
- [ ] 8.3 Implement commands/config.rs (get/set preferences)
- [ ] 8.4 Implement config file persistence (JSON read/write)
- [ ] 8.5 Write configuration unit tests

## 9. Frontend - Layout & Disk List

- [ ] 9.1 Create Layout component (left/right panels)
- [ ] 9.2 Create DiskList component
- [ ] 9.3 Create DiskItem component with VmStatusBadge
- [ ] 9.4 Create useDisks hook
- [ ] 9.5 Create typed Tauri invoke wrappers (lib/tauri.ts)
- [ ] 9.6 Create TypeScript types (lib/types.ts)

## 10. Frontend - Config Panel & Controls

- [ ] 10.1 Create ConfigPanel component
- [ ] 10.2 Create MachineConfig with preset cards
- [ ] 10.3 Create MachineConfig advanced section
- [ ] 10.4 Create VmControls component (Start/Stop buttons)
- [ ] 10.5 Create ResourceSummary component

## 11. Frontend - Pricing & Settings

- [ ] 11.1 Create PricingDisplay component
- [ ] 11.2 Create usePricing hook (debounced)
- [ ] 11.3 Create SettingsPanel component
- [ ] 11.4 Create AuthStatus component
- [ ] 11.5 Create useConfig hook

## 12. Frontend - Status Monitoring

- [ ] 12.1 Create useVmStatus hook (Tauri event listener)
- [ ] 12.2 Integrate status updates into DiskList
- [ ] 12.3 Add SSH connection string to status bar

## 13. Frontend Tests

- [ ] 13.1 Write DiskList component tests
- [ ] 13.2 Write MachineConfig component tests
- [ ] 13.3 Write PricingDisplay component tests
- [ ] 13.4 Write VmControls component tests
- [ ] 13.5 Write hook tests (useDisks, useVmStatus, usePricing)

## 14. Integration & E2E

- [ ] 14.1 Write Tauri command integration tests with FakeRunner
- [ ] 14.2 Write E2E smoke test (full workflow)
- [ ] 14.3 Verify cargo test passes
- [ ] 14.4 Verify npm test passes
