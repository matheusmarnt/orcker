---
id: SPEC-0058
title: Remove or rewrite the stale orcker-php crate doc
phase: 0
covers: [FR-002]
depends_on: [SPEC-0002]
surface:
  - docs/
status: draft
attempts: 0
---

## Context

`docs/developer/crates/orcker-php.md` documents a `crates/orcker-php` crate
that does not exist in this workspace (native-runtime PHP-FPM pool
rendering, gone with SPEC-0002). `docs/developer/ipc-protocol.md:173`
similarly documents a `SetPhpPoolSettings` IPC request that has no matching
variant in `orcker-ipc` (confirmed by grep: zero references outside these
two doc files). Found, and deliberately left out of that cycle's diff, while
fixing SPEC-0035's own stale `php_pool` doc references (JG6). Decide whether
either doc's history belongs in `docs/developer/building.md`'s removed-crates
notes instead, or should just be deleted outright.
