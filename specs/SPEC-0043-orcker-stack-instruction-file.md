---
id: SPEC-0043
title: Bring the path-scoped instruction files up to date with the crates that landed
phase: 0
covers: [FR-022]
depends_on: [SPEC-0003, SPEC-0004]
surface:
  - .github/
status: accepted
attempts: 1
---

## Context

The filename says `orcker-stack-instruction-file` and the title no longer does.
Leave it: `specs/logs/SPEC-0003.md` cites this path, and closed cycle logs are
never rewritten (`specs/DECISIONS.md`, 2026-08-30). The id is the identifier.

Every inherited crate has a `.github/instructions/<crate>.instructions.md` that
agents must read before editing it. SPEC-0003 created `crates/orcker-stack` and
SPEC-0004 created `crates/orcker-engine`, neither with one, because `.github/`
sits outside those specs' surfaces. Add both files (layer, owns, must-not,
conventions, review checklist) mirroring `orcker-core.instructions.md`, and do
the same for `orcker-catalog` when it lands.

`orcker-engine.instructions.md` has two specifics its neighbours do not: the
crate is the only place `bollard` may appear (no TLS feature is enabled - see
`specs/DECISIONS.md`, 2026-08-31), and `MIN_ENGINE_VERSION` / `MIN_COMPOSE_VERSION`
in `src/pure/mod.rs` are the single source of the supported floor, pinned by an
assertion in `minimum_version_policy`.

Amend `orcker-ipc.instructions.md` in the same cycle with the trap SPEC-0004
hit: `StatusReport` is **not** `#[non_exhaustive]` and is built with a full
struct literal at nine sites across five crates, so adding a field to it breaks
`orcker-doctor`, `orcker-mcp`, `orcker-ipc`'s own tests and both binaries. Only
`bin/orckerd/src/ipc_server.rs` is a production site; the other eight are test
fixtures. A spec that intends to extend `StatusReport` has to declare all of
them in its `surface:` up front. Name them, and name the two non-fixes so the
next cycle does not re-derive them: `#[non_exhaustive]` would forbid the
daemon's own literal, and `derive(Default)` does not compile because
`dns_addr: SocketAddr` has no `Default`. SPEC-0020 (doctor Docker checks) is the
first queued spec that will hit this, because `orcker_doctor::diagnose` takes a
`&StatusReport`.

## Requirements

- R1. `.github/instructions/orcker-stack.instructions.md` exists, `applyTo:
      "crates/orcker-stack/**/*.rs"`, mirroring `orcker-core.instructions.md`
      (layer, owns, must-not, conventions, tests/invariants, review checklist).
      It states the crate is strictly pure (no I/O, no async, no `orcker-*`
      dependency) and that the rendered compose file must never publish on
      `0.0.0.0`, set `privileged: true`, or mount the Docker socket.
- R2. `.github/instructions/orcker-engine.instructions.md` exists, `applyTo:
      "crates/orcker-engine/**/*.rs"`, same shape, and names the two specifics
      its neighbours do not have: `bollard` may appear in this crate only and
      with no TLS feature enabled (`specs/DECISIONS.md`, 2026-08-31), and
      `MIN_ENGINE_VERSION` / `MIN_COMPOSE_VERSION` in `src/pure/mod.rs` are the
      single source of the supported floor, pinned by `minimum_version_policy`.
- R3. `.github/instructions/orcker-ipc.instructions.md` gains the `StatusReport`
      trap under its *Contract rules*: the struct is not `#[non_exhaustive]` and
      is built with a full struct literal at nine sites across five crates, so a
      spec intending to extend it must declare all of them in `surface:` up
      front. The one production site (`bin/orckerd/src/ipc_server.rs`) is
      distinguished from the eight test fixtures.
- R4. The same note records the two non-fixes so the next cycle does not
      re-derive them: `#[non_exhaustive]` would forbid the daemon's own literal,
      and `derive(Default)` does not compile because `dns_addr: SocketAddr` has
      no `Default`. It names SPEC-0020 as the first queued spec that will hit
      this, because `orcker_doctor::diagnose` takes a `&StatusReport`.

## Acceptance checklist

- [ ] AC1 (R1) → evidence: `test -f .github/instructions/orcker-stack.instructions.md`
      exits 0 and `grep -c '0\.0\.0\.0\|privileged\|docker\.sock'` on it prints 3 or more
- [ ] AC2 (R2) → evidence: `grep -n 'bollard\|MIN_ENGINE_VERSION\|MIN_COMPOSE_VERSION\|minimum_version_policy'
      .github/instructions/orcker-engine.instructions.md` prints all four
- [ ] AC3 (R3) the nine sites are enumerated and match the tree → evidence: the
      file lists them, and `git grep -nE 'StatusReport \{' -- '*.rs'` in the cycle
      log shows the same nine construction sites across five crates
- [ ] AC4 (R4) both non-fixes and SPEC-0020 are named → evidence:
      `grep -n 'non_exhaustive\|Default\|SPEC-0020' .github/instructions/orcker-ipc.instructions.md`
- [ ] AC5 `scripts/gate.sh specs/SPEC-0043-orcker-stack-instruction-file.md` passes

FR acceptance: FR-022 has AC1/AC2/AC3 (`docs/PRD.md`). AC2 (`docker compose
config` accepts the render) closed by SPEC-0003 AC5. AC1 (snapshots over
`{postgres,mysql} × {reference,minimal} × {fino,source}`) — open, only
`postgres × reference` exists; closed by SPEC-0007 and SPEC-0008. AC3 (host
UID/GID applied at build) — open, closed by SPEC-0007. This spec documents the
crate that renders those templates and closes none of FR-022's ACs on its own.

## Out of scope

`orcker-catalog` (the crate has not landed). The other crates without an
instruction file (`orcker-depcheck`, `orcker-release-manifest`,
`orcker-service-ctl`, `orcker-update`, `orcker-mail`, `orcker-tunnel`). The IPC
struct-literal *precheck command*, which is SPEC-0050's R1. Any `.rs` change.
