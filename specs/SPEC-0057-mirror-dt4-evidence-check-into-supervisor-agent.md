---
id: SPEC-0057
title: Mirror DT4's evidence mirror-check into the supervisor agent definition
phase: 0
covers: [FR-001]
depends_on: [SPEC-0052]
surface:
  - .claude/agents/supervisor.md
status: draft
attempts: 0
---

## Context

SPEC-0052 added a mirror check to `docs/SDD.md` section 8.1's DT4 (reject an
evidence block that contradicts the tree; treat an unreplaced "re-run" claim as
that specific failure). Its surface was `docs/` only, so `.claude/agents/
supervisor.md` line 35 — the second, English copy of the DT table the real
supervisor subagent actually reads — still carries the pre-SPEC-0052 wording.
R4 is inert at runtime until this mirrors it, same pattern SPEC-0054 already
set for a `docs/SDD.md` -> `.claude/agents/supervisor.md` sync.

## Requirements

- R1. `.claude/agents/supervisor.md`'s DT4 row gains the same mirror-check
      clause `docs/SDD.md` section 8.1 carries.

## Acceptance checklist

- [ ] AC1 `grep -n 're-run' .claude/agents/supervisor.md` matches
- [ ] AC2 `scripts/gate.sh specs/SPEC-0057-*.md` passes
