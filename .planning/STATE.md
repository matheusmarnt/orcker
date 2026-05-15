# Project State — Orcker

*Last updated: 2026-05-15 — Completed 04-08-PLAN.md*

## Current Phase

**Phase 0 — COMPLETE.** Scaffold verified. Planning complete.
**Phase 1 — COMPLETE.** All 5 plans executed.
**Phase 2 — COMPLETE.** All 4 plans executed.
**Phase 3 — COMPLETE.** All 12 plans executed.
**Phase 4 — IN PROGRESS.** Plans 04-01 through 04-10 complete.

## Completed

- [x] Phase 0: Tauri 2 + Vue 3 + TypeScript scaffold
- [x] Phase 0: CI/CD pipeline (lint, test, build matrix)
- [x] Phase 0: ESLint flat config + Vitest + vue-tsc
- [x] Phase 0: Release Please (v0.1.0 PR open)
- [x] Phase 0: pnpm workspace + esbuild approval
- [x] Phase 0: Dependabot (npm + cargo + actions)
- [x] GSD: codebase mapped (7 docs in `.planning/codebase/`)
- [x] GSD: PROJECT.md created
- [x] GSD: Research complete (STACK, FEATURES, ARCHITECTURE, PITFALLS, SUMMARY)
- [x] GSD: REQUIREMENTS.md created (R-00 through M8 + RNF)
- [x] GSD: ROADMAP.md created (5 phases: Foundation → Sandbox)

## Active Work

- [x] Phase 1 Plan 01: Ed25519 + Cargo scaffold + 4-layer Rust — complete
- [x] Phase 1 Plan 02: Frontend deps + shadcn-vue + Vue bootstrap — complete (2026-05-10)
- [x] Phase 1 Plan 03: lib.rs wiring (specta + tracing + async probe) — complete
- [x] Phase 1 Plan 04: Capabilities + CSP hardening — complete
- [x] Phase 1 Plan 05: Vue UI (ContainerTable, DockerStatusBadge, ErrorScreen) — complete
- [x] Phase 2 Plan 01: Global Stack type contracts (ServiceId, ServiceStatus, ServiceConfig, GlobalStackState) — complete (2026-05-10)
- [x] Phase 2 Plan 02: Global Stack plugins + Switch.vue — complete (2026-05-10)
- [x] Phase 2 Plan 03: Docker network adapter + 5 Tauri commands + bindings regeneration — complete (2026-05-10)
- [x] Phase 2 Plan 04: Vue UI — GlobalStackPanel, service toggles — complete (2026-05-11)
- [x] Phase 3 Plan 01: Phase 3 planning — complete (2026-05-15)
- [x] Phase 3 Plan 02: App shell sidebar + routing + Wave 0 test scaffolds — complete (2026-05-15)
- [x] Phase 3 Plan 03: Core type contracts (projects.rs, compose.rs, artisan.rs) — complete (2026-05-15)
- [x] Phase 3 Plan 04: Project backend commands (5 Tauri commands + dialog + bindings regen) — complete (2026-05-15)
- [x] Phase 3 Plan 05: Projects frontend (useProjectsStore, ProjectCard, NewProjectModal, ProjectsView) — complete (2026-05-15)
- [x] Phase 3 Plan 05 (artisan): Docker exec streaming adapter + artisan Tauri commands + bindings regen — complete (2026-05-15)
- [x] Phase 3 Plan 06: CommandPanel + DestructiveConfirmDialog + ProjectDetailView + /projects/:id route — complete (2026-05-15)
- [x] Phase 3 Plan 07: Log stream backend (docker_log_stream + file_tail_stream + start/stop commands + bindings regen) — complete (2026-05-15)
- [x] Phase 3 Plan 08: Log viewer frontend (useLogsStore ring buffer + RecycleScroller + ANSI colors + LogsView) — complete (2026-05-15)
- [x] Phase 3 Plan 09: Xdebug config generation + toggle_vite_auto persistence — complete (2026-05-15)
- [x] Phase 3 Plan 10: .env editor (read_env_file + save_env_file + EnvEditor diff view) — complete (2026-05-15)
- [x] Phase 3 Plan 11: php.ini editor (read/save/parse) + Supervisor panel (list/restart workers) — complete (2026-05-15)
- [x] Phase 3 Plan 12: scaffold_project command (TALL/Inertia+Vue3/Inertia+React) + NewProjectModal streaming UI — complete (2026-05-15)
- [x] Phase 4 Plan 01: Phase 4 deps (Cargo + npm) + Wave 0 test stubs (8 frontend + 1 Rust) — complete (2026-05-15)
- [x] Phase 4 Plan 02: AppSettings Rust struct + get/save_settings commands + useSettingsStore + vue-i18n bootstrap — complete (2026-05-15)
- [x] Phase 4 Plan 03: Compose editor — read_compose_file + save_compose_file + ComposeEditor.vue Monaco drawer + ComposeErrorPanel.vue — complete (2026-05-15)
- [x] Phase 4 Plan 04: M6 database commands (create/dump/restore/open-psql) + DatabaseTab Vue component — complete (2026-05-15)
- [x] Phase 4 Plan 05: Catalog expansion — MinIO/Soketi/Meilisearch global stack + Filament/ApiOnly/Jetstream scaffold templates — complete (2026-05-15)
- [x] Phase 4 Plan 06: System tray (TrayIconBuilder + recent projects submenu) + close-to-tray + tauri-plugin-updater + autostart + checkForUpdate composable — complete (2026-05-15)
- [x] Phase 4 Plan 07: Volume and image management — bollard adapters + VolumeList + ImageList + /infra route — complete (2026-05-15)
- [x] Phase 4 Plan 08: Config versioning (git2) + Command Palette (Cmd/Ctrl+K) + ConfigHistory diff viewer — complete (2026-05-15)
- [x] Phase 4 Plan 10: Template marketplace (fetch_template_manifest + install_template) + CI bundle matrix (.deb/.AppImage/.dmg/.msi) — complete (2026-05-15)

## Phase Map

| Phase | Description | Version | Requirements |
|-------|-------------|---------|--------------|
| 0 | Scaffold + CI | baseline | R-00.* ✓ |
| 1 | Async Foundation (Docker + IPC + cache) | — | R-A.* |
| 2 | M1 Global Stack | v0.1.0 | R-M1.* |
| 3 | M2 + M3 + M4 (Daily Driver) | v0.2.0 | R-M2.*, R-M3.1–3.3, R-M4.1–4.4 |
| 4 | M5 + M6 + M7 (Power Tools) | v0.3.0 | R-M5.*, R-M6.1–6.3, R-M7.*, R-M3.4–3.5 |
| 5 | M8 Sandbox | v1.0.0 | R-M8.* |

## Key Decisions

- AppSettingsData is the serializable IPC type; AppSettings wraps it with AtomicBool for sync on_window_event use
- tray_enabled AtomicBool allows reading in Tauri on_window_event (sync context) without async lock
- show_menu_on_left_click replaces deprecated menu_on_left_click in Tauri 2.x TrayIconBuilder
- chrono feature "clock" (not "local-offset") enables Local::now() in chrono 0.4
- tray submenu uses "(no projects)" placeholder at setup() — projects list is empty at tray build time
- vi.mocked() preferred over unsafe "as ReturnType<typeof vi.fn>" cast for typed mock access
- bollard 0.19 images: fields id/repo_tags/size are non-optional; use CreateImageOptionsBuilder
- bollard 0.19 volumes: volumes field is Vec<_> (non-optional); use .into_iter().flatten()
- vue-i18n legacy:false required for <script setup> Composition API useI18n hook
- monaco-editor excluded from Vitest server.deps.external — imports window at module level which breaks Node env
- configureMonacoYaml called once at module scope via _yamlConfigured guard — prevents re-registration on remount
- save_compose_file always writes to docker-compose.yml (not .yaml) — canonical name; .yaml only used for read fallback
- ComposeEditor marker tests use pure logic assertions (no DOM mount) — avoids window-not-defined in Vitest Node env
- defineProps in ComposeErrorPanel without const assignment — avoids vue-tsc TS6133 false positive on template-only prop usage
- IniSection needs Deserialize (not just Serialize) — it is a #[tauri::command] parameter, serde requires Deserialize on all parameters
- supervisor container convention: {project_name}_supervisor_1 — derived in computed(), same pattern as app container
- AppState has no docker() method — correct pattern is app_state.docker.read().await (matches artisan.rs)
- AppError::Internal used for project-not-found in env commands — no NotFound variant; Internal is the catch-all
- String(result.error) for AppError→string coercion in Vue — AppError is a tagged union, not a primitive
- AppError::Internal used for spawn/process errors — no Io variant exists; Internal is the catch-all
- ScaffoldTemplate enum variants match TS union exactly: Tall | InertiaVue3 | InertiaReact
- run_step! macro streams stdout then stderr sequentially — avoids interleaved async read complexity
- tokio process+io-util features required for tokio::process::Command and AsyncBufReadExt
- Progress panel shown via v-if on isScaffolding — form fields hidden during active scaffold run
- ansi-to-html Convert instance created once per CommandPanel mount — not per line
- CommandPanel mounted with v-if (not v-show) — Vue teardown clears Channel listeners automatically
- logs cancel_tokens key format: "logs:{project_id}" — avoids collision with plain "{project_id}" artisan keys
- AppError::NotFound added to error enum — required by read_env_file/save_env_file and new xdebug/vite commands
- AppError::Internal for file I/O in log_stream — no Io variant; Internal is the correct catch-all
- start_log_stream blocks on token.cancelled().await — keeps command alive so frontend Channel stays open until stop_log_stream fires
- DestructiveConfirmDialog uses fixed-position Card overlay (shadcn Dialog not installed)
- Container name convention: {project_name}_app_1 — Docker Compose default
- ProjectCard Terminal button uses router.push directly — no parent emit delegation
- docker_exec_stream checks token.is_cancelled() before create_exec — pre-cancellation exits without touching Docker
- CancellationToken removed from ProjectsState after run_artisan_command returns — prevents token leak across invocations
- AppError::DockerApi used for bollard exec errors — matches existing Phase 2 error variant naming
- Local ProjectStatus type in ProjectCard — not in bindings.ts (backend doesn't emit project status events yet)
- vi.mock factory inlines data to avoid Vitest TDZ hoisting errors when mock references outer const
- Overlay modal with Card instead of shadcn Dialog — Dialog component not installed in this project
- detect_compose_driver() called synchronously in setup() — ComposeDriver ready before first frontend IPC call
- pick_project_folder uses tokio oneshot channel to bridge dialog callback into async Tauri command
- uuid added to [dependencies] (not dev-dep) — register_project generates IDs at runtime in production
- tokio-util `rt` feature (not `sync`) gates `CancellationToken` — plan had wrong feature name
- `artisan_commands()` as fn not const — Vec<T> with heap Strings cannot be Rust const
- `ComposeDriver::None` retained — callers handle missing Docker Compose without panic
- AppSidebar uses `route.path === item.to` for exact active matching — avoids `/` matching all routes
- `<Sonner>` kept in App.vue outside AppShell — overlays full viewport across all routes
- Dashboard route remains `/dashboard`; `'/'` redirects — DashboardView untouched
- `ServiceStatus` uses `serde(tag="kind", content="message")` — discriminated union safe for TS IPC
- `ServiceId` derives `Copy` — usable as HashMap key without explicit clone
- `GlobalStackState` implements `Default` via `new()` — aligns with Tauri `manage()` pattern
- bollard 0.19: `None::<bollard::container::InspectContainerOptions>` required — no turbofish on method
- specta bindings export at app runtime (cargo run), not at cargo build
- `ServiceStatusEvent` wrapper struct for emit payload (ServiceId + ServiceStatus)
- `typedError` pattern in bindings.ts: store actions check `result.status === 'ok'` before consuming data
- `Switch` component: import from `@/components/ui/switch`, not `@/components/ui/badge`
- `initial-version: "0.1.0"` in release-please-config.json — prevents 1.0.0 proposal from scaffold commits
- `bump-minor-pre-major: true` — feat commits bump minor (not major) pre-1.0.0
- v0.0.0 GitHub Release created manually as anchor at `c15bce69`
- `tauri-specta` chosen for Rust↔TS type bridge — verify version at Phase 1 start
- `Channel<T>` for IPC streams, `invoke` for one-shot commands
- `docker compose` shell-out for Compose orchestration (bollard doesn't implement Compose)
- shadcn-vue 2.x uses Tailwind v4 CSS-based config (not v3 JS config) — @tailwindcss/vite plugin
- `toast` deprecated in shadcn-vue 2.x — use `sonner` component instead
- Hash history router mandatory for Tauri webview (no server-side routing)
- vue-demi build script must be in pnpm.onlyBuiltDependencies to avoid ERR_PNPM_IGNORED_BUILDS
- RecycleScroller ref typed as `{ $el: HTMLElement }` — `InstanceType<typeof RecycleScroller>` fails vue-tsc generic constraint
- appContainerName derived from project.name in LogsView — ProjectConfig has no app_container field; `{name}_app_1` convention used
- git2 feature name is `vendored-libgit2`, not `vendored` — plan had wrong name; auto-fixed during 04-01
- `database.rs` declared in `commands/mod.rs` but NOT registered in `lib.rs` — prevents `todo!()` runtime panics; register in 04-04-PLAN when implemented
- `ServiceId::command()` returns `Option<Vec<&'static str>>` — bollard `Config.cmd` expects `Vec<&str>` not a slice; `.as_deref()` produces wrong type
- `tsconfig.json` needs explicit `paths` entry for `monaco-editor` — bundler moduleResolution alone doesn't resolve ESM exports for vue-tsc
- `GlobalStackView` uses `grid-cols-3` for 6 services — flex row breaks layout at 6 cards
- `spawn_blocking` for pg_dump/psql dump/restore — blocking shell-outs must not block tokio async runtime
- `create_testing_db_inner` fire-and-forget — testing DB creation failure must not block project registration
- `find_global_postgres` by Docker Compose labels — resilient to container name changes
- `open_db_cli` delegates to `docker_exec_stream` adapter — reuses existing streaming infrastructure
- bollard 0.19 `ImageSummary` has non-Option `id`/`repo_tags`/`size` fields — plan assumed Option; access directly
- `space_reclaimed` is `i64` (not `u64`) in bollard 0.19 prune responses — cast via `.max(0) as u64`
- `#![allow(deprecated)]` per module for `CreateImageOptions` — consistent with project bollard 0.19 pattern
- `chrono` feature `local-offset` does not exist in 0.4; correct name is `clock`
- `useInfraStore` calls `commands.listVolumes/listImages` (typed commands object, not raw typedError) — cleaner API
- `git2::DiffFormat::Patch` is the correct unified diff variant in git2 0.20 (not `Unified`)
- `space_reclaimed` is `Option<i64>` in bollard PruneImagesResponse/PruneVolumesResponse — requires `.unwrap_or(0).max(0) as u64`
- `pendingCommand` ref in `useCommandPaletteStore` acts as lightweight event bus — avoids global emitter for palette→CommandPanel routing
- `useCommandPaletteStore` placed in `src/composables/` (not `src/stores/`) per plan spec
- Template manifest fetched from Rust (reqwest) not frontend — CSP blocks external HTTPS from webview
- `save_file` dialog callback returns `Option<FilePath>` (not `Option<Option<PathBuf>>`); convert via `PathBuf::from(path.to_string())`
- CI build matrix explicit `--bundles` args: deb,appimage (ubuntu); dmg (macos); msi (windows)

## Risks

| Risk | Phase | Mitigation |
|------|-------|-----------|
| R01: Rust learning curve | 1+ | Pair-with-AI inline; adapters scoped narrowly |
| R05: Scope creep | all | YAGNI; slices fixed; REQUIREMENTS.md is truth |
| P-C2: Docker socket paths | 1 | Auto-detect cascade in Phase 1 (R-A.1) |
| P-M3: Tauri updater key loss | 1 | Generate + back up Ed25519 key Day 1 of Phase 1 |
| M8 security | 5 | Dedicated pre-phase research gate on cloudflared/GeoIP/bcrypt |

## Open Research Flags

- Verify `reka-ui` vs `radix-vue` peer dep at `shadcn-vue init` (Phase 1)
- Verify `tauri-specta` version + Tauri 2 minor compatibility (Phase 1)
- Benchmark xterm.js + Channel<T> at high log volume (Phase 3)
- cloudflared binary signing per OS — hard gate before Phase 5
- GeoIP database choice: MaxMind vs IP2Location LITE vs DB-IP (Phase 5)
