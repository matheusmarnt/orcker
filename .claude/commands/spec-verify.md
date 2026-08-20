---
description: Run the acceptance protocol for the current spec
argument-hint: [SPEC-ID]
---

1. Run `scripts/gate.sh specs/$ARGUMENTS*.md`. If it fails: print the failure, stop
   (back to S4).
2. Collect: the spec file, `git diff HEAD`, the gate output, the cycle log
   (`specs/logs/$ARGUMENTS.md`, containing the RED evidence).
3. Invoke the `supervisor` subagent with exactly that material.
4. Print the supervisor's verdict block verbatim. On APPROVE proceed to S8 (commit,
   statuses, `specs/TRACEABILITY.md`). On REWORK increment `attempts` and return to
   S4. On ESCALATE stop for the human.

S8 — close the cycle (only after APPROVE):

- One atomic commit, Conventional Commits with the crate scope, body referencing the
  SPEC and FR ids. Squash first if the cycle iterated.
- Never add a Claude Code, Claude, Anthropic or AI co-author trailer to the commit.
- Set the spec status to `implemented`, then `accepted` once the commit exists.
- Append the row to `specs/TRACEABILITY.md` (spec, covers, attempts, cycle duration,
  final verdict, key tests, commit).
- Record any deviation or trade-off in `specs/DECISIONS.md`.
- Never `git push`, never merge, never release — those are the human's acts.
