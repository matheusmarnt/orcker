---
id: SPEC-0056
title: Fix the stale daemon serve subcommand in the building guide
phase: 0
covers: [FR-001]
depends_on: [SPEC-0001]
surface:
  - docs/
status: draft
attempts: 0
---

## Context

`docs/developer/building.md:230` documents starting the daemon from source as
`cargo run -p orckerd -- -v`. This now fails: `error: unexpected argument
'-v' found` (`Usage: orckerd [COMMAND]`). `bin/orckerd/src/args.rs` added a
`serve` subcommand: `Cli.command` is now `Option<Command>`
(`#[command(subcommand)]`), and `verbose` (`-v`/`-vv`) moved onto
`ServeArgs`, i.e. it now belongs to `serve`, not the top-level `Cli`.
Correct invocation today: `cargo run -p orckerd -- serve -v`. Found while
manually verifying SPEC-0046's AC3; unrelated to that spec's own scope.

## Requirements

- R1. Update every `docs/developer/building.md` daemon-from-source run
  command so it matches the current `orckerd` CLI (the `serve` subcommand is
  required; `-v`/`-vv` lives on `ServeArgs`, not the top-level `Cli`), and
  verify each affected command block actually runs.
