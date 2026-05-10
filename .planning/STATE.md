# Project State — Orcker

*Last updated: 2026-05-10*

## Current Phase

**Phase 0 — COMPLETE.** Scaffold verified. Planning complete.
**Phase 1 — READY TO PLAN.** Run `/gsd:plan-phase 1` to start.

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

- [ ] Merge release-please PR #9 (v0.1.0) — pending CI green
- [ ] Phase 1: Foundation — Docker socket, AppState, bollard, tauri-specta, tracing

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

- `initial-version: "0.1.0"` in release-please-config.json — prevents 1.0.0 proposal from scaffold commits
- `bump-minor-pre-major: true` — feat commits bump minor (not major) pre-1.0.0
- v0.0.0 GitHub Release created manually as anchor at `c15bce69`
- `tauri-specta` chosen for Rust↔TS type bridge — verify version at Phase 1 start
- `Channel<T>` for IPC streams, `invoke` for one-shot commands
- `docker compose` shell-out for Compose orchestration (bollard doesn't implement Compose)

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
