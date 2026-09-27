---
id: SPEC-0061
title: Fix the dangling related-document citations inside the SDD's own header and references
phase: 0
covers: [FR-001]
depends_on: [SPEC-0044]
surface:
  - docs/
status: draft
attempts: 0
---

## Context

`docs/SDD.md:4` cites `orcker-prd.md` (v1.0) and `orcker-analise-viabilidade.md`
(v1.1); `docs/SDD.md:495` cites the latter again. The first is a wrong name for
`docs/PRD.md` (exists); the second has never existed in this repository at any
commit (same finding as SPEC-0044/SPEC-0042). Same defect class as SPEC-0044,
found while fixing it, deliberately left out of that diff — but unlike
`docs/PRD.md`, `docs/SDD.md` is not off-limits to agent edits, so this one does
not need an RFC.
