---
id: SPEC-0000
title: <one-line imperative, e.g. "Render dual-network compose file from typed model">
phase: 0            # PRD phase (0..3)
covers: [FR-000]    # PRD requirement ids this spec (fully or partially) implements
depends_on: []      # spec ids that must be `accepted` first
surface:            # ONLY path prefixes the diff may touch (specs/ and docs/ always allowed)
  - crates/orcker-stack/
status: draft       # draft -> approved -> in_progress -> implemented -> accepted
attempts: 0         # incremented on each REWORK; 3 => ESCALATE
---

## Context

Why this exists. Links to PRD sections and prior specs. Max 15 lines.

## Requirements

Numbered, testable statements (R1, R2, ...). No ambiguity left to the
implementer: anything undecided here MUST be resolved before `status: approved`.

## Design & contracts

Public signatures, types, trait definitions, error variants, IPC messages
(additive only), file formats. New dependencies MUST be listed here explicitly.
Pseudocode welcome. This section is the API review.

## Test plan

- Unit (pure, table-driven): <cases>
- Integration (side effects behind traits, tested with fakes): <cases>
- E2E / manual (only when unavoidable — must say why): <steps>

## Acceptance checklist

Every AC is objective and machine- or evidence-verifiable, and maps to >= 1 test:

- [ ] AC1 <statement> -> test: `<module>::<test_name>`
- [ ] AC2 <statement> -> evidence: `<command + expected output>`
- [ ] ACn `scripts/gate.sh specs/SPEC-0000-*.md` passes

## Out of scope

Explicit exclusions (prevents scope creep and guides the supervisor).

## Agent notes

Files to read first (keep minimal), known pitfalls, upstream (Yerd) references.
