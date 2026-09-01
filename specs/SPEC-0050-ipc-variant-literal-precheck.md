---
id: SPEC-0050
title: Make the out-of-surface struct-literal check a design-time precheck, not a gate surprise
phase: 0
covers: [FR-002]
depends_on: [SPEC-0004, SPEC-0006]
surface:
  - .github/
status: draft
attempts: 0
---

## Context

Twice now a cycle has designed an additive IPC change, written it, and only then
discovered the shape was unbuildable inside its surface.

- SPEC-0004 wanted a field on `orcker_ipc::StatusReport`. It is built with a full
  struct literal at nine sites across five crates.
- SPEC-0006 wanted a `projects` field on `Response::Sites`. It is built with a
  full struct literal in `crates/orcker-mcp/tests/render.rs`.

**The gate already detects this.** `scripts/gate.sh` step 3 runs
`cargo test --workspace`, so an out-of-surface literal fails to compile every
time; SPEC-0006 saw exactly that (`missing field 'projects'`, `orcker-mcp`).
Detection is not the gap. The gap is *when*: the failure lands after the design
is committed to, and the only two ways out are a surface violation or a mid-cycle
redesign. SPEC-0006 paid the redesign, rewriting `Response::Sites { projects }`
into a separate `Response::Projects` after the code was written.

The fix is therefore not another check. It is a ten-second grep before choosing
the shape, written down where the next cycle will read it.

## Requirements

- R1. `.github/instructions/orcker-ipc.instructions.md` gains a precheck under
      its *Contract rules*: before adding a field to an existing `Request` or
      `Response` variant (or to a struct one carries), run
      `git grep -nE '(Request|Response)::<Variant> \{' -- '*.rs'` and confirm
      every hit is inside the spec's declared `surface:`. A hit outside it means
      the field is not available; use a new variant, or declare those files in
      the surface up front.
- R2. The note states the escape hatch with its precedent: a separate request
      merged at render time, as `orcker status` does with `Status` +
      `EngineStatus` and `orcker sites` does with `ListSites` + `ListProjects`.
- R3. Both instances (SPEC-0004 `StatusReport`, SPEC-0006 `Response::Sites`) are
      cited by name so the rule reads as evidence, not as folklore.

## Acceptance checklist

- [ ] AC1 (R1) the precheck command is in the instruction file and runs clean
      against the current tree -> evidence: command + output in the cycle log
- [ ] AC2 (R2, R3) both escape hatch and both precedents are named ->
      evidence: `grep -n 'StatusReport\|Response::Sites' .github/instructions/orcker-ipc.instructions.md`
- [ ] AC3 `scripts/gate.sh specs/SPEC-0050-*.md` passes

## Out of scope

Any change to `scripts/gate.sh` (it already catches this, and DT7 forbids
touching it). Making `StatusReport` or `Response` variants `#[non_exhaustive]`:
that forbids the daemon's own literal and breaks more than it fixes.
