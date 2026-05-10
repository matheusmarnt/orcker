# Contributing to Orcker

Thank you for your interest in contributing! This guide covers everything you need to get started.

## Code of Conduct

This project follows the [Contributor Covenant](CODE_OF_CONDUCT.md). By participating, you agree to uphold it.

## How to Contribute

### Reporting Bugs

Open an [issue](https://github.com/matheusmariano/orcker/issues/new?template=bug_report.yml) with:

- OS and version
- Orcker version
- Docker Engine version
- Steps to reproduce
- Expected vs actual behavior

### Suggesting Features

Open a [feature request](https://github.com/matheusmariano/orcker/issues/new?template=feature_request.yml) describing the problem the feature solves and the desired behavior.

### Pull Requests

1. Fork the repository
2. Create a branch: `feat/m1-short-description` or `fix/m2-bug-description`
3. Make your changes with commits following [Conventional Commits](https://www.conventionalcommits.org/)
4. Ensure tests pass: `cargo test` + `pnpm test`
5. Open a PR describing what changes and why

## Local Setup

### Prerequisites

- [Rust stable](https://rustup.rs/) (1.80+)
- Node.js 20+ with pnpm (`npm install -g pnpm`)
- Docker Engine
- **Linux:** `sudo apt install libwebkit2gtk-4.1-dev libxdo-dev libayatana-appindicator3-dev librsvg2-dev libssl-dev build-essential`
- **macOS:** Xcode Command Line Tools
- **Windows:** Visual Studio Build Tools + WebView2

### Install Dependencies

```bash
pnpm install
```

### Run in Development Mode

```bash
pnpm tauri dev
```

### Run Tests

```bash
# Rust backend
cargo test --workspace

# Vue / TypeScript frontend
pnpm test
```

### Production Build

```bash
pnpm tauri build
```

## Commit Convention

All commits must follow [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <imperative description>

[optional body — WHY, not WHAT]
```

**Types:** `feat`, `fix`, `chore`, `docs`, `refactor`, `test`, `style`, `perf`, `ci`, `build`

**Scopes (aligned with PRD modules):**

| Scope | Module |
|---|---|
| `m1-global` | Global Stack |
| `m2-projects` | Project Management |
| `m3-terminal` | Terminal & Commands |
| `m4-logs` | Logs & Observability |
| `m5-infra` | Infrastructure Config |
| `m6-db` | Database Management |
| `m7-settings` | Settings & Preferences |
| `m8-sandbox` | Sandbox & Sharing |
| `ui`, `rust`, `docker`, `ci`, `deps` | Cross-cutting |

**Valid examples:**

```
feat(m1-global): add global services toggle with real-time status
fix(m2-projects): fix legacy docker-compose detection (v2 with hyphen)
chore(ci): configure cross-platform build matrix with tauri-action
refactor(rust): extract DockerAdapter into dedicated module
docs: add local setup guide to docs/development.md
```

## Project Structure

```
orcker/
├── src/                    # Vue 3 frontend
│   ├── components/
│   ├── views/
│   ├── stores/             # Pinia stores
│   └── router/
├── src-tauri/              # Rust backend
│   └── src/
│       ├── adapters/       # Docker, FS, Git, Shell, Keychain
│       ├── core/           # Business logic
│       └── commands/       # Tauri #[command] exports
└── docs/
    └── PRD.md              # Full product requirements
```

## Module Labels

Issues and PRs are tagged `module: M1` through `module: M8` and `phase: 0` through `phase: 3`. See the [PRD](docs/PRD.md) for full module specifications.
