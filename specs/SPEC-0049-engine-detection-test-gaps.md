---
id: SPEC-0049
title: Close the two SPEC-0004 test gaps that hide silent failures
phase: 0
covers: [FR-010]
depends_on: [SPEC-0004]
surface:
  - bin/orckerd/
  - bin/orcker/
status: draft
attempts: 0
---

## Context

SPEC-0004's supervisor listed five untouched areas in `regression_notes`. Two
are worth a test and three are not; this spec is the two.

**`EngineStatusCache::get` (`bin/orckerd/src/engine_status.rs`).** Only `fresh()`
is covered. The expiry path - entry older than `TTL` triggers a re-probe and the
result is written back - is unverified, and both ways it can break are silent:
a cache that never stores serves a connect timeout on every `orcker status` and
every GUI poll, and one that never expires reports Docker as down long after the
user started it. The probe is not injectable today; the smallest change that
makes it testable is to have `get` delegate to a `get_with(probe: impl Future<Output = DockerStatus>)`
that owns the freshness decision, and test that with ready futures.

**`run_status` (`bin/orcker/src/lib.rs`).** The rework moved the branching into
the pure `map::docker_section`, which is covered, so what remains untested is the
two-exchange wiring itself. `bin/orcker/tests/cli_e2e.rs` already runs the CLI
against a live daemon, so this is one more case there, asserting the `docker`
section is present and the exit code is 0.

Deliberately excluded, with reasons, so a later cycle does not re-open them:
`io/mod.rs`'s `DOCKER_HOST`/`HOME` reads are three lines of `std::env::var`
plumbing whose ordering already lives in the nine-row `socket_resolution_matrix`;
the bollard connect path needs a live Docker daemon and belongs with the harness
SPEC-0010 has to build anyway; and `warm_engine_status` is a detached
`tokio::spawn` whose only observable is a log line, not worth a synchronization
seam. The supervisor's fifth note - that `map::render`'s `Response::Status` arm
is now unreachable from `Command::Status` - is **rejected**: that arm is
exercised by `renders_human_responses_and_exit_codes`, the test SPEC-0037
restored verbatim, so collapsing the two renderers would delete an accepted
spec's work.
