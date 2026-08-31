---
id: SPEC-0041
title: Retire the orphaned `exec` help text now printed for `orcker mcp`
phase: 0
covers: [FR-002]
depends_on: [SPEC-0002]
surface:
  - bin/orcker/
  - bin/orckerd/
  - crates/orcker-doctor/
status: draft
attempts: 0
---

## Context

SPEC-0002 deleted `Command::Exec` but left its doc-comment block behind
(`bin/orcker/src/cli.rs:203-214`). Consecutive `///` lines attach to the next
item, so that block now documents `Command::Mcp`, and clap prints it: `orcker
help mcp` opens with "Run a tool under the PHP version pinned to a site…" and
tells the user to run `orcker exec --site blog php -v`, a command that no longer
exists. Found while disposing SPEC-0002's deleted tests under SPEC-0037, which
may not fix it (SPEC-0037 is restore-only, R4).

## Requirements

- R1. Delete the orphaned `Exec` doc block and its `disable_help_flag` `//`
  rationale, so `orcker help mcp` prints only the MCP description.
- R2. Check the same class across `bin/orcker/src/cli.rs`: every other variant
  SPEC-0002 removed, and every surviving variant's rendered help.
- R3. Same class, same crate, found while disposing SPEC-0037's rows: a doc
  comment at `bin/orcker/src/lib.rs:786` still points at `canonicalize_db_paths`,
  and `lib.rs:970` keeps that function's `// ─── canonicalize_db_paths ───` test
  section banner with no tests under it. Both name a deleted function.
  In `bin/orckerd/`, `mutate.rs:79` likewise still explains itself in terms of
  "a `SetPhp` on a parked site", a request SPEC-0002 deleted, and
  `crates/orcker-doctor/src/lib.rs:13` turns on "the typed
  `orcker_core::PhpVersion` a `FixAction::RestartFpm` needs".

## Acceptance checklist

- [ ] AC1 `orcker help mcp` prints only the MCP description -> evidence: the
      command's output before and after
- [ ] AC2 No variant's rendered help names a command SPEC-0002 deleted
- [ ] AC3 No comment under `bin/orcker/`, `bin/orckerd/` or `crates/orcker-doctor/`
      names a symbol SPEC-0002 deleted
- [ ] AC4 `scripts/gate.sh specs/SPEC-0041-*.md` passes
