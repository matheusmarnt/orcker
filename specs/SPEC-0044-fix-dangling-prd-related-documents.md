---
id: SPEC-0044
title: Repoint the two dangling related-document citations in the PRD header
phase: 0
covers: [FR-001]
depends_on: [SPEC-0042]
surface:
  - docs/
  - specs/
status: in_progress
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

Resolved with the human owner: no checkout of `orcker-analise-viabilidade.md`
has ever existed in this repository's history (verified via
`git log --all --full-history`), so the citation is dropped, not imported. The
RFC carries this as a proposal the owner can still override by supplying the
document later.

## Requirements

- **R1** The `orcker-sdd.md` citation at `docs/PRD.md:5` is repointed to its
  real repository path, `docs/SDD.md`, carried via RFC (R3).
- **R2** The `orcker-analise-viabilidade.md` (v1.1) citation at `docs/PRD.md:5`
  is dropped from the "Documentos relacionados" field, carried via the same
  RFC, since no such file exists or has ever existed in this repository.
- **R3** `docs/PRD.md` is not edited by this diff. Both corrections are carried
  to the human as one RFC, `docs/rfc/RFC-0002-*.md`, targeting line 5, which
  quotes the current line, gives the corrected line, and states that applying
  it is the human's act.
- **R4** After the diff, no file *cites* either wrong name
  (`orcker-analise-viabilidade.md`, `orcker-sdd.md`) in a citation role. The
  strings may still occur, but only in two roles, neither of which is a
  citation: in `docs/PRD.md`, which an agent may not edit and which R3 hands to
  the owner instead; and as quoted subject matter in prose *about* this defect
  (this spec, the RFC, the cycle log, `specs/DECISIONS.md`). This is
  deliberately a rule and not a file list (SPEC-0042 R4 found the list form
  breaks under its own artifacts).
- **R5** Historical records are not rewritten. `specs/logs/*.md` and
  `specs/TRACEABILITY.md` state what was true when a past cycle closed and keep
  their current wording.

## Design & contracts

No code, no crates, no dependencies. File operations:

| File | Operation |
|------|-----------|
| `docs/rfc/RFC-0002-fix-prd-header-related-documents.md` | new, per R3 |
| `specs/SPEC-0044-fix-dangling-prd-related-documents.md` | this file (Requirements/AC, status) |
| `specs/SPEC-0061-fix-dangling-sdd-related-documents.md` | new, 3-line draft, found-and-not-done |
| `specs/ROADMAP.md` | add the SPEC-0061 row |
| `specs/DECISIONS.md` | new entry |

RFC front matter follows the RFC-0001 shape: `id`, `title`, `target` (the PRD
line), `raised_by` (this spec id), `status: open`; then `Current text` /
`Proposed text` / `Rationale` / `Verification once applied`.

## Test plan

Docs-only spec: no unit or integration surface exists. Every AC is verified by
a command whose output is quoted in `specs/logs/SPEC-0044.md`, run before the
fix (RED) and after (GREEN).

- E2E / manual (unavoidable, and stated as such): the `git`/`grep` invocations
  listed in the acceptance checklist. There is no test binary that can observe
  repository-wide prose.

## Acceptance checklist

- [ ] AC1 (R3) the PRD is untouched and the RFC exists → evidence:
      `git diff --exit-code HEAD -- docs/PRD.md` exits 0, and
      `docs/rfc/RFC-0002-fix-prd-header-related-documents.md` is present
- [ ] AC2 (R1) the RFC's proposed text repoints the SDD citation to the real
      path → evidence: `grep -c 'docs/SDD\.md' docs/rfc/RFC-0002-*.md` is ≥1
- [ ] AC3 (R2) the RFC's proposed text drops the viability-analysis citation →
      evidence: the RFC's "Proposed text" section (isolated by
      `sed -n '/## Proposed text/,/## Rationale/p'`) does not match
      `orcker-analise-viabilidade`
- [ ] AC4 (R4) no live file cites either wrong name → evidence:
      `grep -rn 'orcker-analise-viabilidade\.md\|orcker-sdd\.md' --include='*.md' .`
      lists only `docs/PRD.md` (R3, owner-applied), this spec, the RFC, the
      cycle log and `specs/DECISIONS.md` — no other survivor
- [ ] AC5 `scripts/gate.sh specs/SPEC-0044-fix-dangling-prd-related-documents.md`
      passes

## Out of scope

- Applying the RFC to `docs/PRD.md` — R3 forbids it; the human owner's act.
- Fixing `docs/SDD.md`'s own duplicate citations (header line 4, line 495) —
  same defect class, different file, filed as SPEC-0061 instead.
- `README-INSTALL.md`'s install-bundle template naming of `orcker-sdd.md` /
  `orcker-prd.md` — a portable-bundle convention pointing at `docs/prd-sdd/`,
  not a citation into this repo's doc tree; not the same defect.
- Rewriting `specs/TRACEABILITY.md` or `specs/logs/*.md` — R5 forbids it.

## Agent notes

Read first: this file, `CLAUDE.md` (the "never edit `docs/PRD.md`" rule),
`docs/SDD.md` section 4 (RFC format precedent) and `docs/rfc/RFC-0001-*.md`
(the template to copy). No crate instruction file applies — the diff touches
no Rust.
