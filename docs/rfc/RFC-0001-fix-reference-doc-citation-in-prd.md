---
id: RFC-0001
title: Cite the reference document by its real repository path in the PRD
target: docs/PRD.md line 14 (section 1, "Contexto e problema")
raised_by: SPEC-0042
status: open
---

## Why this is an RFC and not a diff

`CLAUDE.md` and `docs/PRD.md` section 12 both state that agents never edit the
PRD; requirement changes are proposed through `docs/rfc/`. SPEC-0042 corrects
every other citation of the reference document directly, but this one is inside
the PRD, so it is carried here for the owner to apply.

The change is editorial, not a requirement change: it repoints a citation at the
file it always meant. No `FR-`/`NFR-` id, wording or decision (D01-D12) moves.
It is a patch-level edit to a v1.0 document, not a minor bump.

## Current text

`docs/PRD.md:14` cites the document as `referenciadockerlaravel.md` - no
hyphens, no path:

> …gerando por projeto o stack de paridade de produção do documento de
> referência (`referenciadockerlaravel.md`): app com PHP-FPM + Supervisor…

No such file exists, and none ever did. The document lives at
`docs/referencia-docker-laravel.md`, and SPEC-0042 puts it under version control
(it was untracked until then). SPEC-0003 searched for the cited name, concluded
the reference document was absent from the repository, and transcribed its
requirements into that spec's own R3 instead.

## Proposed text

Replace the parenthetical with the real path:

> …gerando por projeto o stack de paridade de produção do documento de
> referência (`docs/referencia-docker-laravel.md`): app com PHP-FPM +
> Supervisor…

## Rationale

The PRD is read by agents as the product source of truth. A citation that
resolves to nothing costs a full spec cycle every time an agent trusts it, which
already happened once in SPEC-0003. With the document committed by SPEC-0042,
the path is stable and checkable.

## Verification once applied

```
grep -c 'docs/referencia-docker-laravel\.md' docs/PRD.md   # 1
grep -c 'referenciadockerlaravel\.md' docs/PRD.md          # 0
```
