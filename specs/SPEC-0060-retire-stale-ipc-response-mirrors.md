---
id: SPEC-0060
title: Retire the ipc/types.ts Response mirrors with no Rust counterpart
phase: 0
covers: [FR-002]
depends_on: [SPEC-0040]
surface:
  - apps/orcker-gui/src/ipc/types.ts
status: draft
attempts: 0
---

## Context

Found while building SPEC-0040's dead-export ratchet. Five `Extract<Response, …>`
aliases in `apps/orcker-gui/src/ipc/types.ts` - `PhpVersionsResponse`,
`AvailablePhpResponse`, `ServicesResponse`, `AvailableServicesResponse`,
`ServiceLogsResponse` - extract wire tags (`php_versions`, `available_php`,
`services`, `available_services`, `service_logs`) that no longer exist on
`crates/orcker-ipc::Response` (confirmed against `response.rs`'s current variant
list): the Rust side of this contract is already gone, presumably retired by
SPEC-0034/SPEC-0035 without the TS mirror being swept in the same cycle.

SPEC-0040's ratchet deliberately does not delete them itself (that spec's surface
is the ratchet mechanism, not this archaeology) and instead carries them in
`apps/orcker-gui/tests/dead-export-allowlist.txt` with a reason pointing here.
Unclear before starting whether the right fix is deletion (matching SPEC-0035's
route for `php_pool`) or whether any of these five names are about to be needed
again once the Docker-based PHP-version/services work lands (SPEC-0008, SPEC-0009,
SPEC-0018, SPEC-0019, SPEC-0021 are still queued, not built) - needs product input
before a Requirements/Acceptance checklist can be written.
