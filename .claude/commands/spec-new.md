---
description: Draft a new spec from a PRD requirement
argument-hint: FR-xxx [FR-yyy ...]
---

Draft a spec covering $ARGUMENTS.

1. Invoke the `spec-writer` subagent with the requirement ids. It reads `docs/PRD.md`
   and `specs/_TEMPLATE.md` and writes `specs/SPEC-XXXX-<slug>.md` with the next free
   number and `status: draft`.
2. Add the row to the right phase table in `specs/ROADMAP.md` (spec, status, covers,
   depends_on), keeping the table and the front matter in sync.
3. Report the path and stop. `draft -> approved` is the human's sign-off (SDD
   section 5) — never flip it yourself, and never start implementing the new spec in
   this session.

Also use this for work discovered mid-cycle: a 3-line `draft` spec parks it without
expanding the current diff (SDD section 6, anti-drift rule 1).
