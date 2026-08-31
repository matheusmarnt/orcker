---
id: SPEC-0004
title: Create orcker-engine with Docker environment detection surfaced in `orcker status`
phase: 0
covers: [FR-010]
depends_on: [SPEC-0001, SPEC-0037]
surface:
  - Cargo.toml
  - Cargo.lock
  - crates/orcker-engine/
  - crates/orcker-ipc/
  - bin/orckerd/
  - bin/orcker/
status: accepted
attempts: 1
---

## Context

First I/O-edge crate of the fork: detect whether Docker is usable before any
lifecycle work exists. Follows the inherited pattern exactly — pure decision
logic + side effects behind traits with one real implementation and fakes in
tests (like `orcker-php`'s old `traits.rs`/`real.rs` split, now removed).

## Requirements

- R1. New workspace member `crates/orcker-engine` with a `pure/` layer and an
  I/O edge behind traits.
- R2. Detection model (pure): `EngineStatus { socket: SocketAddrKind,
  reachable: bool, engine_version: Option<Version>, compose: ComposeStatus,
  problems: Vec<EngineProblem> }` where `ComposeStatus` is
  `Found(Version) | Missing | TooOld{found, min}` and every `EngineProblem`
  carries an actionable hint string (NFR-08).
- R3. Socket resolution order (pure function over injected env/values):
  `DOCKER_HOST` env if set, else the platform default unix socket
  (`/var/run/docker.sock`; on macOS also probe the Docker Desktop user socket
  `~/.docker/run/docker.sock`). No Windows path (stub returns Unsupported).
- R4. I/O traits: `EngineApi` (ping + version — real impl uses `bollard`) and
  `ComposeCli` (runs `docker compose version --format json` via the process
  spawner convention used in the codebase). Real impls live at the crate's I/O
  edge; tests use in-memory fakes.
- R5. Minimum supported versions declared as crate constants (initial values:
  Engine `24.0`, compose `2.20` — PRD NFR-04 fixes them in Phase 0; adjust ONLY
  via these constants) and compared in pure code.
- R6. IPC: additive request `EngineStatus` returning R2's data; wire-stability
  baseline extended additively (no renames, no removals).
- R7. Daemon wires the detection at startup and on demand (cached with a short
  TTL owned by the daemon, not the crate).
- R8. CLI: `orcker status` gains a `docker` section (human output + `--json`
  with `docker.engine_version`, `docker.compose_version`, `docker.problems[]`).
  With Docker stopped, output includes the problem + hint and exits 0 (status
  reports, it does not fail).

## Design & contracts

New dependency (declared): `bollard` (workspace-pinned, rustls features only —
never native-tls, consistent with the TLS hard rule). All bollard usage stays
inside `orcker-engine`'s I/O edge; no other crate may depend on it.

## Test plan

- Unit (pure, table-driven): socket resolution matrix (env set/unset, per-OS
  defaults); version comparison matrix (ok / too old / missing); problem->hint
  mapping.
- Integration (fakes): `EngineApi` fake returning version/err -> `EngineStatus`
  assembly; `ComposeCli` fake with real captured `docker compose version`
  output strings (fixtures) -> parser.
- E2E/manual: `orcker status` against real Docker running and stopped —
  outputs recorded in the cycle log.

## Acceptance checklist

- [x] AC1 Socket resolution honors DOCKER_HOST then platform defaults ->
      test: `orcker_engine::pure::socket_resolution_matrix`
- [x] AC2 Compose JSON output parsing tolerates real-world variants ->
      test: `orcker_engine::pure::compose_version_parsing`
- [x] AC3 Versions below minimum produce `TooOld` with hint ->
      test: `orcker_engine::pure::minimum_version_policy`
- [x] AC4 IPC `EngineStatus` round-trips; wire-stability suite green and
      only extended -> test: `orcker_ipc::wire_stability`
- [x] AC5 `orcker status --json` contains the docker section; with the engine
      down it reports the problem with hint, exit code 0 ->
      evidence: both outputs in the cycle log
- [x] AC6 `scripts/gate.sh specs/SPEC-0004-*.md` passes

## Out of scope

Compose lifecycle (up/down/build — SPEC-0010); container observation/events
(SPEC-0011); doctor integration (SPEC-0020); GUI display of the docker section.

## Agent notes

Read first: `crates/orcker-platform/src` (trait style, per-OS cfg discipline,
`ProcessSpawner` convention), `crates/orcker-ipc` request/response modules and
`tests/wire_stability.rs` (additive extension pattern), `bin/orcker` status
command rendering. Pitfalls: keep `tokio` usage inside the I/O edge only;
bollard pulls hyper — ensure `no_runtime_deps`-style graph tests of the
binaries are updated deliberately (in-surface, justified); never shell out to
`docker` without going through the spawner trait (testability + the no-network
rule for pure layers).
