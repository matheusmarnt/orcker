---
id: SPEC-0051
title: Cover `orcker link`'s container semantics, which no automated test exercises
phase: 0
covers: [FR-021]
depends_on: [SPEC-0006]
surface:
  - bin/orcker/
status: accepted
attempts: 1
---

## Context

SPEC-0006 repointed `Command::Link` from the inherited on-disk site link
(`Request::Link`) to the container-project link (`Request::LinkProject`), a
product-level change confirmed with the human and recorded in
`specs/DECISIONS.md`. Nothing tests it.

`bin/orcker/tests/cli_e2e.rs` builds its request by calling
`orcker::resolve_link` directly (its own comment says `Command::Link` never
reaches `map::to_request`), so it still exercises only the legacy path. The new
`resolve_link_project` has no test at all: not the current-directory fallback,
not `--name` validation, not `--port` passthrough. The supervisor recorded this
in SPEC-0006's `regression_notes`; the evidence for the new behaviour is a
manual transcript in `specs/logs/SPEC-0006.md`.

This is a coverage gap in shipped code, not a defect: the manual run linked,
relinked idempotently and served the golden thread. But the next refactor of
`bin/orcker/src/lib.rs` has nothing to fail against.

## Requirements

- R1. Pure unit tests for `resolve_link_project`: an explicit relative path is
      absolutised; an omitted path resolves to the current directory; an invalid
      `--name` is rejected as `ClientError::Usage`; a valid `--name` and `--port`
      reach the request unchanged.
- R2. One end-to-end test that drives `Command::Link` the way `main` does (not
      by calling a helper), against the existing temp-daemon harness, and asserts
      a `Response::Project` with `created: true`, then a second run with
      `created: false` and an unchanged config.
- R3. The legacy `resolve_link` tests stay untouched: `Request::Link` is still
      served for the GUI, and removing its coverage would hide that.

## Acceptance checklist

- [x] AC1 (R1) four cases pass -> test:
      `orcker::tests::resolve_link_project_*`
- [x] AC2 (R2) link then relink over IPC -> test in `bin/orcker/tests/cli_e2e.rs`
- [x] AC3 (R3) `git diff` touches no existing `resolve_link` test
- [x] AC4 `scripts/gate.sh specs/SPEC-0051-*.md` passes

## Out of scope

Retiring `Request::Link` or `resolve_link` (SPEC-0014 owns link/park). Any
change to the link behaviour itself: this spec only pins what SPEC-0006 shipped.

## Agent notes

SPEC-0014 (`cli-link-park`, FR-021/FR-030) may absorb this instead of it running
standalone. If SPEC-0014 is drafted first, fold R1-R3 into it and close this as
superseded rather than writing the tests twice.
