# Traceability matrix — FR <-> spec <-> tests <-> commit

Updated at S8 of every accepted cycle (see docs/SDD.md sections 6 and 11).
One row per accepted spec. `Attempts` counts REWORK rounds (0 = first pass).
The `Commit` column names the commit subject: the hash cannot be recorded inside
the commit that this row is part of.

| Spec | Covers | Attempts | Cycle duration | Final verdict | Key tests | Commit |
|------|--------|----------|----------------|---------------|-----------|--------|
| SPEC-0001-fork-bootstrap | FR-001 | 0 | 1 session (2026-08-22) | APPROVE | inherited suite, 2602 passed / 0 failed; repaired `orcker_platform::pure::dns_probe::tests::query_encodes_probe_name_and_a_question` and `orckerd::self_update::tests::current_version_parses` | `chore(workspace)!: rebrand the Yerd fork as Orcker` on `feat/SPEC-0001-fork-bootstrap` |
| SPEC-0032-pin-gate-sort-collation | FR-001 | 0 | 1 session (2026-08-22) | APPROVE | no Rust changed; the gate itself is the test: `scripts/gate.sh` run twice on one tree, `LC_ALL=pt_BR.UTF-8` and `LC_ALL=C`, exit 0 both, 89 `test result: ok` lines each. Negative reproduction: a bare `pt_BR` sort still fails the step-5 diff | `fix(scripts): pin the gate clippy-allow sort to the C collation` on `feat/SPEC-0001-fork-bootstrap` |

## Process metrics (reviewed every 10 accepted specs — SDD section 11)

| Window | REWORK rate (target < 30%) | ESCALATE rate (target < 10%) | Median gate time | Pure-crate coverage (target >= 80%) |
|--------|---------------------------|------------------------------|------------------|--------------------------------------|
| — | — | — | — | — |
