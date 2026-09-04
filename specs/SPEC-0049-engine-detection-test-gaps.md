---
id: SPEC-0049
title: Close the two SPEC-0004 test gaps that hide silent failures
phase: 0
covers: [FR-010]
depends_on: [SPEC-0004]
surface:
  - bin/orckerd/
  - bin/orcker/
status: accepted
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

## Requirements

R1. `EngineStatusCache::get`'s freshness decision (serve the cached value while
younger than `TTL`; otherwise probe and store the result) is covered by a test
that fails independently for each silent-failure shape named in Context: serving
a stale entry past `TTL` (never expires), and not persisting a fresh probe's
result (never stores). The probe must be injectable so the test does not require
a real Docker daemon.

R2. `run_status`'s two-exchange sequence (`Request::Status`, then
`Request::EngineStatus`, combined into the rendered docker section and exit
code) is covered by a test that fails if either exchange is removed or the two
responses stop being combined correctly. A test that independently replays the
same two exchanges, rather than exercising `run_status` (or a seam only
`run_status` calls in production), does not satisfy R2 - deleting `run_status`'s
second exchange must fail the test.

R3. Neither R1 nor R2 changes observable behavior of `orcker status` or the
daemon's engine-status handling.

## Design & contracts

R1 (already implemented, unchanged by this revision):
`async fn get_with(&self, probe: impl Future<Output = DockerStatus>) -> DockerStatus`,
private to `engine_status.rs`, owns the freshness decision. `get()` delegates to
it with the real probe (`orcker_engine::io::detect_from_env()`).

R2: extract the part of `run_status` that gathers both responses and builds the
rendered result into a `pub` async function in `bin/orcker/src/lib.rs`, returning
`map::Rendered`. `run_status` itself becomes: call it, print `stdout`/`stderr`,
convert `code` to `ExitCode`. The new integration test in `cli_e2e.rs` calls this
function directly - not a hand-replay of its steps - so removing its
`EngineStatus` exchange fails the test. Exact function name is the implementer's
call; it must be reachable from `bin/orcker/tests/cli_e2e.rs` as an external
integration test (`pub`, not `pub(crate)`).

## Test plan

- Unit (pure, table-driven): `engine_status.rs` tests, `get_with` driven with
  `std::future::ready` - no real Docker daemon (R1).
- Integration (real daemon, real IPC socket, no fakes): `cli_e2e.rs` calls the
  R2 seam directly against the per-test daemon already brought up by
  `cli_commands_round_trip_against_daemon` (R2).
- E2E / manual: none - both gaps close inside daemon-plus-CLI integration tests
  already exercised by `cargo test --workspace`.

## Acceptance checklist

- [ ] AC1 the cache's expiry-then-reprobe-then-store path is proven, with the
  two failure shapes distinguishable in one test → test:
  `engine_status::tests::an_expired_entry_is_reprobed_and_the_result_is_stored`
- [ ] AC2 `run_status`'s two-exchange wiring is proven through a seam only
  `run_status` calls, and is mutation-sensitive (the test fails if the
  `EngineStatus` exchange is deleted from production code - record the actual
  before/after run in the cycle log as evidence, since this AC has no natural
  RED) → test: `bin/orcker/tests/cli_e2e.rs` (name decided during implementation)
- [ ] AC3 `scripts/gate.sh specs/SPEC-0049-engine-detection-test-gaps.md` passes

## Out of scope

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

`specs/TRACEABILITY.md` and the `accepted` status transition are S8 work, not
part of this checklist.

## Agent notes

Read first: `bin/orckerd/src/engine_status.rs`, `bin/orcker/src/lib.rs`
(`run_status`, its neighbors `run_lan_toggle`/`run_sites` for the extraction
pattern), `bin/orcker/src/map.rs` (`Rendered`, `docker_section`,
`render_status`), `bin/orcker/tests/cli_e2e.rs` (`send`, and the existing
`Command::Status` arm in `cli_commands_round_trip_against_daemon`).

Pitfall: `pub(crate)` is invisible to `tests/cli_e2e.rs` - it compiles as a
separate crate depending on the library's public API, same reason `run_status`
itself cannot be called from there today. The R2 seam must be `pub`.

Precedent for "extract a testable core, keep a thin printing/`ExitCode`
wrapper": `get_with` in this same spec (R1).
