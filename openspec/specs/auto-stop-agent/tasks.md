# Auto-Stop Agent - Tasks

schema: rks-sdd
status: draft
created: 2026-03-07

---

## 1. Agent Shell Scripts

- [ ] 1.1 Write `auto-stop-check.sh` — idle detection script (read config, check SSH via `ss`, manage timestamp file, trigger shutdown)
- [ ] 1.2 Write `auto-stop-startup.sh` — startup script that installs check script, systemd service, timer, and default config
- [ ] 1.3 Verify scripts are POSIX-compliant and work on Debian/Ubuntu GCE images

## 2. Shell Script Tests

- [ ] 2.1 Set up BATS test framework in `tests/auto-stop/`
- [ ] 2.2 Implement TC-AS-001 through TC-AS-009 (shell script tests)
- [ ] 2.3 Implement TC-AS-011, TC-AS-012 (grace period, config defaults)

## 3. Launcher Integration

- [ ] 3.1 Embed `auto-stop-startup.sh` content as a Rust const or bundled resource in `src-tauri/src/gcloud/`
- [ ] 3.2 Modify `create_instance` in `gcloud/instance.rs` to append `--metadata startup-script=...` to gcloud args
- [ ] 3.3 Implement TC-AS-010 (Rust test for metadata flag)

## 4. Documentation

- [ ] 4.1 Add auto-stop agent section to README (usage, override, configuration)
- [ ] 4.2 Document `/etc/auto-stop-agent.disabled` override in startup script comments
