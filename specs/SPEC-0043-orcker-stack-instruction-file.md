---
id: SPEC-0043
title: Bring the path-scoped instruction files up to date with the crates that landed
phase: 0
covers: [FR-022]
depends_on: [SPEC-0003, SPEC-0004]
surface:
  - .github/
status: draft
attempts: 0
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
