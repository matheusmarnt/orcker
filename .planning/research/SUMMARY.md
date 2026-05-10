# Research Summary — Orcker

**Synthesized from:** STACK.md · FEATURES.md · ARCHITECTURE.md · PITFALLS.md
**Confidence:** MEDIUM-HIGH overall

---

## Executive Summary

Orcker is a niche-but-defensible desktop Docker manager for Laravel developers. No competitor combines global shared services, Laravel-aware scaffolding, and a click-to-run GUI. The stack (Tauri 2 + Rust + Vue 3 + bollard) is already scaffolded and well-chosen. The primary risk is the solo dev's Rust learning curve — not technology uncertainty. The four-layer Rust architecture (`adapters` → `core` → `commands` → UI) with `Channel<T>` streaming must be established in Phase A before any feature work; retrofitting backpressure or error patterns across 8 modules is expensive.

---

## Competitive Position

No direct competitor:
- **Docker Desktop / Portainer / Lazydocker** — generic, not Laravel-aware
- **Laravel Herd / Sail** — dev env, not Docker infrastructure GUI
- **Takeout** — closest spirit, CLI only, no GUI

Moats:
- M1 Global Stack (shared `orcker-global` network with toggleable catalog)
- Visual editors: php.ini (M2.07), Supervisor (M2.08), Xdebug + IDE config gen (M2.09)
- M8 Sandbox — bundled cloudflared + bcrypt + mandatory expiry + GeoIP log

---

## Stack Decisions (validated)

| Layer | Choice | Confidence |
|-------|--------|------------|
| Desktop | Tauri 2 | HIGH |
| Docker API | bollard (async, Arc-safe) | HIGH |
| Frontend | Vue 3 + Pinia + vue-router | HIGH |
| UI primitives | shadcn-vue + reka-ui (verify name at install) | MEDIUM |
| CSS | Tailwind v3 (pin — v4 not ready for ecosystem) | HIGH |
| IPC streaming | `Channel<T>` (not `emit`) | HIGH |
| Type bridge | `tauri-specta` (verify version) | MEDIUM |
| Terminal | `@xterm/xterm` (new package name) | HIGH |
| Error handling | `thiserror` in adapters/core; `AppError: Serialize` at commands boundary | HIGH |

Key patterns:
- `bollard::Docker` is `Arc`-wrapped internally — clone handle into spawned tasks
- Use `tokio::sync::Mutex` (NOT `std::sync::Mutex`) in `tauri::State`
- Pipe bollard `Stream<LogOutput>` directly into `Channel<T>` — do NOT collect to String
- Shell out to `docker compose` for Compose orchestration (bollard doesn't implement Compose)

---

## Architecture (4-layer mandatory)

```
UI (Vue 3 + Pinia)
  ↕ Tauri IPC (Channel<T> for streams, invoke for one-shot)
Commands layer (src-tauri/src/commands/) — #[tauri::command] + AppState
  ↕
Core layer (src-tauri/src/core/) — business logic, no Tauri deps
  ↕
Adapters layer (src-tauri/src/adapters/) — Docker, FS, Shell, Git, Keychain
```

Container state: event-sourced cache (`docker events()` → `DashMap<ContainerId, State>`) beats polling — meets < 2% idle CPU target.

---

## Suggested Phase Structure (7 phases)

| Phase | Focus | Key deliverable |
|-------|-------|----------------|
| A | Async Foundation | Docker socket detection, AppState, first typed IPC, tauri-specta, tracing |
| B | Container Observability | Event-sourced cache, Pinia store, minimal container list UI |
| C | Persistent Config + Projects | `.orcker.json`, schemaVersion, migration skeleton |
| D | M1 Global Stack | `orcker-global` network, Redis + PG + Mailpit toggles — **proves thesis** |
| E | M2 + M3 + M4 | Full daily driver: project wizard, quick actions, log viewer |
| F | M5 + M6 + M7 | Compose editor, testing DB, settings, auto-updater |
| G | M8 Sandbox | cloudflared, bcrypt, mandatory expiry, GeoIP — **last, security-critical** |

---

## Critical Pitfalls

**P-C1 (CRITICAL):** Never block Tauri main thread — use `tokio::spawn` + `Channel<T>` always.

**P-C2 (CRITICAL):** Docker socket path varies: `/var/run/docker.sock` (Linux), `~/.docker/run/docker.sock` (Docker Desktop macOS), OrbStack, Colima, Windows named pipe. Auto-detect cascade required in Phase A.

**P-C5 (CRITICAL):** Linux `docker` group permission error → translate to actionable UI error, not panic.

**P-C7 (CRITICAL):** Log streaming (M4) needs cancellation tokens, throttled batch emits, virtualized 5000-line buffer.

**P-M2:** `release-please-config.json` must keep `extra-files` for `Cargo.toml` + `tauri.conf.json` version sync.

**P-M3:** Tauri updater Ed25519 key loss is unrecoverable — back up private key on Day 1.

---

## Research Flags (open items)

| Item | When | Priority |
|------|------|----------|
| Verify `reka-ui` vs `radix-vue` peer dep name at `shadcn-vue init` | Phase A | HIGH |
| Verify `tauri-specta` current version + Tauri 2 minor compatibility | Phase A | HIGH |
| Verify bollard 0.17 vs 0.18 latest | Phase A | MEDIUM |
| Benchmark xterm.js + Channel<T> at high log volume | Phase E | HIGH |
| Compose v1 hyphen-binary deprecation status | Phase E | MEDIUM |
| cloudflared binary signing per OS (macOS notarization, Windows Authenticode) | Phase G gate | HIGH |
| GeoIP database choice: MaxMind GeoLite2 vs IP2Location LITE vs DB-IP | Phase G gate | HIGH |
| bcrypt middleware compatibility across Cloudflare/ngrok/Expose | Phase G gate | HIGH |

---

## Anti-Features (scope guard)

Explicitly excluded: K8s, production management, cloud deploy (Forge/Vapor/Envoyer), built-in DB GUI, AI assistance, telemetry, unlimited sandbox expiry, multi-host, plugin marketplace (Phase 3+).

---

## Ready for Roadmap

All 4 research dimensions complete. Proceed to `/gsd:plan-phase` or requirements definition.
