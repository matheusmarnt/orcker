# Orcker

## What This Is

Orcker is a cross-platform desktop application (Linux, macOS, Windows) built for Laravel/TALL Stack developers using Docker. It centralizes Docker infrastructure management into a single visual interface — shared global services across all projects (Redis, PostgreSQL, Mailpit, etc.) and per-project services — eliminating repetitive CLI commands and environment inconsistencies without leaving the GUI.

Target: Solo Laravel devs (primary) and small team tech leads who need portable `.orcker.json` config for reproducible environments.

## Core Value

A Laravel dev should go from zero to a running project environment in under 3 minutes — no CLI, no Docker knowledge required, no RAM waste from duplicate services.

## Requirements

### Validated

- ✓ Tauri 2 desktop scaffold with Vue 3 + TypeScript — Phase 0
- ✓ CI/CD pipeline (GitHub Actions: lint, test, release-please, build matrix) — Phase 0
- ✓ ESLint flat config, Vitest, vue-tsc type-checking — Phase 0
- ✓ Release Please automated versioning (v0.0.0 baseline, semver pre-1.0.0) — Phase 0
- ✓ pnpm workspace with esbuild build approval — Phase 0
- ✓ Dependabot for npm, cargo, actions — Phase 0

### Active

- [ ] Rust backend adapters (Docker via bollard, filesystem, shell, git, keychain)
- [ ] Tauri commands: list/start/stop containers, get Docker version
- [ ] M1 Global Stack — Docker network `orcker-global`, service catalog (Redis, PostgreSQL, MySQL, Mailpit, MinIO, Soketi), toggle ON/OFF per service
- [ ] M2 Projects — register/import/scaffold projects (TALL, Inertia.js + Vue 3, Filament), `.env` and `php.ini` editors, Supervisor panel, Xdebug toggle
- [ ] M3 Quick Actions — artisan, migrate, composer, pest, npm via docker exec; streaming output
- [ ] M4 Logs — unified viewer for docker logs, laravel.log, Nginx, Supervisor; virtualised buffer 5000 lines; filters + export
- [ ] M5 Infra — docker-compose editor (Monaco), volume management, image management, config versioning via git2, template marketplace
- [ ] M6 Database — auto-create testing DB per project, dump/restore, CLI integration
- [ ] M7 Settings — dark/light/system theme, PT/BR + EN i18n, Docker socket path, system tray, auto-updater
- [ ] M8 Sandbox — HTTPS tunnel (cloudflare bundled + ngrok + Expose), bcrypt password, expiry, CIDR allowlist, read-only mode, GeoIP access log, QR code

### Out of Scope

- Production environment management — dev-only tool
- Cloud deploy integrations (Forge, Vapor, Envoyer) — 100% local focus
- Teams with dedicated DevOps infra (K8s, enterprise CI/CD)
- Non-Laravel developers
- Linux ARM64 at MVP — planned Phase 3

## Context

- **Developer profile:** Solo dev, 6 years PHP/TALL, learning Rust/Tauri/Vue via pair-with-AI (no formal study)
- **Learn-by-doing:** Rust/Tauri concepts explained inline during implementation — no spike phases
- **Current state:** Phase 0 scaffold complete (Tauri 2 + Vue 3 + CI green). No Rust implementation yet. No UI modules yet.
- **Git workflow:** Conventional commits (no co-authorship), feature branches per module slice, Release Please for automated changelog + release
- **Reference architecture:** PHP-FPM + Nginx + Supervisor per-project Docker stack (PRD §7, referencia-docker-laravel.md)
- **Codebase map:** `.planning/codebase/` — STACK, ARCHITECTURE, STRUCTURE, CONVENTIONS, TESTING, INTEGRATIONS, CONCERNS all documented

## Constraints

- **Tech stack:** Tauri 2 + Vue 3 + TypeScript + Rust (bollard) — locked by PRD §10; no Electron
- **Rust learning curve:** R01 risk — pair-with-AI mitigates; adapters scoped narrowly
- **Solo dev:** No team; YAGNI enforced; scope creep is highest risk (R05)
- **Security (M8):** Sandbox module must bundle cloudflared with SHA-256 verification; no "unlimited" expiry option
- **Platform:** Linux x64 MVP; macOS + Windows in Phase 0 CI matrix; ARM64 Phase 3

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Tauri 2 over Electron | Memory efficiency, native Rust backend, CSP security | — Pending validation |
| bollard crate for Docker API | Async-native, used by Podman Desktop, avoids shell-out | — Pending validation |
| Release Please (not manual tags) | Automated semver from conventional commits | ✓ Working (v0.0.0 baseline) |
| shadcn/vue + Radix Vue | Accessible primitives, no style lock-in | — Pending implementation |
| TALL/Inertia first, Filament in Phase 1+ | Reduce scaffold complexity for MVP | — Pending |
| MIT open-source from day 1 | Community adoption + contribution funnel | ✓ Done |
| `.planning/` committed to git | Planning docs version-controlled alongside code | ✓ Done |
| No Co-Authored-By AI in commits | Project ownership stays with human developer | ✓ Rule in CLAUDE.md |

---
*Last updated: 2026-05-10 after GSD initialization (Phase 0 scaffold complete)*
