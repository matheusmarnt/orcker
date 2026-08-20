---
id: SPEC-0003
title: Create orcker-stack (pure) with a typed model rendering a minimal reference compose
phase: 0
covers: [FR-022]
depends_on: [SPEC-0001]
surface:
  - Cargo.toml
  - Cargo.lock
  - crates/orcker-stack/
status: draft
attempts: 0
---

## Context

First new crate of the fork and the calibration exercise for the process:
a 100% pure library that turns a typed stack model into a rendered
`docker-compose.yml` string following the reference architecture (PRD D02,
`referenciadockerlaravel.md`). This spec covers the minimal renderer for the
PostgreSQL reference topology; the full template set (Dockerfile, nginx, php,
supervisord, MySQL variant) is Phase 1 (SPEC-0007/0008).

## Requirements

- R1. New workspace member `crates/orcker-stack`, pure per the inherited rule:
  no I/O, no clock/env reads, no async, `#![forbid(unsafe_code)]`, thiserror
  error enum `StackError`.
- R2. Typed model (all fields explicit, no stringly-typed maps):
  `StackConfig { site: SiteName, php: PhpVersion, db: DbEngine::Postgres,
  preset: Preset::Reference, uid: u32, gid: u32, http_loopback_port: u16,
  vite_port: u16 }` plus the value types. `DbEngine`/`Preset` are enums with a
  single variant each for now (non_exhaustive; more variants in Phase 1).
- R3. `pub fn render_compose(cfg: &StackConfig) -> Result<String, StackError>`
  produces a compose file with, exactly as the reference doc prescribes:
  services `app`, `nginx`, `db`; `restart: "no"` on all services; a healthcheck
  on `db` (`pg_isready`) and `depends_on: { db: { condition: service_healthy } }`
  on `app`; volumes `./:/var/www` (app), `./public` and
  `./storage/app/public:ro` (nginx), named volume for db data; project-internal
  network plus `development` as `external: true`; nginx port published ONLY as
  `127.0.0.1:{http_loopback_port}:80`; Vite port `127.0.0.1:{vite_port}:5173`
  on `app`; `env_file: .env` on `db`; `FASTCGI_PASS=app:9000` environment on
  `nginx`.
- R4. Deterministic output: same input -> byte-identical output (stable key
  order); rendering uses explicit string building or a template constant in the
  crate — either way covered by snapshot tests committed as fixtures.
- R5. Validation in the constructor: `SiteName` (lowercase alphanumeric +
  hyphen, DNS-label rules), `PhpVersion` (enum 8.1..8.5), ports non-zero and
  distinct; violations return typed errors, never panic.

## Design & contracts

New dependencies: none preferred (hand-rolled YAML emission with tests) —
`serde_yaml`-style crates are NOT approved. If emission via `serde` +
workspace-pinned `serde_json`-like machinery proves insufficient, escalate
rather than adding a dependency. Errors:
`StackError { InvalidSiteName{..}, InvalidPort{..}, .. }`.

## Test plan

- Unit (table-driven): SiteName/port validation matrix; render snapshot for the
  reference config (fixture file `tests/fixtures/compose_reference_postgres.yml`);
  determinism test (two renders byte-equal); loopback-only publishing asserted
  by parsing the rendered lines (`127.0.0.1:` prefix present, `0.0.0.0` absent).
- Integration: none (pure crate).
- E2E/manual (documented, non-blocking for this spec): `docker compose config`
  on the rendered output exits 0 — recorded in the cycle log; the automated
  version of this check lands with SPEC-0010 (engine available in tests).

## Acceptance checklist

- [ ] AC1 Reference render matches the committed snapshot ->
      test: `orcker_stack::compose::reference_postgres_snapshot`
- [ ] AC2 Renders are deterministic -> test: `orcker_stack::compose::deterministic_output`
- [ ] AC3 No project port on 0.0.0.0; nginx/vite published on 127.0.0.1 only ->
      test: `orcker_stack::compose::loopback_only_ports`
- [ ] AC4 Invalid site names and ports return typed errors ->
      test: `orcker_stack::validate::rejects_invalid_inputs`
- [ ] AC5 `docker compose config` accepts the rendered file ->
      evidence: command output in the cycle log
- [ ] AC6 `scripts/gate.sh specs/SPEC-0003-*.md` passes

## Out of scope

Dockerfile/nginx/php/supervisord/init templates; MySQL variant; minimal preset;
writing files to disk (a pure crate returns strings; the I/O edge integrates in
SPEC-0012/0013); `orcker.yml` schema (SPEC-0006 starts it).

## Agent notes

Read first: `docs/developer/architecture.md` (pure/io split),
`crates/orcker-core/src` for value-type conventions (`tld.rs`, `host.rs` show
the validated-newtype style), the reference doc tables (volumes, networks,
healthcheck). Pitfall: keep `orcker-stack` out of any binary's runtime graph
until a consumer spec wires it (workspace member + tests only) so
`no_runtime_deps` graphs stay untouched.
