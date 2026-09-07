---
id: SPEC-0034
title: Make IPC version skew produce a typed error instead of a decode failure
phase: 1
covers: [FR-002]
depends_on: [SPEC-0002]
surface:
  - crates/orcker-ipc/
  - bin/orckerd/
  - bin/orcker/
status: accepted
attempts: 1
---

## Context

`crates/orcker-ipc/src/lib.rs` documents the gap: `PROTOCOL_VERSION` never
travels on the wire (a frame is a 4-byte big-endian length plus JSON, no
handshake), and "a client speaking a newer protocol against an older daemon
surfaces as `IpcError::Decode` when an unknown `type` tag arrives". `orckerd` is
long-lived, so an upgrade leaves a window where client and daemon disagree and
the user sees a decode error rather than an actionable one. SPEC-0002 bumped
`PROTOCOL_VERSION` to 2 and marked the constant informational precisely because
this spec does not exist yet.

## Requirements

- R1. Decide between the two shapes and record the choice: a `Hello`/`Welcome`
  handshake that exchanges `PROTOCOL_VERSION` on connect, or a
  `#[serde(other)]` fallback variant on `Request` so an unknown tag decodes to
  a known "unsupported" value.
- R2. Whichever is chosen, an unknown or mismatched request must surface as a
  typed `ErrorCode` naming the version mismatch, never `IpcError::Decode`.
- R3. Additive only: no existing wire literal changes, `tests/wire_stability.rs`
  is extended, not rewritten.

## Acceptance checklist

- [x] AC1 A test feeds the daemon an unknown `type` tag and asserts the typed
      error, not a decode failure
- [x] AC2 `tests/wire_stability.rs` diff is additions only
- [x] AC3 The `PROTOCOL_VERSION` doc comment stops saying the constant is
      informational

FR acceptance: FR-002 has AC1/AC2 (`docs/PRD.md`), both already closed by
SPEC-0002 (workspace compiles/tests green without the native-runtime crates;
no binary starts native PHP/DB processes). This cycle adds no new FR
acceptance criteria; AC1-AC3 above close R1-R3 of this spec, not FR-002's
PRD ACs. The gate's `cargo test --workspace` run (S5) incidentally
re-exercises FR-002 AC1 as a side effect of running the whole suite; it does
not close it independently.
