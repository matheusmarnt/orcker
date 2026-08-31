---
id: SPEC-0042
title: Commit the docker/Laravel reference document and fix every citation of it
phase: 0
covers: [FR-022]
depends_on: [SPEC-0001]
surface:
  - docs/
  - specs/
status: approved
attempts: 0
---

## Context

`docs/referencia-docker-laravel.md` (405 lines) is the source of truth for the
generated stack, but it is **untracked**: it exists on one machine, is not
ignored, and a `git clean -fd` removes it. CI and every other checkout do not
have it.

Every citation in the repository also names it wrongly, as
`referenciadockerlaravel.md` with no hyphens and no path: `docs/PRD.md:14`,
`specs/SPEC-0003-stack-compose-renderer.md:20`,
`specs/SPEC-0005-proxy-container-spike.md:21` and this file. That is why
SPEC-0003 concluded the document was absent and worked from its own R3
transcription instead.

Commit the file and repoint the citations. `docs/PRD.md` is never edited
directly (CLAUDE.md), so the PRD citation goes through `docs/rfc/`.

## Known divergence to settle in SPEC-0007

The reference document names the project-internal network `app-network` and the
database volume `pgsql-data`. SPEC-0003 renders them per-site as `{site}` and
`{site}-db-data`. Both are defensible - compose already namespaces networks and
volumes by project - and SPEC-0003's R3 named neither, so this is not a defect
in an accepted spec. SPEC-0007 covers parity with the reference document and has
to choose deliberately rather than inherit the difference by accident.

## Requirements

- **R1** `docs/referencia-docker-laravel.md` is tracked by git at exactly that
  path, byte-identical to the untracked working copy this cycle started from.
- **R2** Every *live* citation of the reference document names it by its real
  repository path, `docs/referencia-docker-laravel.md`. The live citations are
  `specs/SPEC-0003-stack-compose-renderer.md`,
  `specs/SPEC-0005-proxy-container-spike.md` and this spec's own Context.
- **R3** `docs/PRD.md` is not edited by this diff. The wrong citation at
  `docs/PRD.md:14` is carried to the human as `docs/rfc/RFC-0001-*.md`, which
  quotes the current line, gives the corrected line, and states that the PRD
  bump is the human's act.
- **R4** After the diff, no file *cites* the reference document as
  `referenciadockerlaravel.md`. The string may still occur, but only in two
  roles, neither of which is a citation: in `docs/PRD.md`, which an agent may
  not edit and which R3 hands to the owner instead; and as quoted subject matter
  in prose *about* this defect - the RFC that proposes the correction, the specs
  and cycle logs that describe it, the decision entries that record it, and the
  historical records R5 protects.
  This requirement is deliberately a rule and not a list of files. Earlier
  drafts enumerated the permitted files and broke three times running (at S3, at
  S4, and again when a supervisor-requested `specs/DECISIONS.md` entry became an
  unlisted survivor), because every artifact this cycle writes about the defect
  has to quote it.
- **R5** Historical records are not rewritten. `specs/TRACEABILITY.md` and
  `specs/logs/*.md` state what was true when a past cycle closed; correcting
  them would falsify the record, so both keep their current wording.

## Design & contracts

No code, no crates, no dependencies. Four file operations:

| File | Operation |
|------|-----------|
| `docs/referencia-docker-laravel.md` | `git add` (content unchanged) |
| `docs/rfc/RFC-0001-fix-reference-doc-citation-in-prd.md` | new, per R3 |
| `specs/SPEC-0003-stack-compose-renderer.md` | citation repointed (R2) |
| `specs/SPEC-0005-proxy-container-spike.md` | citation repointed (R2) |
| `specs/SPEC-0042-restore-missing-reference-doc.md` | this file (R2, status) |

RFC format (first of its kind, so it is defined here): front matter
`id`, `title`, `target` (the PRD line), `raised_by` (this spec id), `status:
open`; then `Current text`, `Proposed text`, `Rationale`.

## Test plan

Docs-only spec: no unit or integration surface exists. Every AC is verified by a
command whose output is quoted in `specs/logs/SPEC-0042.md`, run before the fix
(RED) and after (GREEN).

- E2E / manual (unavoidable, and stated as such): the four `git`/`grep`
  invocations listed in the acceptance checklist. There is no test binary that
  can observe git tracking state or repository-wide prose.

## Acceptance checklist

- [ ] AC1 (R1) the reference doc is tracked → evidence:
      `git ls-files --error-unmatch docs/referencia-docker-laravel.md` exits 0
- [ ] AC2 (R1) tracked content matches the working copy → evidence:
      `git diff --exit-code -- docs/referencia-docker-laravel.md` exits 0 and
      `git show :docs/referencia-docker-laravel.md > /dev/null` exits 0 over a
      405-line blob
- [ ] AC3 (R2, R4) the two live citations no longer use the wrong name, and the
      PRD one is untouched → evidence:
      `grep -c 'referenciadockerlaravel\.md' specs/SPEC-0003-stack-compose-renderer.md specs/SPEC-0005-proxy-container-spike.md`
      prints `0` for both, and the same `grep -c` over `docs/PRD.md` prints `1`,
      proving R3 was routed to the RFC rather than applied. Remaining
      occurrences elsewhere are prose *about* the defect, which R4 permits by
      role; they are a review judgment, not an enumeration this AC pins
- [ ] AC4 (R2) the two live specs cite the real path → evidence:
      `grep -l 'docs/referencia-docker-laravel\.md' specs/SPEC-0003-stack-compose-renderer.md specs/SPEC-0005-proxy-container-spike.md`
      lists both files, and that path exists on disk
- [ ] AC5 (R3) the PRD is untouched and the RFC exists → evidence:
      `git diff --exit-code HEAD -- docs/PRD.md` exits 0, and
      `docs/rfc/RFC-0001-fix-reference-doc-citation-in-prd.md` is present
- [ ] AC6 `scripts/gate.sh specs/SPEC-0042-restore-missing-reference-doc.md`
      passes

## Out of scope

- Moving the document under `docs/reference/`. The approved Context commits it
  where it already lives; a move would break the very citations this spec fixes.
- Translating the document (it is Portuguese; so are `docs/PRD.md` and
  `docs/SDD.md`).
- Adding it to the VitePress sidebar, or any docs-site work.
- Editing `docs/PRD.md` — R3 forbids it.
- Rewriting `specs/TRACEABILITY.md` or `specs/logs/*.md` — R5 forbids it.
- Settling the `app-network` / `pgsql-data` divergence; that is SPEC-0007's.

## Agent notes

Read first: this file, `CLAUDE.md` (the "never edit `docs/PRD.md`" rule),
`docs/SDD.md` section 3 (where `docs/rfc/` belongs). No crate instruction file
applies — the diff touches no Rust.

Pitfall: `docs/rfc/` does not exist yet, so this cycle creates it and writes the
first RFC; there is no template to copy.
