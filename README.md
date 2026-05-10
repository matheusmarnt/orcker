# Orcker

**Your Laravel Dev Infrastructure, Under Control.**

[![CI](https://github.com/matheusmarnt/orcker/actions/workflows/ci.yml/badge.svg)](https://github.com/matheusmarnt/orcker/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/matheusmarnt/orcker)](https://github.com/matheusmarnt/orcker/releases)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

> **Status:** Pre-MVP — active development. Not production-ready.

---

## What is Orcker?

Orcker is a cross-platform desktop application (Linux, macOS, Windows) built for Laravel developers using the **TALL Stack** with Docker. It centralizes Docker infrastructure management into a single, specialized visual interface — shared global services across all projects and per-project services, all without touching the CLI.

**Problems it solves:**

- Multiple Redis/PostgreSQL/Mailpit instances wasting RAM across projects
- Dozens of repetitive CLI commands per dev session
- No visibility into container state without opening multiple terminals
- Hours configuring Docker from scratch for every new Laravel project

## Tech Stack

| Layer | Technology |
|---|---|
| Desktop | Tauri 2 |
| Frontend | Vue 3 + TypeScript + TailwindCSS 3 |
| UI Components | shadcn/vue + Radix Vue |
| State | Pinia |
| Backend | Rust (bollard, tokio, serde) |
| Docker Client | bollard (Rust crate) |
| Terminal | xterm.js |

## Features (Planned)

- **Global Stack** — Redis, PostgreSQL, MySQL, Mailpit, MinIO, Soketi on a shared Docker network
- **Project Management** — register, scaffold TALL/Inertia/Filament, edit `.env` / `php.ini` / Supervisor
- **Quick Actions** — artisan, composer, pest, npm via Command Palette (`Ctrl+K`)
- **Centralized Logs** — docker logs, laravel.log, Nginx, Supervisor in real time
- **M8 Sandbox** — temporary HTTPS tunnel for client demos (Cloudflare Tunnel, ngrok, Expose)

See the [full PRD](docs/PRD.md) for all requirements and module details.

## Roadmap

| Phase | Weeks | Delivery |
|---|---|---|
| **0** Foundation | 2–5 | Tauri app + Docker integration + green CI |
| **1** MVP Core | 6–13 | v0.1.0 — Global Stack + TALL/Inertia projects |
| **2** Full Feature | 14–25 | v0.2.0 — All modules + M8 Sandbox |
| **3** Polish / GA | 26–37 | v1.0.0 — ARM64, plugin system, orcker.dev |

## Local Setup

> Setup documentation will be available after Phase 0. Follow [releases](https://github.com/matheusmariano/orcker/releases) for updates.

## Contributing

Contributions are welcome. Read [CONTRIBUTING.md](CONTRIBUTING.md) before opening a PR.

Issues labeled `good first issue` are good entry points.

## License

[MIT](LICENSE) — 100% open-source, free forever.
