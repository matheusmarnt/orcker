---
id: SPEC-0052
title: Cycle-log evidence must be captured output, never a re-asserted paste
phase: 0
covers: [FR-001]
depends_on: [SPEC-0045, SPEC-0047]
surface:
  - docs/
status: draft
attempts: 0
---

## Context

SPEC-0006 round 2 was a REWORK with every deterministic check green and no code
defect. It failed on the record: the AC5 block claimed "re-run after the
round-1 rework" while holding the paste from *before* it. The run had genuinely
happened; only the transcript was stale, because the cycle edited a number
inside the old block instead of replacing it.

The supervisor caught it from the code alone, on two tells: the shipped renderer
emits a `DOMAIN` column unconditionally, and `DEFAULT_PHP` can only write `8.4`,
so a transcript lacking both could not have come from that tree. That catch was
luck of a sort - it needed a reviewer willing to re-derive a table header from
source. A cheaper mechanism should exist.

This is the third record-integrity failure in the fork's history, after
SPEC-0005 (a retracted latency number left standing in `DECISIONS.md` and a
spec note) and SPEC-0047 (a column shift that a field count could not see).
The pattern is the same each time: **a record edited by hand drifts from the
run it claims to describe.**

## Requirements

- R1. `docs/SDD.md` section 6's anti-drift rules gain: evidence in a cycle log
      is *captured*, not typed. Run the command through `tee` into
      `specs/logs/<spec-id>-evidence/<name>.txt` (or paste the captured file
      verbatim), and never edit inside an evidence block.
- R2. Same section: when a fix changes observable output, the old evidence block
      is **deleted before** the command is re-run. Editing a value inside a stale
      block is what produced SPEC-0006's failure, and the phrase "re-run" must
      never be asserted over a block that was not itself replaced.
- R3. Each evidence block names what produced it: the commit or, for an
      uncommitted tree, the binary's build time. A reviewer can then falsify the
      claim without re-deriving it from source.
- R4. `docs/SDD.md` section 8's DT4 gains the mirror check: the supervisor
      rejects an evidence block whose content contradicts the tree, and treats a
      "re-run" claim over unchanged output as the specific failure to look for.

## Acceptance checklist

- [ ] AC1 (R1, R2) the two rules exist -> evidence:
      `grep -n 'tee\|re-run' docs/SDD.md`
- [ ] AC2 (R3) the provenance rule exists and the cycle-log template shows it
- [ ] AC3 (R4) DT4's wording covers a contradicted transcript -> evidence:
      `grep -n 'DT4' docs/SDD.md`
- [ ] AC4 the three prior instances (SPEC-0005, SPEC-0006, SPEC-0047) are cited
      so the rule reads as evidence rather than ceremony
- [ ] AC5 `scripts/gate.sh specs/SPEC-0052-*.md` passes

## Out of scope

Automating the capture (a wrapper script, a hook). Prove the rule holds by hand
for one cycle first. Rewriting past cycle logs: their failures are recorded and
the record of a failure is not itself a defect.
