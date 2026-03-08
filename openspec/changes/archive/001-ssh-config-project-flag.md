# Change: SSH config-ssh missing --project flag

**Date:** 2026-03-07
**Capability:** ssh-config
**Status:** Applied

## Problem

`gcloud compute config-ssh` was invoked without `--project`, causing it to use the default gcloud project instead of the project configured in the app. Additionally, errors from this command were silently discarded.

## Root Cause

In `src-tauri/src/gcloud/ssh.rs`, the `configure_ssh` function accepted a `project` parameter but never passed it to the gcloud command. In `src-tauri/src/monitor.rs`, the result was discarded with `let _ =`.

## Changes

### `src-tauri/src/gcloud/ssh.rs`
- Added `--project` argument to the `gcloud compute config-ssh` invocation.

### `src-tauri/src/monitor.rs`
- Replaced silent error discard (`let _ =`) with `eprintln!` logging on failure.

## Validation

- All 81 existing tests pass.
- `FakeRunner` prefix matching continues to work since `"compute config-ssh"` still matches the expanded command.
