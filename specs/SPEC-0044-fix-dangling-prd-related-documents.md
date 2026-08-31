---
id: SPEC-0044
title: Repoint the two dangling related-document citations in the PRD header
phase: 0
covers: [FR-001]
depends_on: [SPEC-0042]
surface:
  - docs/
  - specs/
status: draft
attempts: 0
---

## Context

`docs/PRD.md:5` cites `orcker-analise-viabilidade.md` (v1.1) and `orcker-sdd.md`
as related documents. Neither path exists anywhere in the repository: the second
is `docs/SDD.md`, and the first appears never to have been imported at all.
Same defect class as SPEC-0042's, different files; found during that cycle and
deliberately left out of its diff. The PRD cannot be edited by an agent, so this
also goes through `docs/rfc/`, and the viability analysis has to be imported or
the citation dropped.
