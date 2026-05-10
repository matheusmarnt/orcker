# Requirements — Orcker

> Derived from PRD v0.3.1 + research synthesis. Scoped for solo dev, pre-1.0.0 roadmap.

## Validated (Phase 0 — Complete)

| ID | Requirement | Status |
|----|-------------|--------|
| R-00.1 | Tauri 2 + Vue 3 + TypeScript desktop scaffold | ✓ Done |
| R-00.2 | CI/CD pipeline (GitHub Actions: lint, test, build matrix) | ✓ Done |
| R-00.3 | ESLint flat config + Vitest + vue-tsc | ✓ Done |
| R-00.4 | Release Please automated semver (pre-1.0.0 baseline) | ✓ Done |
| R-00.5 | pnpm workspace with esbuild build approval | ✓ Done |
| R-00.6 | Dependabot for npm + cargo + actions | ✓ Done |

## Foundation Layer (Phase A — Blocks Everything)

| ID | Requirement | Notes |
|----|-------------|-------|
| R-A.1 | Docker socket auto-detect cascade (Linux/macOS Desktop/OrbStack/Colima/Windows pipe) | Critical cross-platform (Pitfall P-C2) |
| R-A.2 | `AppState` with single bollard `Docker` handle (Arc-wrapped, cloned into tasks) | No `std::sync::Mutex` |
| R-A.3 | `AppError` enum with `serde::Serialize` derivation at commands boundary | Required for IPC |
| R-A.4 | First typed IPC command (`docker_get_version`) via `Channel<T>` or `invoke` | Validates stack end-to-end |
| R-A.5 | `tauri-specta` codegen wiring (Rust types → TypeScript) | Prevents type drift at 39+ RFs |
| R-A.6 | `tracing` + `tauri-plugin-log` setup | Foundation for M4 |
| R-A.7 | Tauri capabilities scoped (CSP, allowlist minimal) | RNF-03 security |
| R-A.8 | Tauri updater Ed25519 keypair generated + backed up | P-M3: unrecoverable if lost |

## M1 — Global Stack

| ID | Requirement |
|----|-------------|
| R-M1.1 | Create/manage `orcker-global` Docker network (bridge external) |
| R-M1.2 | Service catalog: Redis, PostgreSQL, Mailpit (MVP); MinIO, Soketi, Meilisearch (Phase F) |
| R-M1.3 | Toggle ON/OFF per service + status badge + healthcheck |
| R-M1.4 | Per-service config panel: version, port, env vars, volume, resources |
| R-M1.5 | Global ON/OFF/Restart + keyboard shortcut (`Cmd/Ctrl+Shift+G`) |

## M2 — Projects

| ID | Requirement |
|----|-------------|
| R-M2.1 | Register project via form + native file picker |
| R-M2.2 | Import existing project (detect `artisan`, `composer.json`, `.env`, `docker-compose.yml`) |
| R-M2.3 | Scaffold templates: TALL (MVP), Inertia+Vue3, Inertia+React; Filament v3/v4/v5 + API + Jetstream (Phase F) |
| R-M2.4 | Detect `docker compose` plugin vs `docker-compose` binary at runtime |
| R-M2.5 | Toggle Vite startup (auto/manual) per project, persist in `.orcker.json` |
| R-M2.6 | Project dashboard: status, service list, quick actions, local URLs, resource usage, logs preview |
| R-M2.7 | Visual `.env` editor with diff vs `.env.example` |
| R-M2.8 | Visual `php.ini` editor (Performance/Upload/Execution/OPcache/Extensions categories + raw fallback) |
| R-M2.9 | Supervisor panel: add/remove/edit/restart workers, view logs, write `supervisord.conf` |
| R-M2.10 | Xdebug toggle + generate `.vscode/launch.json` + PhpStorm `.idea/` config |

## M3 — Quick Actions

| ID | Requirement |
|----|-------------|
| R-M3.1 | Built-in actions: `migrate`, `migrate:fresh --seed` (with confirmation), `tinker`, `*:clear`, `npm run dev`, `pest` |
| R-M3.2 | Execute via `docker exec` in project `app` container |
| R-M3.3 | Output streaming to UI via `Channel<T>` |
| R-M3.4 | Command Palette `Cmd/Ctrl+K` with fuzzy search (Phase F) |
| R-M3.5 | Favorites + history + re-execution (Phase F) |

## M4 — Logs

| ID | Requirement |
|----|-------------|
| R-M4.1 | Unified viewer: `docker logs -f`, `storage/logs/laravel.log`, Nginx, Supervisor |
| R-M4.2 | Virtualised buffer 5000 lines max (xterm.js `scrollback` or vue-virtual-scroller) |
| R-M4.3 | Filters (level, keyword, time), search with highlight, export |
| R-M4.4 | Cancellation tokens on all streaming handles (Pitfall P-C7) |
| R-M4.5 | Resource graphs CPU/mem time series — polling 2s (Phase F) |
| R-M4.6 | OS notifications + internal notification bell (Phase F) |

## M5 — Infra

| ID | Requirement |
|----|-------------|
| R-M5.1 | docker-compose editor (Monaco or CodeMirror 6 fallback) + YAML validation |
| R-M5.2 | Volume management: list size, prune, backup |
| R-M5.3 | Image management: list, rebuild, prune, pull |
| R-M5.4 | Config versioning via `git2` crate: diff, rollback, sync-on-open |
| R-M5.5 | Template marketplace: `orcker-templates` repo, JSON manifest, browse + install in UI |

## M6 — Database

| ID | Requirement |
|----|-------------|
| R-M6.1 | Auto-create `{project}_testing` DB in global PostgreSQL/MySQL on project register/scaffold |
| R-M6.2 | Dump/restore via `pg_dump`/`mysqldump` |
| R-M6.3 | CLI integration terminal in-app |
| R-M6.4 | Laravel Telescope slow query viewer (Phase G+) |

## M7 — Settings

| ID | Requirement |
|----|-------------|
| R-M7.1 | Dark/Light/System theme |
| R-M7.2 | PT-BR + EN i18n (vue-i18n) |
| R-M7.3 | Docker socket path config, resources default |
| R-M7.4 | System tray: status indicator + Start/Stop All + recent projects |
| R-M7.5 | Auto-updater (Tauri Updater, stable + beta channels) |
| R-M7.6 | Export/Import config JSON |

## M8 — Sandbox (Phase G — Last, Security-Critical)

| ID | Requirement |
|----|-------------|
| R-M8.1 | Activate per project; pulsing badge; auto-close on app exit |
| R-M8.2 | Bundle `cloudflared` binary per OS with SHA-256 runtime verification |
| R-M8.3 | HTTPS URL display + QR code (generated locally) |
| R-M8.4 | Multi-provider: Cloudflare (bundled), ngrok (token via keyring), Expose (self-hosted config) |
| R-M8.5 | Proxy middleware: bcrypt password, mandatory expiry (1h/2h/4h/8h/24h, no unlimited), CIDR allowlist, read-only mode |
| R-M8.6 | GeoIP access log: country flag, masked IP, counters, CSV export |
| R-M8.7 | Security warning on activation; warning for sensitive `.env` data |

## Non-Functional

| ID | Requirement |
|----|-------------|
| RNF-01 | Cold start < 2s; RAM idle < 150MB |
| RNF-02 | Linux x64 MVP; macOS + Windows CI matrix; ARM64 Phase 3 |
| RNF-03 | CSP strict; no arbitrary network access from UI |
| RNF-04 | Log buffer max 5000 lines (no OOM from tail -f) |
| RNF-05 | Test coverage ≥ 60% (Phase E), ≥ 80% (Phase F), ≥ 90% (Phase G) |
| RNF-06 | Build artifacts: `.deb` + `.AppImage` (Linux), `.dmg` (macOS universal), `.msi` (Windows) |

## Out of Scope

- Production environment management
- Cloud deploy integrations (Forge, Vapor, Envoyer)
- Teams with dedicated DevOps infra (K8s, enterprise CI/CD)
- Non-Laravel developers
- Linux ARM64 at MVP
- Built-in database GUI
- AI assistance / telemetry
- Unlimited sandbox expiry

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| R-00.1 | Phase 0 | Done |
| R-00.2 | Phase 0 | Done |
| R-00.3 | Phase 0 | Done |
| R-00.4 | Phase 0 | Done |
| R-00.5 | Phase 0 | Done |
| R-00.6 | Phase 0 | Done |
| R-A.1 | Phase 1 | Pending |
| R-A.2 | Phase 1 | Pending |
| R-A.3 | Phase 1 | Pending |
| R-A.4 | Phase 1 | Pending |
| R-A.5 | Phase 1 | Pending |
| R-A.6 | Phase 1 | Pending |
| R-A.7 | Phase 1 | Pending |
| R-A.8 | Phase 1 | Pending |
| R-M1.1 | Phase 2 | Pending |
| R-M1.2 (Redis/PG/Mailpit) | Phase 2 | Pending |
| R-M1.2 (MinIO/Soketi/Meilisearch) | Phase 4 | Pending |
| R-M1.3 | Phase 2 | Pending |
| R-M1.4 | Phase 2 | Pending |
| R-M1.5 | Phase 2 | Pending |
| R-M2.1 | Phase 3 | Pending |
| R-M2.2 | Phase 3 | Pending |
| R-M2.3 (TALL/Inertia) | Phase 3 | Pending |
| R-M2.3 (Filament/API/Jetstream) | Phase 4 | Pending |
| R-M2.4 | Phase 3 | Pending |
| R-M2.5 | Phase 3 | Pending |
| R-M2.6 | Phase 3 | Pending |
| R-M2.7 | Phase 3 | Pending |
| R-M2.8 | Phase 3 | Pending |
| R-M2.9 | Phase 3 | Pending |
| R-M2.10 | Phase 3 | Pending |
| R-M3.1 | Phase 3 | Pending |
| R-M3.2 | Phase 3 | Pending |
| R-M3.3 | Phase 3 | Pending |
| R-M3.4 | Phase 4 | Pending |
| R-M3.5 | Phase 4 | Pending |
| R-M4.1 | Phase 3 | Pending |
| R-M4.2 | Phase 3 | Pending |
| R-M4.3 | Phase 3 | Pending |
| R-M4.4 | Phase 3 | Pending |
| R-M4.5 | Phase 4 | Pending |
| R-M4.6 | Phase 4 | Pending |
| R-M5.1 | Phase 4 | Pending |
| R-M5.2 | Phase 4 | Pending |
| R-M5.3 | Phase 4 | Pending |
| R-M5.4 | Phase 4 | Pending |
| R-M5.5 | Phase 4 | Pending |
| R-M6.1 | Phase 4 | Pending |
| R-M6.2 | Phase 4 | Pending |
| R-M6.3 | Phase 4 | Pending |
| R-M6.4 | Deferred (Phase G+) | Out of scope |
| R-M7.1 | Phase 4 | Pending |
| R-M7.2 | Phase 4 | Pending |
| R-M7.3 | Phase 4 | Pending |
| R-M7.4 | Phase 4 | Pending |
| R-M7.5 | Phase 4 | Pending |
| R-M7.6 | Phase 4 | Pending |
| R-M8.1 | Phase 5 | Pending |
| R-M8.2 | Phase 5 | Pending |
| R-M8.3 | Phase 5 | Pending |
| R-M8.4 | Phase 5 | Pending |
| R-M8.5 | Phase 5 | Pending |
| R-M8.6 | Phase 5 | Pending |
| R-M8.7 | Phase 5 | Pending |
| RNF-01 | Phase 1 | Pending |
| RNF-02 | Phase 1 | Pending |
| RNF-03 | Phase 1 | Pending |
| RNF-04 | Phase 3 | Pending |
| RNF-05 | Phase 3/4/5 (progressive) | Pending |
| RNF-06 | Phase 4 | Pending |
