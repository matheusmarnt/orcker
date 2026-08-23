---
id: SPEC-0040
title: Turn the dead-export delta scan into a standing ratchet
phase: 0
covers: [FR-002]
depends_on: [SPEC-0036]
surface:
  - apps/orcker-gui/
status: draft
attempts: 0
---

## Context

SPEC-0036 added `apps/orcker-gui/tests/dead-export-delta.mjs` and made it R1's
completeness criterion, but deliberately scoped it to `dead(HEAD) -> dead(now)`
and kept it out of the gate: the answer depends on `HEAD`, so it is cycle
evidence, not a standing check.

That leaves the absolute number unguarded. At SPEC-0036's `HEAD` the GUI carried
**96 exports with no consumer**; the diff left 77. Nothing stops that number
climbing, and nothing tells a later spec which of the 77 it just inherited.

SPEC-0036 tried the obvious standing version and rejected it on measured output
(see `specs/logs/SPEC-0036.md` S8). Any spec that revives it must answer what
that attempt could not:

- An ~77-entry allowlist is 24% of the tree's exports. A ratchet that large
  reads as permission, not as a guard, unless it is visibly shrinking.
- "Unreferenced outside its home file" is not "unused": `jobStatus` is called by
  `pollJobToEnd` in the same file. The rule needs a same-file-usage pass, or the
  convention that such helpers lose their `export` (SPEC-0036 did that to
  `jobStatus` and `ComboboxOption`).
- `src/ipc/types.ts` is a wire-contract mirror. Its exports mirror the daemon's
  `Response` variants whether or not today's GUI reads each one, so a naive gate
  demands deleting the protocol - including the `JobState` that SPEC-0036 R3
  orders kept. The exemption must be designed, not bolted on.

Three further blind spots belong to the same reckoning, found by the SPEC-0036
supervisor probing the delta scan itself:

- Test files sit **inside** the scanned subtree, so they count as consumers. An
  export kept alive only by its own test is invisible - the `phpSettings.ts`
  class, caught in SPEC-0036 only because that test happened never to name
  `TEXT_SETTINGS`. 13 exports are in that state today.
- A `\bname\b` match over raw text counts a mention in a **comment** as usage:
  `formatLoadAvg` and `invalidate` are alive today only in prose.
- The scan's `SUBTREE` is `apps/orcker-gui/src`, so `src-tauri/` is unscanned -
  the Rust half, where a dead `pub(crate)` fn would hide.

## Requirements

- R1. Decide the standing form: a shrink-only allowlist seeded with the current
  set, or a cap on the count. Whichever is chosen must fail on a *new* dead
  export while ignoring the inherited ones.
- R2. Handle the same-file-usage case so a private helper is never reported as
  dead - either by detecting it or by un-exporting the helpers and pinning that
  as the convention.
- R3. Give `src/ipc/types.ts` (and any other contract mirror) a documented
  exemption whose reason is written down where the next reader will find it.
- R5. Close the three blind spots above: classify test-only consumers as not
  consumers, ignore matches inside comments and strings, and extend the scan to
  `src-tauri/`.
- R4. Dispose of the inherited set: each entry deleted, un-exported, or recorded
  with a reason. SPEC-0037 is the natural partner for the ones that are
  SPEC-0002's leftovers.

## Test plan

- RED first: add a dead export, watch the check fail, remove it.
- The delta scan stays: the two answer different questions.

## Acceptance checklist

- [ ] AC1 A new dead export fails the check; RED recorded
- [ ] AC2 The inherited set is disposed of, each entry with a disposition
- [ ] AC3 `scripts/gate.sh specs/SPEC-0040-*.md` passes

## Out of scope

Dead code in `crates/` and `bin/`. This is the GUI's export graph.
