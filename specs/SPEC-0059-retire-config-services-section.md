---
id: SPEC-0059
title: Retire or reconnect the orcker.toml [services] section
phase: 0
covers: [FR-002]
depends_on: [SPEC-0002]
surface:
  - crates/orcker-core/
  - crates/orcker-config/
status: draft
attempts: 0
---

## Context

Found while closing SPEC-0035 (R1/DECISIONS.md, JG1/JG8 round 2). `orcker-config`'s
`ServicesSection`/`ServiceInstance`/`KNOWN_SERVICES` (`schema.rs`) and
`crates/orcker-core/src/service_directives.rs` (dialect-aware override
validation/rendering for mysql/postgres/redis) have no consumer outside
`crates/orcker-config/` itself - confirmed by grep, the same structural
shape `php_pool.rs` was in before SPEC-0035. `bin/orckerd` has no `services`
module at all, despite `schema.rs:636` citing
`orckerd::services::auto_start_installed` as the daemon-side consumer; that
citation does not resolve to any code in this workspace. Unlike `[php.pool]`,
this is a much larger surface (a whole config section plus a ~1000-line core
module), and it is unclear whether the right fix is deletion (SPEC-0035's
route) or reconnecting it to `orcker-stack`/`orcker-engine`'s Docker-based
service rendering, which may simply not exist yet. Needs product input before
a Requirements/Acceptance checklist can be written.
