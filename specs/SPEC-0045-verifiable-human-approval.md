---
id: SPEC-0045
title: Make the human-only approval verifiable in the committed record
phase: 0
covers: [FR-001]
depends_on: [SPEC-0042]
surface:
  - docs/
  - specs/
  - .claude/
status: accepted
attempts: 1
---

## Context

SPEC-0042 exposed a hole in the loop, and its supervisor named it in three
verdicts. `draft -> approved` is the one transition SDD section 5 reserves for
the human, but nothing requires it to exist in the committed record. SPEC-0042
reached `/spec-next` with `approved` present only as an uncommitted working-tree
edit of unknown authorship, while `HEAD` said `draft`.

The consequence is not cosmetic. The supervisor runs as a subagent: it sees the
repository, never the session. An approval the human gives mid-cycle reaches it
only as agent-authored prose in a cycle log, which is indistinguishable from an
approval the cycle invented. SPEC-0042's supervisor refused to ratify one and
was right to. A spec that enters the loop without a committed approval therefore
cannot be closed by the supervisor at all, no matter how the human rules.

The same supervisor flagged that `attempts = 3` never fired across its three
rounds, because SDD section 5 increments only on REWORK. That is defensible -
every ESCALATE is a handoff to the human, so the counter is not what bounds the
loop - but the argument holds only if each handoff is verifiably real. The two
findings are one defect, and R4 states the reasoning rather than adding a
counter nobody needs.

## Requirements

- **R1** SDD section 5 states that `draft -> approved` must exist in the
  committed record before `/spec-next` may select a spec, as a commit of its own
  whose diff touches nothing but that spec's `status` line and its
  `specs/ROADMAP.md` row.
- **R2** `.claude/commands/spec-next.md` step 2 selects on the committed status
  (`git show HEAD:<spec file>`), not on the working tree. A spec whose
  `approved` exists only as an uncommitted edit is reported as blocked, naming
  the reason, rather than selected.
- **R3** SDD section 8.1 gains a deterministic item, `DT9`: the supervisor
  verifies an approval commit for the spec exists in the branch history and
  precedes the implementation commit. Section 8.3 routes a failed DT9 to
  `ESCALATE`, not `REWORK` - the agent cannot fix it, only the human can.
- **R4** SDD section 5 states that `attempts` counts REWORK rounds only, that
  ESCALATE rounds are deliberately uncounted because each is a handoff to the
  human, and that this reasoning depends on R1-R3 making the handoff verifiable.
- **R5** No existing `accepted` spec is retroactively edited, and
  `specs/TRACEABILITY.md`'s `Attempts` column keeps its current meaning (its
  header already defines it as REWORK rounds).

## Design & contracts

No code, no crates, no dependencies. Three files change:
`docs/SDD.md` (sections 5, 8.1, 8.3), `.claude/commands/spec-next.md` (step 2
plus one line of loop discipline), and this spec.

The selection rule R2 mandates, stated once so both artifacts can quote it:

```
git show "HEAD:$spec" | awk -F': ' '/^status:/{print $2; exit}'
```

## Test plan

Process spec: no Rust, no test binary can observe a prompt file. Every AC is a
command whose output is quoted in `specs/logs/SPEC-0045.md`, run before (RED)
and after (GREEN).

- E2E / manual, unavoidable and stated as such: one negative reproduction on a
  throwaway spec file whose `approved` is uncommitted, confirming the documented
  selection rule reports it blocked while the old working-tree rule selects it.

## Acceptance checklist

- [x] AC1 (R1) SDD section 5 requires a committed approval → evidence:
      `grep -n 'draft -> approved' docs/SDD.md` shows the new sentence naming
      the standalone commit
- [x] AC2 (R2) the command selects on `HEAD` → evidence:
      `grep -n 'git show' .claude/commands/spec-next.md` returns step 2's rule
      (RED: no `git show` in the file)
- [x] AC3 (R2) negative reproduction → evidence: on a throwaway spec whose
      `approved` is written but uncommitted, the R2 rule prints `draft` while
      the working-tree read prints `approved`; both outputs quoted in the log
- [x] AC4 (R3) DT9 exists and routes to ESCALATE → evidence:
      `grep -n 'DT9' docs/SDD.md` returns a row in the 8.1 table and a line in
      the 8.3 decision block
- [x] AC5 (R4) the `attempts` semantics are stated → evidence:
      `grep -n 'attempts' docs/SDD.md` shows REWORK-only wording and the
      dependency on R1-R3
- [x] AC6 (R5) no accepted spec was touched → evidence:
      `git diff --name-only HEAD` lists no `specs/SPEC-00*.md` other than this
      one, and no change to `specs/TRACEABILITY.md`
- [x] AC7 `scripts/gate.sh specs/SPEC-0045-verifiable-human-approval.md` passes

## Out of scope

- A `scripts/gate.sh` step enforcing the approval commit. DT9 puts it in the
  supervisor's deterministic layer, which already runs on every cycle; a gate
  ratchet is a bigger change and would need its own spec. File one if the
  supervisor check proves too soft.
- Any new counter or `escalations:` front-matter field. R4 explains why the
  absence is correct rather than adding a field nobody reads.
- Retrofitting approval commits onto specs already `accepted`.
- Changing `/spec-verify` or the supervisor agent definition beyond the DT9 row.

## Agent notes

Read first: this file, `docs/SDD.md` sections 5, 8.1 and 8.3,
`.claude/commands/spec-next.md`, and `specs/logs/SPEC-0042.md` (which records
the three supervisor rounds this spec is a response to).

Pitfall: this spec is subject to its own R1. Its approval has to be a committed
act before `/spec-next` may select it, which is also the cleanest available
demonstration that the rule works.
