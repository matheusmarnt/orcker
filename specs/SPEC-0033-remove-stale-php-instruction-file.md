---
id: SPEC-0033
title: Remove the stale path-scoped instruction file for the deleted PHP crate
phase: 0
covers: [FR-002]
depends_on: [SPEC-0002]
surface:
  - .github/
status: accepted
attempts: 1
---

## Context

`.github/instructions/orcker-php.instructions.md` is the path-scoped instruction
file for `crates/orcker-php`, which SPEC-0002 deletes. `.github/` is outside
SPEC-0002's surface, so the file survives the deletion and keeps telling agents
how to edit a crate that no longer exists. Found at S2 of SPEC-0002.

## Requirements

- R1. Delete `.github/instructions/orcker-php.instructions.md` and any
  `applyTo` glob elsewhere in `.github/instructions/` that points at
  `crates/orcker-php/` or `crates/orcker-services/`.
- R2. Check the remaining instruction files for prose describing the native
  runtime (FPM pools, native DB engines) and correct it or drop it.

## Acceptance checklist

- [ ] AC1 `rg -n "orcker-php|orcker-services|orcker-supervise" .github/` returns
      no matches
- [ ] AC2 `scripts/gate.sh specs/SPEC-0033-*.md` passes
