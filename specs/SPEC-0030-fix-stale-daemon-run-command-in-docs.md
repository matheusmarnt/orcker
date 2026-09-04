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
`cargo run -p orckerd -- -v`. `orckerd` accepts no top-level `-v`: it exits
with `error: unexpected argument '-v' found`, `Usage: orckerd [COMMAND]`.
Inherited from upstream and unrelated to the rename, found while gathering
AC2 evidence for SPEC-0001. Re-confirmed 2026-09-04 while manually verifying
SPEC-0046's AC3: `bin/orckerd/src/args.rs` has since gained an explicit
`serve` subcommand (`Cli.command: Option<Command>`), and `verbose` lives on
`ServeArgs`, not the top-level `Cli` — so the fix is `orckerd -- serve -v`,
not a bare `orckerd -- serve`. Dropping `-v` (the earlier framing here) would
silence the verbose logging the command was written to enable.

## Requirements

- R1. Replace every `orckerd -- -v` occurrence with `orckerd -- serve -v`
  and verify each command block in the guide actually runs.
