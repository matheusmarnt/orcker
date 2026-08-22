---
id: SPEC-0030
title: Fix the stale daemon run command in the building guide
phase: 0
covers: [FR-001]
depends_on: [SPEC-0001]
surface:
  - docs/
status: draft
attempts: 0
---

## Context

`docs/developer/building.md` tells the reader to start the daemon with
`cargo run -p orckerd -- -v`. `orckerd` accepts no `-v`: it exits with
`error: unexpected argument '-v' found`. The only subcommand is `serve`.
Inherited from upstream and unrelated to the rename, found while gathering AC2
evidence for SPEC-0001.

## Requirements

- R1. Replace every `orckerd -- -v` occurrence with `orckerd -- serve` and
  verify each command block in the guide actually runs.
