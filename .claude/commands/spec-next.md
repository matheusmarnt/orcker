---
description: Select and start the next spec from the queue
---

1. Read `specs/ROADMAP.md` and the front matter of every spec listed there.
2. Pick the first spec with status `approved` whose `depends_on` are all `accepted`.
   If none: report the blocking chain and stop.
3. Set its status to `in_progress`, create branch `feat/<spec-id>-<slug>`.
4. Enter plan mode; read ONLY the spec, `CLAUDE.md`, the crate instruction files and
   the files in `surface`. Produce the S2 plan and start the loop at S3 (test-first).

Loop discipline (SDD sections 6 and 9.5):

- One spec per session. `/clear` when the cycle closes.
- S3 comes before S4: write the acceptance tests, run them, and record the RED
  evidence in `specs/logs/<spec-id>.md`. Without it the supervisor treats the tests
  as suspected tautologies (DT4).
- The diff must stay inside `surface`. Work discovered mid-cycle becomes a 3-line
  `draft` spec — never an expansion of the current diff.
- Never weaken `scripts/gate.sh`, the workspace lints, the wire-stability tests or
  any existing test to make the gate pass. That is an automatic REWORK (DT7).
- Product ambiguity or a spec that contradicts the code: stop and ESCALATE with a
  diagnosis. Do not improvise a product decision.
