---
id: SPEC-0047
title: Make the traceability record able to answer SDD section 11 before the Phase-0 retrospective runs
phase: 0
covers: [FR-001]
depends_on: [SPEC-0045]
surface:
  - specs/
status: draft
attempts: 0
---

## Context

`docs/SDD.md` section 11 mandates four process metrics and says the retrospective
runs every ten accepted specs. The tenth (SPEC-0005) closed on 2026-08-31, so the
retrospective is due - but `specs/TRACEABILITY.md` cannot produce three of the
four numbers, and two of its rows are in the wrong table.

| Metric required by section 11 | Column that would carry it | Computable today |
|---|---|---|
| REWORK rate (< 30%) | `Attempts` | yes |
| ESCALATE rate (< 10%) | none | **no** |
| Gate duration (< 10 min) | none (`Cycle duration` holds "1 session") | **no** |
| Pure-crate coverage (>= 80%) | none | **no** |

SPEC-0042 is the proof: its row reads `attempts=0` beside
`APPROVE (round 3)`, because `Attempts` counts REWORK rounds only and its three
ESCALATE rounds exist solely in prose. A retrospective run against this table
would print three dashes and one number, and would certify nothing.

Two rows are also structurally misplaced: SPEC-0045 and SPEC-0005 sit at lines
24-25, *below* the `## Process metrics` heading, so a five-column table is
holding seven-column rows. The SPEC-0045 cycle introduced it and the SPEC-0005
cycle repeated it; both are on `main`.

Separately, the SPEC-0005 cycle exposed that `docs/PRD.md:100` requires
`orcker link` in FR-003's AC1 while the spike delivered `orcker proxy add`.
The owner decided on 2026-08-31 to record FR-003 as **partial** rather than
amend the PRD, using the notation `specs/TRACEABILITY.md:16` already uses for
`FR-022 (partial)`. FR-003's AC1 closes with SPEC-0006.

## Requirements

- R1. `specs/TRACEABILITY.md` gains the columns section 11 needs to be
  answerable: ESCALATE rounds, gate duration, and pure-crate coverage, beside
  the existing `Attempts`.
- R2. Backfill carries **only what the ten cycle logs actually record**. A value
  that was never measured is written as `—`, never inferred, estimated or
  reconstructed. The retrospective is allowed to conclude "not measured"; it is
  not allowed to read an invented number.
- R3. The SPEC-0045 and SPEC-0005 rows move into the spec table, above the
  `## Process metrics` heading, in accepted order.
- R4. SPEC-0005's coverage reads `FR-003 (partial)` in `specs/TRACEABILITY.md`
  and in the `specs/ROADMAP.md` row, and the Phase-0 exit line states that
  FR-003's AC1 closes with SPEC-0006. `docs/PRD.md` is not touched.
- R5. `specs/ROADMAP.md`'s Phase-0 heading stops claiming the phase is "drafted,
  awaiting human approval" - eight of its ten rows are `accepted`.

## Design & contracts

Record-only. No crate, no binary, no dependency, no script. The new columns are
additive: no existing column is renamed or removed, so every prior row stays
readable and the eight untouched rows keep their content verbatim.

Backfill sources, in order of authority: `specs/logs/SPEC-*.md`, then the
verdict block quoted in each log. Anything absent from both is `—`.

## Test plan

- Unit: none. No code changes.
- Evidence (the acceptance is textual and mechanical): column count per row,
  row placement relative to the `## Process metrics` heading, and a diff showing
  the eight untouched rows carry only the new trailing cells.

## Acceptance checklist

- [ ] AC1 Every row in the spec table has the same column count as its header ->
      evidence: `awk -F'|' 'NF' ` count per row, all equal
- [ ] AC2 No row appears below the `## Process metrics` heading ->
      evidence: line number of the heading is greater than the last `| SPEC-` line
- [ ] AC3 Each of section 11's four metrics has a column, and every backfilled
      value is traceable to a cycle log or is `—` ->
      evidence: per-value citation table in `specs/logs/SPEC-0047.md`
- [ ] AC4 SPEC-0005 reads `FR-003 (partial)` in `TRACEABILITY.md` and
      `ROADMAP.md`; `git diff docs/PRD.md` is empty ->
      evidence: both greps plus the empty diff
- [ ] AC5 The Phase-0 heading and exit line in `ROADMAP.md` match reality ->
      evidence: heading text plus the FR-003/SPEC-0006 sentence
- [ ] AC6 `scripts/gate.sh specs/SPEC-0047-*.md` passes

## Out of scope

Running the retrospective itself (human, SDD section 11 - this spec only makes it
possible); editing `docs/PRD.md`; changing the front matter of any accepted spec;
adding a gate step that measures the new metrics automatically.

## Agent notes

The temptation to fix here is backfilling a plausible gate time or coverage
number for the older cycles. Do not. The SPEC-0005 cycle was sent back to REWORK
for exactly that class of error - a superseded measurement left standing in the
records the next cycle reads. `—` is a finding; a guess is a defect.
