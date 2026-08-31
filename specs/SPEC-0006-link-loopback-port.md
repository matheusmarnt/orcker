---
id: SPEC-0006
title: Container-project sites - `orcker link` with orcker.yml v1 and persistent loopback port allocation
phase: 0
covers: [FR-021, FR-013]
depends_on: [SPEC-0004, SPEC-0005, SPEC-0037]
surface:
  - crates/orcker-core/
  - crates/orcker-config/
  - crates/orcker-ipc/
  - bin/orckerd/
  - bin/orcker/
status: draft
attempts: 0
---

## Context

Closes the Phase-0 golden thread: a project directory becomes a first-class
"container project" site — registered by `orcker link`, described by a minimal
versioned `orcker.yml`, routed by the proxy to a loopback port that the daemon
allocates once and keeps stable. Full stack generation and compose lifecycle
are Phase 1; in Phase 0 the containers still start via `docker compose up`
manually (spike stack), with the compose port matching the allocated one.

**SPEC-0005 outcome (2026-08-31), read before drafting further.** The spike served
a containerized Laravel app at `https://spike.test` with **no code delta at all**:
`orcker proxy add <name> http://127.0.0.1:<port>` plus `orcker secure <name>` already
routes to a loopback upstream, websockets included (`101 Switching Protocols` with the
`vite-hmr` subprotocol preserved). Latency was *not* settled: warm medians were
~25.7-31.3 ms direct versus 113-125 ms proxied on a debug build, dominated by a
per-request TLS handshake that no release build has been measured against - an NFR-02 /
Phase-1 question, not a Phase-0 result. So R1's
"building on the SPEC-0005 delta" has nothing to build on - there is no delta to
consolidate. What this spec actually adds over the inherited mechanism is *port
allocation, persistence and the project registry* (R2-R4), not routing. Findings and
evidence: `docs/spike/PHASE0-SPIKE.md`.

## Requirements

- R1. Site model: `orcker-core` gains a site kind for container projects whose
  upstream is `http://127.0.0.1:<allocated_port>` (building on the SPEC-0005
  delta; consolidate, do not duplicate).
- R2. Port allocation (pure logic + injected probe): allocate from the range
  `20000..=29999`; skip ports already allocated in config and ports the
  injected `PortProbe` reports busy; deterministic given the same inputs;
  typed error when the range is exhausted.
- R3. Persistence: allocations stored in the daemon config
  (`orcker-config`, additive schema migration) keyed by site; `orcker restart`
  and daemon restarts reuse the same port; `unlink` frees it.
- R4. `orcker.yml` v1 (project file, created by `link` when absent, read when
  present): `schema_version: 1`, `site: <name>`, `php: "8.4"`,
  `db: postgres`, `preset: reference`. Parser tolerates unknown keys
  (forward-compat, FR-024 starts here); values validate with the same newtypes
  as `orcker-stack`/`orcker-core`.
- R5. `orcker link [path] [--name <site>] [--port <n>]`: registers the site,
  reads-or-creates `orcker.yml`, allocates (or honors `--port` for the spike
  flow), prints the resulting URL(s). Idempotent: relinking an already-linked
  project changes nothing and says so (FR-021).
- R6. `orcker unlink <site>` unregisters and frees the port;
  `orcker sites` lists container projects with their ports and orcker.yml data.
- R7. IPC: additive messages for link/unlink/list with the new fields.
- R8. Golden thread check: with the SPEC-0005 spike stack running on the
  allocated port, `https://spike.test` serves after
  `orcker link --name spike --port <allocated>` + `orcker secure spike`.

## Design & contracts

`orcker.yml` parsing uses the workspace-pinned `toml`? No — the file is YAML by
product decision (D12); since the workspace deliberately avoids YAML crates so
far, implement a minimal strict reader for the flat v1 schema by hand (pure,
table-tested) OR escalate if that proves unreasonable — adding a YAML
dependency is NOT authorized by this spec. `PortProbe` is a trait with a real
TCP-bind impl at the edge and a fake in tests.

## Test plan

- Unit (pure, table-driven): allocation matrix (free/busy/persisted/exhausted);
  orcker.yml v1 parse/serialize round-trip incl. unknown-key tolerance and
  invalid values; router mapping for the new site kind.
- Integration (fakes): link flow against fake probe + in-memory config
  (create-when-absent, read-when-present, idempotence); unlink frees the port.
- E2E/manual: R8 golden thread on Linux, transcript in the cycle log.

## Acceptance checklist

- [ ] AC1 Allocation is deterministic, collision-free and persistent ->
      tests: `orcker_core::ports::allocation_matrix`,
      `orcker_config::ports::allocation_roundtrip`
- [ ] AC2 orcker.yml v1 round-trip with unknown-key tolerance ->
      test: `orcker_config::orcker_yml::v1_roundtrip_and_forward_compat`
- [ ] AC3 `orcker link` is idempotent (second run: no changes, explicit
      message) -> test: `link::idempotent_relink` (fake edge)
- [ ] AC4 `orcker sites --json` lists the project with port and metadata ->
      evidence: output in the cycle log
- [ ] AC5 Golden thread (R8) passes -> evidence: transcript in the cycle log
- [ ] AC6 IPC wire-stability suite green, extensions additive only
- [ ] AC7 `scripts/gate.sh specs/SPEC-0006-*.md` passes

## Out of scope

Stack generation on link (Phase 1, SPEC-0013/0014); `park` for container
projects; compose lifecycle from the daemon; port range configurability; the
`docker/`-files ownership rules (FR-023).

## Agent notes

Read first: `crates/orcker-config/src/{schema,migrate}.rs` (additive migration
pattern + `config-schema-history.md` in docs/developer), `crates/orcker-core`
site/router modules as touched by SPEC-0005, the inherited `link` command
implementation in `bin/orcker`. Pitfalls: the config crate is pure with an
`io.rs` edge — schema changes need a migration entry and a history note; do not
bind sockets in `orcker-core` (probe stays a trait); site names share DNS-label
validation with `orcker-stack::SiteName` — reuse the core newtype rather than
duplicating rules.
