---
id: SPEC-0054
title: Nothing joins a spec's acceptance checklist to the FR acceptance it claims to close
phase: 0
covers: [FR-001]
depends_on: [SPEC-0045, SPEC-0047]
surface:
  - docs/
  - .claude/
status: accepted
attempts: 1
---

## Context

`DT3` checks that every AC **of the spec** maps to a test or executable evidence,
and stops there. A spec declares `covers: [FR-XXX]` in its front matter, and
nothing anywhere checks that FR-XXX's own acceptance criteria — the ones written
in `docs/PRD.md` — are actually satisfied. The join between the two records does
not exist.

Two consequences already in the tree:

**SPEC-0005 claimed FR-003 with AC1 unwritten.** FR-003 has three ACs; the spike
closed AC2 and AC3 and never enumerated AC1 (`orcker link` registers the site
with a loopback upstream). It took a later reading to notice, and the roadmap
carried `FR-003 (partial)` as a hand-applied annotation rather than as anything
derived.

**SPEC-0006's own record then pointed at the wrong AC.** Its `DECISIONS.md`
entry named FR-003 **AC1** as the criterion left open by the golden-thread
deviation. AC1 is the one criterion that needs no browser, and it is precisely
what that spec closed; the open-looking one was AC2, closed by SPEC-0005. The
entry was wrong in the file the next cycle is told to read, and it was corrected
only because a human asked whether the elevate run was actually necessary.

Both failures share a cause: **nobody is ever forced to write the FR's ACs down
next to the spec's.** The fix belongs at spec-writing time, where enumerating
three bullet points is free, not at verification time, where it needs a parser
for Portuguese prose.

## Requirements

- R1. `docs/SDD.md` section 4 (the spec contract): when a spec's `covers:` names
      an FR, its **Acceptance checklist** must account for every AC of that FR —
      each one either closed by an item of this spec's checklist, or listed under
      a short *FR acceptance* note saying which AC stays open and which spec is
      expected to close it.
- R2. The `(partial)` marker already used in `specs/ROADMAP.md` becomes the
      derived consequence of R1, not an annotation applied by hand: an FR is
      `(partial)` exactly while R1's note lists an open AC for it.
- R3. `docs/SDD.md` section 8.1 gains `DT10`: for each FR in `covers:`, every AC
      of that FR appears in the spec's checklist or in R1's note. Verifiable by
      reading two files — no new tooling.
- R4. `DT10` routes to `REWORK`, not `ESCALATE`: unlike `DT9`, an agent can fix
      it by enumerating what it left out. Section 8.3's decision block says so.
- R5. `.claude/agents/spec-writer.md` and `.claude/commands/spec-new.md` require
      the enumeration at drafting time, so a spec that would fail `DT10` is not
      written in the first place.

## Test plan

Record-only, no Rust. The discriminating check is a negative reproduction, scoped
to FR-003 (the FR the Context section discusses): apply `DT10` by hand to
`specs/SPEC-0005-proxy-container-spike.md` as committed — `covers: [FR-003]` and
its checklist never enumerates FR-003 AC1 — and confirm it **fails**. Then apply
it to `specs/SPEC-0006-link-loopback-port.md` — `covers: [FR-021, FR-013]`, FR-003
is not among them, so `DT10` imposes nothing about FR-003 there — and confirm it
**passes** (vacuously, for FR-003 specifically; a full `DT10` audit of SPEC-0006's
own `FR-021`/`FR-013` coverage is out of scope, see below). A rule that does not
fail the case that motivated it is decoration.

## Acceptance checklist

- [x] AC1 (R1, R2) the contract requires the enumeration and derives `(partial)`
      -> evidence: `grep -n 'FR acceptance' docs/SDD.md`
- [x] AC2 (R3, R4) `DT10` exists in the 8.1 table and in the 8.3 decision block,
      routing to REWORK -> evidence: `grep -n 'DT10' docs/SDD.md`
- [x] AC3 negative reproduction, scoped to FR-003: `DT10` applied to SPEC-0005
      fails on FR-003 AC1; applied to SPEC-0006 passes vacuously (FR-003 is not
      in SPEC-0006's `covers`). Both transcripts in the cycle log
- [x] AC4 (R5) drafting enforces it -> evidence:
      `grep -n 'covers' .claude/agents/spec-writer.md .claude/commands/spec-new.md`
- [x] AC5 no accepted spec is edited -> evidence: `git diff --name-only` lists no
      `specs/SPEC-00*.md` other than this one
- [x] AC6 `scripts/gate.sh specs/SPEC-0054-*.md` passes

FR acceptance: FR-001 has AC1/AC2/AC3 (`docs/PRD.md`); AC2 and AC3 closed by
SPEC-0001. AC1 (`cargo fmt`/`clippy`/`test` green) is this cycle's own gate,
closed by AC6 above.

## Out of scope

Parsing `docs/PRD.md` into machine-readable ACs, and any gate step that does so:
the ACs are prose, and a parser would be a second source of truth that drifts
from the first. Back-filling the enumeration into the thirteen accepted specs —
their records are closed, and AC3 reads SPEC-0005 without editing it. Editing
`docs/PRD.md`, which is forbidden outside an RFC.

## Agent notes

Do **not** reach for a new ledger file mapping FR ACs to specs. That was the
first design considered here and it loses to R1 on the same ground `(partial)`
loses to R2: a second record of the same fact drifts from the first, which is
the failure `specs/TRACEABILITY.md` needed SPEC-0047 to repair. The spec's own
checklist is already the record — R1 only requires that it be complete.

One line falls out of this cycle for free and should ride along:
`specs/ROADMAP.md`'s Phase-0 header still reads `FR-003 partial`, which stopped
being true when SPEC-0006 closed AC1.
