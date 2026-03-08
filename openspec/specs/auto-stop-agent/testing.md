# Auto-Stop Agent - Test Plan

schema: rks-sdd
status: draft
created: 2026-03-07

---

## Test Strategy

Testing covers two domains:

1. **Shell script unit tests**: Validate the agent's idle detection logic, timestamp management, disable file check, and shutdown trigger. Tested using BATS (Bash Automated Testing System) or inline shell test harness.

2. **Rust integration tests**: Validate that `create_instance` includes the correct `--metadata startup-script=...` argument via the existing `FakeRunner` pattern.

The shell script is the primary artifact; Rust changes are minimal (metadata flag injection).

## Framework and Tooling

| Layer | Tool | Run Command |
|-------|------|-------------|
| Shell script tests | BATS (bats-core) | `bats tests/auto-stop/` |
| Rust integration | cargo test | `cargo test auto_stop` |

## Automation Plan

- `bats tests/auto-stop/` — Run shell script tests. Must pass before merge.
- `cargo test` — Existing Rust test suite, extended with auto-stop metadata tests.
- Shell tests run in CI using a minimal Docker container (or WSL locally).

## Coverage and Scope

| Behavior | Shell Test | Rust Test |
|----------|:---------:|:---------:|
| SSH session detection | x | |
| Idle timeout calculation | x | |
| Timestamp file management | x | |
| Disable file check | x | |
| Shutdown trigger | x | |
| Grace period | x | |
| Config file parsing | x | |
| Metadata flag in gcloud cmd | | x |
| Startup script idempotency | x | |

## Test Cases

| ID | Name | Purpose | Type | Preconditions | Steps/Input | Expected Results | Priority |
|----|------|---------|------|---------------|-------------|------------------|----------|
| TC-AS-001 | Active SSH prevents shutdown | Verify no shutdown when SSH sessions exist | Shell | Mock `ss` returns active connection on port 22 | Run check script | `last-active` file updated to current time; no shutdown called | Critical |
| TC-AS-002 | Idle timeout triggers shutdown | Verify shutdown after idle period exceeds timeout | Shell | Mock `ss` returns no connections; `last-active` is 31 minutes ago; timeout=1800s | Run check script | `shutdown +1` is called | Critical |
| TC-AS-003 | Idle within timeout no shutdown | Verify no shutdown when idle but within timeout | Shell | Mock `ss` returns no connections; `last-active` is 10 minutes ago; timeout=1800s | Run check script | No shutdown called; `last-active` unchanged | Critical |
| TC-AS-004 | Disable file prevents shutdown | Verify no shutdown when disable file exists | Shell | `/etc/auto-stop-agent.disabled` exists; idle > timeout | Run check script | Script exits without calling shutdown | Critical |
| TC-AS-005 | Custom timeout from config | Verify config file overrides default timeout | Shell | `/etc/default/auto-stop` contains `AUTO_STOP_TIMEOUT=600`; idle for 11 minutes | Run check script | Shutdown triggered (600s < 660s idle) | High |
| TC-AS-006 | Missing last-active file | Verify first run initializes timestamp | Shell | `/var/run/auto-stop-agent/last-active` does not exist; SSH sessions active | Run check script | File created with current timestamp | High |
| TC-AS-007 | Missing last-active no SSH | Verify first run with no SSH sets timestamp and does not shutdown | Shell | No `last-active` file; no SSH sessions | Run check script | File created with current timestamp; no shutdown (first run grace) | High |
| TC-AS-008 | Startup script installs units | Verify startup script creates systemd files | Shell | Clean system (no existing units) | Run startup script in test environment | `/usr/local/bin/auto-stop-check.sh`, `/etc/systemd/system/auto-stop.service`, `/etc/systemd/system/auto-stop.timer` all exist | High |
| TC-AS-009 | Startup script is idempotent | Verify running startup script twice does not fail | Shell | Units already installed from previous run | Run startup script again | No errors; files unchanged; timer still running | High |
| TC-AS-010 | Metadata flag in create command | Verify gcloud create includes startup-script metadata | Rust | FakeRunner captures command args | Call `create_instance` with default config | Command args include `--metadata` with `startup-script=` prefix | High |
| TC-AS-011 | Grace period wall message | Verify wall message is sent before shutdown | Shell | Idle > timeout; no disable file | Run check script | `wall` called with shutdown warning before `shutdown +1` | Medium |
| TC-AS-012 | Config defaults when no file | Verify defaults used when config file is missing | Shell | No `/etc/default/auto-stop` file | Run check script | Timeout defaults to 1800s | Medium |

## Traceability Matrix

| Spec Reference | Test Case IDs |
|---|---|
| auto-stop-agent :: Requirement: Idle Detection | TC-AS-001, TC-AS-002, TC-AS-003 |
| auto-stop-agent :: Requirement: Configurable Timeout | TC-AS-005, TC-AS-012 |
| auto-stop-agent :: Requirement: Safe Shutdown | TC-AS-002, TC-AS-011 |
| auto-stop-agent :: Requirement: Systemd Integration | TC-AS-008, TC-AS-009 |
| auto-stop-agent :: Requirement: Manual Override | TC-AS-004 |
| auto-stop-agent :: Scenario: Idle VM is auto-stopped | TC-AS-002 |
| auto-stop-agent :: Scenario: Active SSH session prevents shutdown | TC-AS-001 |
| auto-stop-agent :: Scenario: Timeout is configurable | TC-AS-005 |
| auto-stop-agent :: Scenario: Manual override disables agent | TC-AS-004 |
| vm-lifecycle (modified) :: Metadata injection | TC-AS-010 |

## Merge Gates

1. `bats tests/auto-stop/` — all shell script tests pass.
2. `cargo test` — all Rust tests pass (including new metadata tests).
3. No test case may be skipped without a linked tracking issue.
