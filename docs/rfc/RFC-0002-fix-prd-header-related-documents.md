---
id: RFC-0002
title: Repoint and drop the dangling citations in the PRD's related-documents field
target: docs/PRD.md line 5 (header, "Documentos relacionados")
raised_by: SPEC-0044
status: open
---

## Why this is an RFC and not a diff

`CLAUDE.md` and `docs/PRD.md` section 12 both state that agents never edit the
PRD; requirement changes are proposed through `docs/rfc/`. Both corrections
below are editorial (no `FR-`/`NFR-` id, wording or decision moves), but they
still touch the PRD header, so they are carried here for the owner to apply.

## Current text

`docs/PRD.md:5` cites two related documents that resolve to nothing:

> **Owner:** Matheus Mariano · **Documentos relacionados:**
> `orcker-analise-viabilidade.md` (v1.1), `orcker-sdd.md`

Neither path exists in the repository. `orcker-sdd.md` is a wrong name for a
file that does exist, `docs/SDD.md`. `orcker-analise-viabilidade.md` (v1.1) has
never existed in this repository at any commit (checked with
`git log --all --full-history`) — the same finding SPEC-0042's cycle log
recorded when it deferred this defect.

## Proposed text

Repoint the SDD citation to its real path, and drop the viability-analysis
citation rather than cite a document that was never committed:

> **Owner:** Matheus Mariano · **Documentos relacionados:** `docs/SDD.md`

## Rationale

The PRD is read by agents as the product source of truth. A citation that
resolves to nothing costs a full spec cycle every time an agent trusts it
(SPEC-0042's own genesis). `docs/SDD.md` is a stable, checkable path, so
repointing it removes that cost. The viability-analysis citation has no target
to repoint to — nothing by that name has ever been committed here — so the
honest fix is to drop it rather than invent or guess at a replacement path.

**Owner override:** if a "análise de viabilidade" document does exist outside
this repository and should be cited, importing it under version control (the
same way SPEC-0042 committed `docs/referencia-docker-laravel.md`) and citing
its real path is a straightforward alternative to dropping the citation — that
call belongs to the document's owner, not to this RFC.

## Known related mentions, out of scope

`docs/PRD.md` cites the same phantom document by title, not filename, in four
more places this RFC's target (line 5) does not reach: lines 68 and 89 ("na
análise de viabilidade" / "roadmap da análise de viabilidade"), line 242
("Detalhamento e mitigação na análise de viabilidade, seção 8"), and line 250
("Análise de viabilidade Orcker v1.1 (documento irmão)", in the References
list). Repointing or dropping those needs a rewrite of the surrounding prose
in each case, not a one-line swap, so it is deliberately not folded into this
RFC — filed instead as `specs/SPEC-0062-fix-prd-viability-analysis-prose-citations.md`.
Applying this RFC alone leaves those four mentions standing; do not read
`docs/PRD.md`'s "análise de viabilidade" citation as fully resolved until
SPEC-0062 is too.

## Verification once applied

```
grep -c 'docs/SDD\.md' docs/PRD.md                     # 1
grep -c 'orcker-sdd\.md' docs/PRD.md                    # 0
grep -c 'orcker-analise-viabilidade\.md' docs/PRD.md    # 0
```
