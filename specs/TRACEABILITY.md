# Traceability matrix — FR <-> spec <-> tests <-> commit

Updated at S8 of every accepted cycle (see docs/SDD.md sections 6 and 11).
One row per accepted spec. `Attempts` counts REWORK rounds (0 = first pass).
The `Commit` column names the commit subject: the hash cannot be recorded inside
the commit that this row is part of.

| Spec | Covers | Attempts | Cycle duration | Final verdict | Key tests | Commit |
|------|--------|----------|----------------|---------------|-----------|--------|
| SPEC-0001-fork-bootstrap | FR-001 | 0 | 1 session (2026-08-22) | APPROVE | inherited suite, 2602 passed / 0 failed; repaired `orcker_platform::pure::dns_probe::tests::query_encodes_probe_name_and_a_question` and `orckerd::self_update::tests::current_version_parses` | `chore(workspace)!: rebrand the Yerd fork as Orcker` on `feat/SPEC-0001-fork-bootstrap` |

## Process metrics (reviewed every 10 accepted specs — SDD section 11)

| Window | REWORK rate (target < 30%) | ESCALATE rate (target < 10%) | Median gate time | Pure-crate coverage (target >= 80%) |
|--------|---------------------------|------------------------------|------------------|--------------------------------------|
| — | — | — | — | — |
