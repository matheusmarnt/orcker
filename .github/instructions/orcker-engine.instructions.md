---
applyTo: "crates/orcker-engine/**/*.rs"
---

# orcker-engine — Docker Engine detection & compose CLI

Detects a usable Docker Engine and Compose plugin, and is the only crate
allowed to talk to Docker: the Engine API via `bollard`, and the `docker
compose` CLI via a spawned process.

**Layer split:** `pure/` (`detect`, `socket`, `version`) is sync, runtime-free,
no I/O. `io/` is the only module allowed to touch `bollard` or spawn a
process. `probe.rs` orchestrates the two but owns no policy — it calls
`pure::assemble`. Side effects are abstracted behind `traits.rs`
(`EngineApi`, `ComposeCli`), both `#[async_trait]`.

## Owns

- `detect(...)`: tries engine/socket candidates in order, first answer wins.
- `io::BollardEngine` (Engine API via `bollard`, `/version` as the ping,
  `CONNECT_TIMEOUT_SECS = 4`) and `io::DockerComposeCli` (`docker compose
  version --format json` via `tokio::process::Command`).
- `io::detect_from_env()` — the crate's only env reads (`DOCKER_HOST`, `HOME`).
- `pure::assemble`, `pure::resolve_socket`, `pure::parse_compose_version`,
  `pure::hint_for`, `pure::endpoint_label`, `pure::ProbeOutcome`,
  `pure::Version` (`new` / `parse` / `satisfies`).
- `MIN_ENGINE_VERSION` (`24.0.0`) and `MIN_COMPOSE_VERSION` (`2.20.0`) in
  `src/pure/mod.rs` — the single source of the supported floor (PRD NFR-04),
  pinned by `minimum_version_policy` (`src/pure/mod.rs`).
- `EngineError` (`Unreachable`, `ComposeUnavailable`, `Unsupported`).

Wire types consumed by clients (`DockerStatus`, `SocketKind`, `ComposeStatus`,
`EngineProblem`, `EngineProblemCode`) live in `orcker-ipc` deliberately, so a
CLI/GUI client never links `bollard`. `orcker-ipc` is this crate's only
`orcker-*` dependency.

## Must not

- Let `bollard` or process spawning appear outside `io/`.
- Enable a `bollard` TLS/`ssl*` feature. The workspace is rustls-only; the pin
  is `default-features = false, features = ["http", "pipe"]` (root
  `Cargo.toml`, see the comment beside it) — bollard's `ssl` features pull in
  OpenSSL.
- Hardcode a version number inside a hint string. `hint_for` must produce a
  hint that *contains* the relevant constant's `to_string()`, so the floor has
  one source of truth.
- Put detection policy in `probe.rs`; it stays a thin orchestrator over
  `pure::assemble`.
- Talk to a real Docker daemon in a test — drive `EngineApi` / `ComposeCli`
  through fakes.

## Conventions

- Raising `MIN_ENGINE_VERSION` or `MIN_COMPOSE_VERSION` is a one-line change
  in `src/pure/mod.rs`; `minimum_version_policy` re-derives every downstream
  hint and problem from it, so no other file should duplicate the number.

## Tests / invariants

- `src/pure/mod.rs` table tests: `socket_resolution_matrix` (DOCKER_HOST/HOME/OS
  combinations, including `ssh://` → `Unsupported`), `compose_version_parsing`
  (JSON, capitalised key, plain-text fallback, `-desktop.N` suffix, malformed →
  `None`), `minimum_version_policy` (healthy vs. too-old engine/compose,
  boundary versions, every problem has a non-empty hint).
- `tests/assemble.rs` — `FakeEngine` / `FakeCompose` drive the five
  detection scenarios (healthy, desktop-socket fallback, engine down, compose
  absent, unsupported platform) through the real traits, no real daemon.
- Consumers to re-check on a signature change: `bin/orckerd/src/engine_status.rs`,
  `bin/orckerd/src/state.rs`.

## Review checklist

- [ ] `bollard` and process spawning stay inside `io/`; `pure/` and
      `probe.rs` remain I/O-free / policy-only respectively.
- [ ] No bollard TLS/`ssl*` feature enabled.
- [ ] Any hint or problem message embeds the constant, not a literal number.
- [ ] New detection scenarios are covered via `EngineApi`/`ComposeCli` fakes,
      not a real daemon.
- [ ] `MIN_ENGINE_VERSION` / `MIN_COMPOSE_VERSION` changed only deliberately,
      with `minimum_version_policy` updated in the same diff.
