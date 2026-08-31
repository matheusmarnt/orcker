---
id: SPEC-0048
title: Make the pure-crate coverage metric measurable
phase: 0
covers: [FR-001]
depends_on: [SPEC-0047]
surface:
  - scripts/
  - .github/
  - docs/
status: draft
attempts: 0
---

## Context

SPEC-0047 added the `Pure-crate coverage` column to `specs/TRACEABILITY.md` so
the SDD section 11 retrospective can answer its fourth metric, and required
cycles from that point on to fill it in. SPEC-0004 could not: no coverage tool
is installed and adding one is outside any spec's surface, so the column is
`—` again for the first cycle that shipped a pure crate. Pick a measurement
(`cargo llvm-cov` is the obvious candidate), wire it into `scripts/gate.sh` or a
sibling script as a reported number rather than a gate failure, and state in
`docs/SDD.md` how a cycle reads it at S8.
