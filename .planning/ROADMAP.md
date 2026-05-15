# Roadmap: Orcker

## Overview

Orcker goes from a scaffolded Tauri 2 shell (Phase 0, done) to a production-grade desktop Docker manager for Laravel developers. Phase 1 lays the async Rust foundation and container observability infrastructure so no patterns need retrofitting later. Phase 2 proves the core thesis — global shared services working end-to-end — and ships v0.1.0. Phase 3 makes Orcker a daily driver with project management, quick actions, and logs. Phase 4 adds power-user infrastructure tools, database workflows, and polished settings. Phase 5 delivers the security-critical Sandbox module last, when all dependencies are stable. Each phase has a coherent, verifiable capability at completion.

## Version Milestones

| Version | Phase gate | Meaning |
|---------|-----------|---------|
| v0.0.0 | Phase 0 (done) | Scaffold baseline, CI green |
| v0.1.0 | Phase 2 complete | Thesis proven: global Docker services toggle from GUI |
| v0.2.0 | Phase 3 complete | Daily driver: projects + quick actions + logs |
| v1.0.0 | Phase 5 complete | Full feature set including Sandbox |

## Phases

- [ ] **Phase 1: Foundation** - Async Rust infrastructure, Docker connection, typed IPC, container observability cache
- [ ] **Phase 2: Global Stack** - M1 — `orcker-global` network + Redis/PG/Mailpit toggles; proves the thesis; ships v0.1.0
- [ ] **Phase 3: Daily Driver** - M2 + M3 + M4 — project wizard, quick actions, unified log viewer; ships v0.2.0
- [ ] **Phase 4: Power Tools** - M5 + M6 + M7 — Compose editor, database workflows, settings, auto-updater
- [ ] **Phase 5: Sandbox** - M8 — cloudflared bundle, bcrypt, mandatory expiry, GeoIP log; ships v1.0.0

## Phase Details

### Phase 1: Foundation
**Goal**: The async Rust architecture is established end-to-end so all subsequent feature work builds on validated patterns — no retrofitting.
**Depends on**: Phase 0 (scaffold complete)
**Requirements**: R-A.1, R-A.2, R-A.3, R-A.4, R-A.5, R-A.6, R-A.7, R-A.8, RNF-01, RNF-02, RNF-03
**Success Criteria** (what must be TRUE):
  1. The app connects to Docker (auto-detects socket on Linux/macOS/Windows) and displays the Docker version in the UI without any manual config
  2. A live container list renders in the UI and updates within 2 seconds of a container starting or stopping (event-sourced cache, not polling)
  3. A Rust error (e.g. Docker not running) surfaces as a readable message in the UI — not a crash, not a raw Rust string
  4. TypeScript types for all IPC commands are generated automatically from Rust types (tauri-specta) — no hand-written type files
  5. The app cold-starts in under 2 seconds; RAM at idle is under 150 MB
**Plans**: 5 plans

Plans:
- [ ] 01-01-PLAN.md — Ed25519 keypair + Cargo.toml deps + four-layer Rust module scaffold (AppError, AppState, DockerAdapter)
- [ ] 01-02-PLAN.md — Frontend deps + shadcn-vue init + Vue app bootstrap (Pinia, Router, layout shell, IPC wrappers)
- [ ] 01-03-PLAN.md — lib.rs full wiring: tauri-specta Builder, tracing + LogTracer, async Docker probe, bindings.ts codegen
- [ ] 01-04-PLAN.md — Tauri capabilities hardening + CSP + updater pubkey in tauri.conf.json
- [ ] 01-05-PLAN.md — Vue UI: ContainerTable, DockerStatusBadge, ErrorScreen, skeleton startup, Pinia event wiring

**Risk annotations:**
- P-C1: All Docker calls must use `tokio::spawn` + `Channel<T>` — never block main thread
- P-C2: Socket auto-detect cascade is required before any other work
- P-M3: Ed25519 updater key must be generated and backed up in this phase
- Research flag: verify `tauri-specta` version + `reka-ui`/`radix-vue` peer dep name before installing

---

### Phase 2: Global Stack
**Goal**: A Laravel developer can toggle shared Docker services (Redis, PostgreSQL, Mailpit) on and off from the GUI without any CLI — proving Orcker's core thesis.
**Depends on**: Phase 1
**Requirements**: R-M1.1, R-M1.2, R-M1.3, R-M1.4, R-M1.5
**Version milestone**: v0.1.0 ships at completion
**Success Criteria** (what must be TRUE):
  1. User can start and stop the `orcker-global` Docker network from the GUI — the network appears/disappears in `docker network ls`
  2. User can toggle Redis, PostgreSQL, and Mailpit independently; each shows a live status badge (running / stopped / unhealthy) within 2 seconds
  3. User can configure per-service options (port, version, env vars) via the config panel and the change persists across app restarts
  4. User can start all services at once with the Global ON shortcut (`Cmd/Ctrl+Shift+G`) and stop them with Global OFF
**Plans**: 5 plans

Plans:
- [ ] 02-01-PLAN.md — Rust type contracts: ServiceId, ServiceConfig, ServiceStatus, GlobalStackState + TDD unit tests
- [ ] 02-02-PLAN.md — Plugin deps: tauri-plugin-global-shortcut + tauri-plugin-store + shadcn-vue Switch; wire into lib.rs + capabilities
- [ ] 02-03-PLAN.md — Rust backend: Docker network adapter + 5 Tauri commands (toggle, status, config, global_on, global_off) + bindings.ts regen
- [ ] 02-04-PLAN.md — Vue frontend: useGlobalStackStore + ServiceCard + ServiceConfigPanel + GlobalStackView + /global route + nav
- [ ] 02-05-PLAN.md — Human visual checkpoint: end-to-end UX verification on live Docker

**Risk annotations:**
- R05 (scope creep): MinIO, Soketi, Meilisearch are Phase F scope — do not implement here
- Research flag: verify bollard 0.17 vs 0.18 latest before writing Docker network commands

---

### Phase 3: Daily Driver
**Goal**: A developer can register a Laravel project, run common artisan commands, and read all relevant logs — all from the GUI, making Orcker the primary interface for daily development.
**Depends on**: Phase 2
**Requirements**: R-M2.1, R-M2.2, R-M2.3, R-M2.4, R-M2.5, R-M2.6, R-M2.7, R-M2.8, R-M2.9, R-M2.10, R-M3.1, R-M3.2, R-M3.3, R-M4.1, R-M4.2, R-M4.3, R-M4.4, RNF-04, RNF-05
**Version milestone**: v0.2.0 ships at completion
**Success Criteria** (what must be TRUE):
  1. User can register a new project via the GUI form (file picker), or import an existing project by pointing at a directory containing `artisan` — the project appears in the project list
  2. User can scaffold a TALL Stack project from a template and have it running (containers up, services attached to `orcker-global`) within 3 minutes from zero
  3. User can run `migrate`, `migrate:fresh --seed` (with a confirmation dialog), and `*:clear` commands from the GUI; output streams live to the UI
  4. User can view a unified log stream (docker logs + laravel.log + Nginx + Supervisor) for a project, scroll 5000 lines without UI freeze, and filter by log level or keyword
  5. User can configure Xdebug and have a working `.vscode/launch.json` generated automatically
**Plans**: 13 plans

Plans:
- [ ] 03-01-PLAN.md — Rust type contracts: ProjectConfig, ComposeDriver, ArtisanCommand catalog + Cargo deps
- [ ] 03-02-PLAN.md — Navigation refactor: AppShell, collapsible sidebar, route stubs, Wave 0 test scaffolds
- [ ] 03-03-PLAN.md — Project CRUD Rust backend: 5 Tauri commands + capabilities + bindings regen
- [ ] 03-04-PLAN.md — Projects Vue frontend: useProjectsStore, ProjectCard grid, NewProjectModal (Import + Scaffold tabs)
- [ ] 03-05-PLAN.md — Artisan exec Rust backend: docker exec streaming, CancellationToken, 3 commands
- [ ] 03-06-PLAN.md — CommandPanel Vue: ANSI output, cancel button, DestructiveConfirmDialog, ProjectDetailView
- [ ] 03-07-PLAN.md — Log stream Rust backend: docker logs + file tail, ring buffer, 2 commands
- [ ] 03-08-PLAN.md — LogViewer Vue: useLogsStore ring buffer + filter, RecycleScroller, ansi-to-html
- [ ] 03-09-PLAN.md — Xdebug config generation (.vscode/launch.json port 9003) + Vite toggle persistence
- [ ] 03-10-PLAN.md — .env editor: read/parse/save + diff vs .env.example (R-M2.7)
- [ ] 03-11-PLAN.md — php.ini editor + Supervisor panel (R-M2.8, R-M2.9) — optional, can slip to Phase 4
- [ ] 03-12-PLAN.md — Scaffold templates: TALL, Inertia+Vue3, Inertia+React with Channel<T> progress streaming
- [ ] 03-13-PLAN.md — Human visual verification: full daily driver flow end-to-end

**Risk annotations:**
- P-C7: Log streaming cancellation tokens must be implemented — unbounded streams will OOM
- Research flag: benchmark xterm.js + Channel<T> at high log volume before committing to virtualisation approach
- R-M2.4: `docker compose` plugin vs `docker-compose` binary detection must happen at runtime, not hardcoded
- R-M3.5 (Command Palette, Favorites): deferred to Phase 4 per requirements

---

### Phase 4: Power Tools
**Goal**: Power users can manage Docker infrastructure, database workflows, and app settings — including auto-updates — making Orcker complete for advanced daily use.
**Depends on**: Phase 3
**Requirements**: R-M5.1, R-M5.2, R-M5.3, R-M5.4, R-M5.5, R-M6.1, R-M6.2, R-M6.3, R-M7.1, R-M7.2, R-M7.3, R-M7.4, R-M7.5, R-M7.6, R-M3.4, R-M3.5, R-M1.2 (MinIO/Soketi/Meilisearch catalog expansion), R-M4.5, R-M4.6, R-M2.3 (Filament/API/Jetstream templates), RNF-05, RNF-06
**Success Criteria** (what must be TRUE):
  1. User can open, edit, and save a `docker-compose.yml` file using the in-app editor with YAML validation; invalid YAML is highlighted before saving
  2. User can see a `{project}_testing` database auto-created in the global PostgreSQL instance when registering or scaffolding a project
  3. User can switch between dark, light, and system themes; the preference persists across restarts
  4. User can enable the system tray icon and use it to Start All / Stop All services without opening the main window
  5. User receives an in-app update prompt when a new stable release is available and can install it without leaving the app
**Plans**: 11 plans

Plans:
- [ ] 04-01-PLAN.md — Cargo/npm deps install + Wave 0 test stubs (Nyquist compliance)
- [ ] 04-02-PLAN.md — Settings core: AppSettings Rust struct + useSettingsStore + vue-i18n bootstrap
- [ ] 04-03-PLAN.md — Compose editor: read/save commands + Monaco YAML drawer on ProjectCard
- [ ] 04-04-PLAN.md — Database backend: auto-create testing DB + dump/restore + CLI terminal
- [ ] 04-05-PLAN.md — Catalog expansion: MinIO/Soketi/Meilisearch + Filament/API/Jetstream templates
- [ ] 04-06-PLAN.md — Tray + updater + autostart + close-to-tray logic
- [ ] 04-07-PLAN.md — Volume + image management (bollard adapters + Vue lists in /infra route)
- [ ] 04-08-PLAN.md — Config versioning (git2) + Command Palette + Favorites/history
- [ ] 04-09-PLAN.md — Settings modal UI (4 sections) + resource graphs (Chart.js) + OS notifications
- [ ] 04-10-PLAN.md — Template marketplace Vue UI + CI multi-platform build matrix (RNF-06)
- [ ] 04-11-PLAN.md — Human visual verification checkpoint

**Risk annotations:**
- R05 (scope creep): Template marketplace (R-M5.5) is included here but should be cut if it risks delay
- R-M4.5/R-M4.6 (resource graphs, notifications): included in this phase's expanded catalog — defer if scope pressure builds

---

### Phase 5: Sandbox
**Goal**: A developer can securely share a local Laravel project over HTTPS with an auto-expiring, password-protected, access-logged tunnel — last phase because it is security-critical and depends on all other modules being stable.
**Depends on**: Phase 4
**Requirements**: R-M8.1, R-M8.2, R-M8.3, R-M8.4, R-M8.5, R-M8.6, R-M8.7, RNF-05
**Version milestone**: v1.0.0 ships at completion
**Success Criteria** (what must be TRUE):
  1. User can activate a Sandbox tunnel for a project and receive a working HTTPS URL and QR code within 30 seconds
  2. The tunnel requires a bcrypt password and has a mandatory expiry (1h/2h/4h/8h/24h) — there is no "unlimited" option in the UI
  3. The app displays a GeoIP access log (country flag, masked IP, access count) for the active tunnel
  4. Closing the app terminates all active tunnels automatically — no orphaned cloudflared processes
  5. The bundled `cloudflared` binary passes SHA-256 verification before execution; a tampered binary is rejected with a clear error
**Plans**: TBD

**Risk annotations:**
- Gate on research flags before starting: cloudflared binary signing (macOS notarization + Windows Authenticode), GeoIP database choice (MaxMind GeoLite2 vs alternatives), bcrypt middleware compatibility across providers
- R-M8.4 (ngrok/Expose): implement Cloudflare provider first; add others only after Cloudflare is solid
- Security: never implement unlimited expiry regardless of user request — this is a hard constraint

---

## Progress

**Execution Order:** 1 → 2 → 3 → 4 → 5

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Foundation | 1/5 | In Progress|  |
| 2. Global Stack | 4/5 | In Progress|  |
| 3. Daily Driver | 12/13 | In Progress|  |
| 4. Power Tools | 6/11 | In Progress|  |
| 5. Sandbox | 0/TBD | Not started | - |

---

## Coverage Map

All v1 requirements mapped (Phase 0 requirements already done):

| Requirement | Phase |
|-------------|-------|
| R-A.1 | Phase 1 |
| R-A.2 | Phase 1 |
| R-A.3 | Phase 1 |
| R-A.4 | Phase 1 |
| R-A.5 | Phase 1 |
| R-A.6 | Phase 1 |
| R-A.7 | Phase 1 |
| R-A.8 | Phase 1 |
| R-M1.1 | Phase 2 |
| R-M1.2 (Redis/PG/Mailpit) | Phase 2 |
| R-M1.2 (MinIO/Soketi/Meilisearch) | Phase 4 |
| R-M1.3 | Phase 2 |
| R-M1.4 | Phase 2 |
| R-M1.5 | Phase 2 |
| R-M2.1 | Phase 3 |
| R-M2.2 | Phase 3 |
| R-M2.3 (TALL/Inertia) | Phase 3 |
| R-M2.3 (Filament/API/Jetstream) | Phase 4 |
| R-M2.4 | Phase 3 |
| R-M2.5 | Phase 3 |
| R-M2.6 | Phase 3 |
| R-M2.7 | Phase 3 |
| R-M2.8 | Phase 3 |
| R-M2.9 | Phase 3 |
| R-M2.10 | Phase 3 |
| R-M3.1 | Phase 3 |
| R-M3.2 | Phase 3 |
| R-M3.3 | Phase 3 |
| R-M3.4 | Phase 4 |
| R-M3.5 | Phase 4 |
| R-M4.1 | Phase 3 |
| R-M4.2 | Phase 3 |
| R-M4.3 | Phase 3 |
| R-M4.4 | Phase 3 |
| R-M4.5 | Phase 4 |
| R-M4.6 | Phase 4 |
| R-M5.1 | Phase 4 |
| R-M5.2 | Phase 4 |
| R-M5.3 | Phase 4 |
| R-M5.4 | Phase 4 |
| R-M5.5 | Phase 4 |
| R-M6.1 | Phase 4 |
| R-M6.2 | Phase 4 |
| R-M6.3 | Phase 4 |
| R-M6.4 | Out of scope (Phase G+, deferred) |
| R-M7.1 | Phase 4 |
| R-M7.2 | Phase 4 |
| R-M7.3 | Phase 4 |
| R-M7.4 | Phase 4 |
| R-M7.5 | Phase 4 |
| R-M7.6 | Phase 4 |
| R-M8.1 | Phase 5 |
| R-M8.2 | Phase 5 |
| R-M8.3 | Phase 5 |
| R-M8.4 | Phase 5 |
| R-M8.5 | Phase 5 |
| R-M8.6 | Phase 5 |
| R-M8.7 | Phase 5 |
| RNF-01 | Phase 1 |
| RNF-02 | Phase 1 |
| RNF-03 | Phase 1 |
| RNF-04 | Phase 3 |
| RNF-05 | Phase 3, 4, 5 (progressive) |
| RNF-06 | Phase 4 |
