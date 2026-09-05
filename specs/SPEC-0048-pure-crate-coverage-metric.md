---
id: SPEC-0048
title: Make the pure-crate coverage metric measurable in CI
phase: 0
covers: [FR-001]
depends_on: [SPEC-0047]
surface:
  - .github/
  - docs/
status: accepted
attempts: 0
---

## Context

SPEC-0047 added the `Pure-crate coverage` column to `specs/TRACEABILITY.md` so
the SDD section 11 retrospective can answer its fourth metric, and required
cycles from that point on to fill it in. SPEC-0004 could not, and recorded `—`
with the reason in `specs/DECISIONS.md`: nothing measures coverage here and
`rustup component list` shows `llvm-tools` is not installed, so any measurement
needs `rustup component add llvm-tools-preview` plus `cargo install
cargo-llvm-cov`.

Put the measurement in **CI on the Linux leg only, reported and never gated**,
so no developer has to install anything and the macOS leg stays untouched:
`cargo llvm-cov --summary-only` over the pure crates (`orcker-core`,
`orcker-stack`, `orcker-engine`, and `orcker-catalog` when it lands). A cycle
reads the number off that job at S8. Do **not** add it to `scripts/gate.sh` -
`gate.sh` is the thing every cycle runs locally, and a coverage threshold there
would turn a reporting metric into a build dependency.

State in `docs/SDD.md` where the number comes from and that a failed or skipped
coverage job leaves the column `—` rather than blocking S8.

## Requirements

- R1: `.github/workflows/ci.yml`'s `rust` job gains one step that runs
  `cargo llvm-cov --summary-only -p orcker-core -p orcker-stack -p
  orcker-engine`. The step runs only on the single x86_64 Linux leg (`if:
  matrix.os == 'ubuntu-22.04'`), not on `ubuntu-22.04-arm` or `macos-14`, and
  is marked `continue-on-error: true` so it reports without ever failing the
  job. `orcker-catalog` is not added to the `-p` list because the crate does
  not exist in this repository yet.
- R2: `scripts/gate.sh` is not modified in any way by this spec — coverage
  stays a CI-reported metric, never a local build dependency or a gate step.
- R3: `docs/SDD.md` section 11 gains one new English sentence (the rest of the
  section stays in its existing language) stating that the `Pure-crate
  coverage` column's number comes from the `cargo llvm-cov --summary-only`
  step on the Linux (`ubuntu-22.04`) leg of the `rust` CI job, and that when
  that job fails or is skipped the column is left `—` at S8 rather than
  blocking it.

## Design & contracts

No Rust code changes and no public API, trait, or IPC changes: this spec is a
CI workflow edit (YAML) plus one documentation sentence.

- New CI step (added to the existing `rust` job in
  `.github/workflows/ci.yml`, after the existing test steps), scoped with
  `if: matrix.os == 'ubuntu-22.04'` and `continue-on-error: true`, running:
  - Setup (once, same step or a preceding one, same OS scoping):
    `rustup component add llvm-tools-preview` and
    `cargo install cargo-llvm-cov`.
  - Measurement: `cargo llvm-cov --summary-only -p orcker-core -p
    orcker-stack -p orcker-engine`.
- New dependency declared here per the supervisor's DT6: the CI runner
  installs the `cargo-llvm-cov` crate (via `cargo install`) and the
  `llvm-tools-preview` rustup component. Both are CI-only tooling, not
  workspace dependencies — nothing changes in any `Cargo.toml`.
- `docs/SDD.md` §11 edit: append one sentence in English to the existing
  paragraph that names the `Pure-crate coverage` target and column, without
  translating or altering the surrounding Portuguese text.

## Test plan

- Unit: none — a CI-workflow-only change has no unit under test.
- Integration: none — no trait, no fake, no side effect behind a boundary is
  introduced.
- E2E / manual (unavoidable here: exercising the actual GitHub Actions step
  requires a push, and this repository's process reserves `git push` for the
  human, so the workflow step itself cannot be triggered inside this cycle).
  Evidence substitute: the exact command was already run locally against this
  workspace's three pure crates, after installing `llvm-tools-preview` and
  `cargo-llvm-cov 0.9.0`, and produced a real summary table:
  ```
  cargo llvm-cov --summary-only -p orcker-core -p orcker-stack -p orcker-engine
  ...
  TOTAL 7631 339 95.56%  [region]  594 30 94.95%  [function]  4692 192 95.91%  [line]
  ```
  This is pasted as evidence for AC1; the CI YAML itself is reviewed by
  grep/diff per the acceptance checklist below since the workflow only
  actually runs on a push.

## Acceptance checklist

- [ ] AC1 The command `cargo llvm-cov --summary-only -p orcker-core -p
      orcker-stack -p orcker-engine` runs successfully over the workspace's
      pure crates -> evidence: pasted local run output ending with a `TOTAL`
      line (see Test plan).
- [ ] AC2 `.github/workflows/ci.yml`'s `rust` job has a new step that runs the
      AC1 command, scoped to `if: matrix.os == 'ubuntu-22.04'` and marked
      `continue-on-error: true` -> evidence: `grep -n "llvm-cov"
      .github/workflows/ci.yml` shows the step, and `grep -n
      "matrix.os == 'ubuntu-22.04'"` / `grep -n "continue-on-error: true"`
      show it scoped and non-gating.
- [ ] AC3 `scripts/gate.sh` is untouched by this spec -> evidence: `git diff
      --name-only <base>..HEAD -- scripts/gate.sh` prints nothing.
- [ ] AC4 `docs/SDD.md` states, in English, where the `Pure-crate coverage`
      number comes from and that a failed or skipped job leaves the column
      `—` at S8 -> evidence: `grep -n "llvm-cov" docs/SDD.md` shows the new
      sentence.
- [ ] AC5 `scripts/gate.sh specs/SPEC-0048-pure-crate-coverage-metric.md`
      passes -> evidence: command output ends `[gate] OK`.

## Out of scope

- Adding `orcker-catalog` to the measured crate list — the crate does not
  exist in this repository yet; wiring it in is future work once it lands.
- Any coverage threshold or gate, in CI or in `scripts/gate.sh` — this metric
  is report-only, per the Context and R2.
- The `macos-14` and `ubuntu-22.04-arm` CI legs — the measurement step runs
  on the single `ubuntu-22.04` leg only, per R1.

## Agent notes

Files to read first: `.github/workflows/ci.yml` (existing `rust` job and the
`if: runner.os == 'Linux'` precedent used for other Linux-only steps),
`docs/SDD.md` §11 (retrospective metrics, around the `Pure-crate coverage`
target and column), `specs/TRACEABILITY.md` (rows already marked "queued as
SPEC-0048"), and `specs/DECISIONS.md`'s SPEC-0004 entry (the original reason
the column was left `—`).

Known pitfall: do not add `orcker-catalog` to the `-p` list — the crate does
not exist yet in `crates/`, and `cargo -p` on an absent package is a hard
error that would fail the whole step even under `continue-on-error` semantics
at the wrong layer (the install/setup commands, not the coverage run, could
still fail the job outright if misordered).
