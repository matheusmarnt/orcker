---
id: SPEC-0043
title: Add the path-scoped instruction file for orcker-stack
phase: 0
covers: [FR-022]
depends_on: [SPEC-0003]
surface:
  - .github/
status: draft
attempts: 0
---

## Context

Every inherited crate has a `.github/instructions/<crate>.instructions.md` that
agents must read before editing it. SPEC-0003 created `crates/orcker-stack`
without one, because `.github/` sits outside that spec's surface. Add the file
(layer, owns, must-not, conventions, review checklist) mirroring
`orcker-core.instructions.md`, and do the same for `orcker-engine` and
`orcker-catalog` when those crates land.
