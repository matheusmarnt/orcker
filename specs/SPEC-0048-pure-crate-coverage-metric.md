---
id: SPEC-0048
title: Make the pure-crate coverage metric measurable in CI
phase: 0
covers: [FR-001]
depends_on: [SPEC-0047]
surface:
  - .github/
  - docs/
status: draft
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
