---
id: SPEC-0035
title: Retire the orcker.toml sections that only the native runtime read
phase: 0
covers: [FR-002]
depends_on: [SPEC-0002]
surface:
  - crates/orcker-core/
  - crates/orcker-config/
status: draft
attempts: 0
---

## Context

SPEC-0002 R6 says to delete pool/runtime logic "that has no future consumer".
`crates/orcker-core/src/php_pool.rs` still has one: `orcker-config` reads it for
the `[php.pool]` section of `orcker.toml`. The section is now write-only - the
FPM manager that consumed it is gone - but deleting it changes a user-facing
config contract that SPEC-0002 says nothing about, and would edit
`orcker-config`'s golden byte-shape test. Left standing at SPEC-0002 S4.

The same holds for `[services]` and the service-directive registry.

## Requirements

- R1. Decide, and record, what happens to an existing `orcker.toml` carrying
  `[php.pool]` / `[services]`: silently ignored, warned about once, or migrated.
- R2. Delete `php_pool.rs` and the schema/parse code that only serves it once R1
  is settled.
- R3. Update `crates/orcker-config/tests/toml_byte_shape.rs` in the same change;
  it is a contract test, so the diff needs the same justification the
  wire-stability reset got.

## Acceptance checklist

- [ ] AC1 A config file with the removed sections still loads, with the
      behaviour R1 chose, proved by a test
- [ ] AC2 `rg -n "php_pool" crates` returns no matches
- [ ] AC3 `scripts/gate.sh specs/SPEC-0035-*.md` passes
