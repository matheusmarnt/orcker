---
phase: 04-power-tools
plan: "02"
subsystem: m7-settings
tags: [rust, vue, i18n, settings, tauri-plugin-store, vueuse]
dependency_graph:
  requires: ["04-01"]
  provides: ["AppSettings Rust struct", "get_settings/save_settings commands", "useSettingsStore", "vue-i18n instance"]
  affects: ["04-03", "04-05", "04-06"]
tech_stack:
  added: [vue-i18n@9, monaco-editor@0.55.1]
  patterns: [tauri-plugin-store for settings persistence, AtomicBool for sync window event handler, useColorMode from @vueuse/core, createI18n with legacy:false]
key_files:
  created:
    - src-tauri/src/core/settings.rs
    - src-tauri/src/commands/settings.rs
    - src/i18n/index.ts
    - src/i18n/en.json
    - src/i18n/pt-BR.json
    - src/stores/useSettingsStore.ts
    - src/stores/__tests__/useSettingsStore.spec.ts
    - src/i18n/__tests__/i18n.spec.ts
  modified:
    - src-tauri/src/core/mod.rs
    - src-tauri/src/commands/mod.rs
    - src-tauri/src/lib.rs
    - src/main.ts
    - src/ipc/bindings.ts
    - src/components/projects/ProjectCard.vue
    - src/components/compose/ComposeEditor.vue
    - src/components/compose/ComposeErrorPanel.vue
    - vite.config.ts
decisions:
  - "AppSettingsData is the serializable IPC type; AppSettings wraps it with AtomicBool for sync on_window_event use"
  - "tray_enabled AtomicBool allows reading in Tauri on_window_event (sync context) without async lock"
  - "vue-i18n legacy:false required for <script setup> Composition API useI18n hook"
  - "monaco-editor excluded from Vitest server.deps.external — imports window at module level which breaks Node env"
  - "DatabaseTab stub created to resolve linter-added import in ProjectCard before 04-04 implements it"
  - "ComposeErrorPanel uses bare defineProps (no const assignment) to avoid vue-tsc TS6133 false positive"
metrics:
  duration: "~25 min"
  completed: "2026-05-15"
  tasks_completed: 2
  files_created: 8
  files_modified: 9
---

# Phase 4 Plan 02: Settings Core Layer Summary

AppSettings Rust struct with AtomicBool tray_enabled + tauri-plugin-store persistence; get_settings/save_settings commands wired in lib.rs with on_window_event close-to-tray; useSettingsStore with theme/locale/tray/socket; vue-i18n bootstrapped with en + pt-BR.

## Tasks Completed

| Task | Name | Commit | Key Files |
|------|------|--------|-----------|
| 1 | AppSettings Rust struct + settings commands | 67ace2c | core/settings.rs, commands/settings.rs, lib.rs |
| 2 | useSettingsStore + vue-i18n bootstrap | 91ad6d4 | useSettingsStore.ts, i18n/index.ts, en.json, pt-BR.json, main.ts |

## Verification Results

- `cargo clippy -- -D warnings`: PASS (46 tests, 0 warnings)
- `cargo fmt --check`: PASS
- `cargo test`: 46 passed
- `pnpm type-check`: PASS
- `pnpm exec vitest run`: 19 passed, 16 todo (other plan stubs), 0 failed

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] DatabaseTab import in ProjectCard.vue blocked type-check**
- **Found during:** Task 2 type-check
- **Issue:** Linter from 04-01 added `import DatabaseTab from '@/components/database/DatabaseTab.vue'` to ProjectCard.vue; file didn't exist, breaking vue-tsc
- **Fix:** Created minimal DatabaseTab.vue stub (placeholder UI) so the import resolves; full implementation deferred to 04-04
- **Files modified:** src/components/database/DatabaseTab.vue

**2. [Rule 3 - Blocking] monaco-editor missing as direct dependency**
- **Found during:** Task 2 type-check
- **Issue:** ComposeEditor.vue imports `monaco-editor` directly but it was only an indirect dep via monaco-editor-vue3; vue-tsc couldn't resolve types
- **Fix:** Added `monaco-editor@0.55.1` as direct dependency
- **Files modified:** package.json, pnpm-lock.yaml

**3. [Rule 3 - Blocking] monaco-editor crashes Vitest with "window is not defined"**
- **Found during:** Task 2 test run
- **Issue:** monaco-editor 0.55.1 imports browser globals at module load time; Node test env has no window
- **Fix:** Added `server.deps.external: ['monaco-editor', /monaco-editor\/.*/]` to vite.config.ts test section
- **Files modified:** vite.config.ts

**4. [Rule 1 - Bug] ComposeEditor.vue implicit `any` on map callback parameter**
- **Found during:** Task 2 type-check
- **Issue:** `raw.map((m) => ...)` — `m` had implicit `any` type under strict TS
- **Fix:** Typed as `m: monaco.editor.IMarker`
- **Files modified:** src/components/compose/ComposeEditor.vue

**5. [Rule 1 - Bug] ComposeErrorPanel.vue vue-tsc TS6133 false positive on `const props`**
- **Found during:** Task 2 type-check
- **Issue:** Assigning `const props = defineProps(...)` triggers "declared but never read" because vue-tsc doesn't count template usage
- **Fix:** Removed `const props =` assignment; template props still accessible
- **Files modified:** src/components/compose/ComposeErrorPanel.vue

**6. [Rule 1 - Bug] ProjectCard.vue missing `showDatabaseTab` ref**
- **Found during:** Task 2 type-check
- **Issue:** Template referenced `showDatabaseTab` toggle added by linter but not declared in script
- **Fix:** Added `const showDatabaseTab = ref(false)` to script setup
- **Files modified:** src/components/projects/ProjectCard.vue

## Self-Check: PASSED

- FOUND: src-tauri/src/core/settings.rs
- FOUND: src-tauri/src/commands/settings.rs
- FOUND: src/i18n/index.ts
- FOUND: src/stores/useSettingsStore.ts
- FOUND: commit 67ace2c (Task 1)
- FOUND: commit 91ad6d4 (Task 2)
