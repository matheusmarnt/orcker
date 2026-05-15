---
phase: 04-power-tools
plan: "06"
subsystem: m7-settings
tags: [tray, updater, autostart, close-to-tray, sonner, tauri-plugin]
dependency_graph:
  requires: [04-02]
  provides: [system-tray, close-to-tray, auto-updater, autostart-plugin]
  affects: [lib.rs, main.ts, capabilities]
tech_stack:
  added:
    - "@tauri-apps/plugin-process@2.3.1"
  patterns:
    - TrayIconBuilder with hot-reload guard (tray_by_id check)
    - on_window_event sync handler reads AtomicBool (no tokio await)
    - checkForUpdate composable with non-blocking startup call
    - vi.mocked() pattern for typed mock access (avoids TDZ hoisting)
key_files:
  created:
    - src/composables/useUpdater.ts
    - src/composables/__tests__/useUpdater.spec.ts
  modified:
    - src-tauri/src/lib.rs
    - src-tauri/tauri.conf.json
    - src-tauri/capabilities/default.json
    - src-tauri/Cargo.toml
    - src/main.ts
    - src/ipc/bindings.ts
decisions:
  - show_menu_on_left_click replaces deprecated menu_on_left_click in tauri 2.x
  - tray submenu uses "(no projects)" placeholder at startup; projects list empty at setup time
  - chrono feature is "clock" (not "local-offset") for Local::now() support
  - vi.mocked() preferred over unsafe "as ReturnType<typeof vi.fn>" cast
metrics:
  duration: "~45 minutes"
  completed: 2026-05-15
  tasks_completed: 2
  files_modified: 8
---

# Phase 4 Plan 06: Tray Icon + Auto-Updater Summary

System tray wired via TrayIconBuilder in setup() with Start All / Stop All / Recent Projects (placeholder) / Quit menu; close-to-tray reads AtomicBool from AppSettings in sync on_window_event handler; tauri-plugin-updater + tauri-plugin-autostart registered; checkForUpdate() composable shows persistent Sonner toast with Install action on update available.

## Tasks Completed

| # | Name | Commit | Key Files |
|---|------|--------|-----------|
| 1 | Tray icon + plugins in lib.rs | `509b312` | lib.rs, tauri.conf.json, capabilities/default.json, Cargo.toml |
| 2 | useUpdater composable + startup check | `ecf1179` | useUpdater.ts, useUpdater.spec.ts, main.ts |

## Verification

- `cargo clippy -- -D warnings`: clean (0 errors)
- `cargo fmt --check`: clean
- `pnpm test`: 21 passed, 14 todo (7 test files, 4 skipped)
- `pnpm type-check`: clean

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] `menu_on_left_click` deprecated in Tauri 2.x**
- **Found during:** Task 1 (cargo clippy)
- **Issue:** `TrayIconBuilder::menu_on_left_click()` deprecated; clippy error with `-D warnings`
- **Fix:** Renamed to `show_menu_on_left_click(false)` per deprecation hint
- **Files modified:** src-tauri/src/lib.rs
- **Commit:** 509b312

**2. [Rule 3 - Blocking] chrono `local-offset` feature doesn't exist in 0.4.44**
- **Found during:** Task 1 (cargo clippy)
- **Issue:** Linter/auto-fixer attempted `local-offset` feature; valid feature for `Local::now()` is `clock`
- **Fix:** Correct chrono feature is `clock` (enables `Local` struct)
- **Files modified:** src-tauri/Cargo.toml
- **Commit:** 509b312

**3. [Rule 1 - Bug] Pre-existing bollard 0.19 API drift in images.rs + volumes.rs**
- **Found during:** Task 1 (cargo clippy — blocked compilation)
- **Issue:** `bollard::image::{CreateImageOptions, ...}` deprecated in 0.19; fields `id`, `repo_tags`, `size` changed from `Option<T>` to `T`; `space_reclaimed` is `i64` not `Option<i64>`
- **Fix:** images.rs migrated to `CreateImageOptionsBuilder`; volumes.rs got `#![allow(deprecated)]` + `.into_iter().flatten()` replacing `.unwrap_or_default()`; prune results cast via `.max(0) as u64`
- **Files modified:** src-tauri/src/adapters/docker/images.rs, volumes.rs
- **Commit:** 509b312

**4. [Rule 1 - Bug] Pre-existing `getComposeDiff` missing from bindings.ts**
- **Found during:** Task 2 (pnpm type-check — blocked)
- **Issue:** `ConfigHistory.vue` called `commands.getComposeDiff(...)` but binding not exported; also called with object `{projectId}` instead of positional string
- **Fix:** Added `getComposeDiff` entry to bindings.ts; fixed ConfigHistory.vue call site
- **Files modified:** src/ipc/bindings.ts, src/components/infra/ConfigHistory.vue
- **Commit:** ecf1179 (bindings included via linter before commit)

**5. [Rule 3 - Blocking] Vitest TDZ hoisting in useUpdater.spec.ts**
- **Found during:** Task 2 (pnpm test — RED phase)
- **Issue:** `vi.mock` factory referenced outer `const mockCheck` which hadn't initialized yet (hoisted before `const`)
- **Fix:** Moved to `vi.mocked(check)` pattern — import mocked module then wrap with `vi.mocked()`
- **Files modified:** src/composables/__tests__/useUpdater.spec.ts
- **Commit:** ecf1179

**6. [Rule 3 - Blocking] `@tauri-apps/plugin-process` not installed**
- **Found during:** Task 2 (pnpm type-check)
- **Issue:** `useUpdater.ts` imports `relaunch` from `@tauri-apps/plugin-process` but package missing
- **Fix:** `pnpm add @tauri-apps/plugin-process`
- **Files modified:** package.json, pnpm-lock.yaml
- **Commit:** ecf1179

## Self-Check: PASSED
