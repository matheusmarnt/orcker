---
id: SPEC-0044
title: Repoint the two dangling related-document citations in the PRD header
phase: 0
covers: [FR-001]
depends_on: [SPEC-0042]
surface:
  - docs/
  - specs/
status: accepted
attempts: 3
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
- **R4** This diff introduces no *new* citation of either wrong name
  (`orcker-analise-viabilidade.md`, `orcker-sdd.md`) or of the same phantom
  document by its title form ("análise de viabilidade"). After the diff, every
  occurrence falls into one of four roles: the `docs/PRD.md` citations
  themselves (line 5, and the title-form mentions at lines 68/89/242/250 —
  none of which this diff edits; an agent may not edit the PRD, and only line 5
  is this spec's target, per R3/Out of scope), which R3 hands to the owner
  instead; quoted subject matter in prose *about* this defect (this spec, the
  RFC, this cycle log, `specs/DECISIONS.md`, `specs/SPEC-0061-*.md`,
  `specs/SPEC-0062-*.md`); the pre-existing, out-of-*scope* (both are inside
  the declared surface `docs/`/`specs/`, just not this diff's target)
  occurrences in `docs/SDD.md` (line 4, line 495) and `specs/logs/SPEC-0042.md`,
  neither of which this diff touches or is scoped to fix (the former is
  SPEC-0061's target; the latter is R5-protected history); and
  `README-INSTALL.md`, genuinely outside the declared surface (repo root) and
  a different, unrelated naming convention besides. This is deliberately a rule
  enumerating every role, not a file count, because SPEC-0042's own R4 found a
  bare file list breaks under its own artifacts — and because this spec's own
  first attempt at that file list omitted `specs/SPEC-0061-*.md` and
  miscounted `this spec` as a new occurrence when it was already a survivor
  before this cycle (supervisor round 1, AC4).
- **R5** Historical records are not rewritten. `specs/logs/*.md` and
  `specs/TRACEABILITY.md` state what was true when a past cycle closed and keep
  their current wording.

## Design & contracts

No code, no crates, no dependencies. File operations:

| File | Operation |
|------|-----------|
| `docs/rfc/RFC-0002-fix-prd-header-related-documents.md` | new, per R3; also discloses the line 68/89/242/250 title-form mentions as out of its scope |
| `specs/SPEC-0044-fix-dangling-prd-related-documents.md` | this file (Requirements/AC, status) |
| `specs/SPEC-0061-fix-dangling-sdd-related-documents.md` | new, 3-line draft, found-and-not-done |
| `specs/SPEC-0062-fix-prd-viability-analysis-prose-citations.md` | new, 3-line draft, found-and-not-done (supervisor round 1) |
| `specs/ROADMAP.md` | add the SPEC-0061 and SPEC-0062 rows |
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
      path → evidence: the RFC's "Proposed text" section (isolated by
      `sed -n '/## Proposed text/,/## Rationale/p'`) matches `docs/SDD\.md` at
      least once. Scoped to that section, not the whole file — the unscoped
      `grep -c` over the full RFC also matches "Current text"/"Rationale"
      prose and cannot fail (supervisor round 1, JG5)
- [ ] AC3 (R2) the RFC's proposed text drops the viability-analysis citation →
      evidence: the same "Proposed text" section does not match
      `orcker-analise-viabilidade`
- [ ] AC4 (R4) this diff introduces no new occurrence of either wrong name, and
      correctly enumerates every survivor → evidence: `grep -rln
      'orcker-analise-viabilidade\.md\|orcker-sdd\.md' --include='*.md' .`
      before this diff lists `README-INSTALL.md`, this spec, `docs/SDD.md`,
      `docs/PRD.md`, `specs/logs/SPEC-0042.md`; after this diff it additionally
      lists exactly `docs/rfc/RFC-0002-*.md`, `specs/DECISIONS.md`, this cycle
      log and `specs/SPEC-0061-*.md` — four additions, not five, and neither
      `this spec` nor `specs/SPEC-0062-*.md` is one of them: the former already
      carried the strings before this cycle, the latter discusses the same
      defect only in its title form ("análise de viabilidade", no `.md`, no
      `orcker-` prefix) so this filename-scoped grep does not and should not
      match it (supervisor round 1 caught the `this spec`/count errors; the
      `SPEC-0062` omission was caught re-deriving this AC's own evidence before
      resubmitting). A second, broader evidence pass covers R4's title-form
      clause specifically: `grep -rli 'viabilidade' --include='*.md' .` before
      this diff lists `docs/PRD.md`, `docs/SDD.md`, `specs/logs/SPEC-0042.md`,
      this spec (4 files — a subset of the filename-scoped RED above, since
      "viabilidade" is a substring of the wrong filename too); after this diff
      it additionally lists `docs/rfc/RFC-0002-*.md`, `specs/DECISIONS.md`,
      this cycle log, `specs/SPEC-0061-*.md` **and** `specs/SPEC-0062-*.md` —
      5 additions, 9 total, `SPEC-0062` correctly present this time because
      this check is not filename-scoped (supervisor round 2, R4/AC4 title-form
      coverage gap)
- [ ] AC5 `scripts/gate.sh specs/SPEC-0044-fix-dangling-prd-related-documents.md`
      passes

FR acceptance: FR-001 has AC1/AC2/AC3 (`docs/PRD.md:93-94`). AC2 (`orcker ping`)
and AC3 (LICENSE/README lineage) are closed by SPEC-0001. AC1 (`cargo fmt` /
`clippy -D warnings` / `cargo test --workspace` green) is this cycle's own
gate, closed by AC5 above.

## Out of scope

- Applying the RFC to `docs/PRD.md` — R3 forbids it; the human owner's act.
- Fixing `docs/SDD.md`'s own duplicate citations (header line 4, line 495) —
  same defect class, different file, filed as SPEC-0061 instead.
- Fixing `docs/PRD.md`'s other, title-form mentions of the same phantom
  document at lines 68, 89, 242 and 250 ("análise de viabilidade" / "Análise de
  viabilidade Orcker v1.1") — same document, different (non-header) citations
  requiring their own RFC and, unlike a filename swap, a rewrite of the prose
  around each mention; found during supervisor round 1, filed as SPEC-0062.
- `README-INSTALL.md`'s install-bundle template naming of `orcker-sdd.md` /
  `orcker-prd.md` — a portable-bundle convention pointing at `docs/prd-sdd/`,
  not a citation into this repo's doc tree; not the same defect.
- Rewriting `specs/TRACEABILITY.md` or `specs/logs/*.md` — R5 forbids it.

## Agent notes

Read first: this file, `CLAUDE.md` (the "never edit `docs/PRD.md`" rule),
`docs/SDD.md` section 4 (RFC format precedent) and `docs/rfc/RFC-0001-*.md`
(the template to copy). No crate instruction file applies — the diff touches
no Rust.
