---
id: SPEC-0062
title: Fix the PRD's remaining title-form citations of the never-committed viability analysis
phase: 0
covers: [FR-001]
depends_on: [SPEC-0044]
surface:
  - docs/
status: draft
attempts: 0
---

## Context

`docs/PRD.md` cites "análise de viabilidade" / "Análise de viabilidade Orcker
v1.1" by title (not filename) at lines 68, 89, 242 and 250 — the same document
SPEC-0044 found has never existed in this repository at any commit. Unlike
SPEC-0044's header-line swap, these are inline prose ("detalhes... na análise
de viabilidade", "roadmap da análise de viabilidade", "Detalhamento e
mitigação na análise de viabilidade, seção 8", plus a References-list entry),
so fixing them rewrites each surrounding sentence rather than repointing a
citation. `docs/PRD.md` is not agent-editable, so this also goes through
`docs/rfc/`; found during SPEC-0044's supervisor review, deliberately left out
of its diff.
