---
id: SPEC-0002
title: Remove the native runtime (PHP-FPM pools, native services, supervision)
phase: 0
covers: [FR-002]
depends_on: [SPEC-0001]
surface:
  - Cargo.toml
  - Cargo.lock
  - crates/
  - bin/
  - apps/orcker-gui/
status: draft
attempts: 0
---

## Context

Orcker's runtime is Docker; Yerd's native runtime must go. This spec removes
`orcker-php`, `orcker-services` and `orcker-supervise` (post-rename names) from
the workspace and every wiring/command/page that depends on them, leaving a
smaller but fully green workspace. Per the viability analysis, "PHP version per
site" becomes "PHP image per project" later (SPEC-0019); nothing replaces the
removed features in this spec.

## Requirements

- R1. Delete crates `orcker-php`, `orcker-services`, `orcker-supervise` and the
  dumps `ext_install`/`dump_server` wiring that depends on installed PHP
  builds; remove them from workspace members and all dependents.
- R2. Daemon (`bin/orckerd`): remove FPM pool startup, native service manager
  wiring and PHP release/update checking. The daemon must still start, own the
  router/DNS/proxy/cert store, and serve IPC.
- R3. CLI (`bin/orcker`): remove commands `install php`, `uninstall php`,
  `update php`, `restart php`, `set php`, `unset php`, `services`, `service *`,
  `db *`, and `use <version>` (global PHP default). Keep `use <site> <ver>` as
  a stub returning a typed "not yet implemented (SPEC-0019)" error only if the
  parser structure makes removal disproportionate; otherwise remove it too.
- R4. IPC (`orcker-ipc`): remove the request/response variants that exist only
  for the removed features. This is an authorized wire-contract reset for the
  fork: bump `PROTOCOL_VERSION` to 2 and update `tests/wire_stability.rs` to
  the new baseline in the same change (document in `specs/DECISIONS.md`).
- R5. GUI: remove/stub the PHP-versions and Services pages (placeholder route
  with a "coming with Docker engine" note is acceptable); Sites, Doctor, Mails
  and Settings pages keep compiling and passing tests.
- R6. `orcker-core` keeps pure PHP *types* that model a per-site PHP version
  (they will describe image tags later); delete only pool/runtime logic
  (`php_pool` and friends) that has no future consumer.
- R7. Doctor: remove checks that probe native PHP/services; keep CA/resolver/
  ports checks green.
- R8. The full workspace (including GUI tests/build) is green afterwards.

## Design & contracts

Deletion-first: rely on the compiler as the dependency walker (`cargo build`
after removing workspace members; delete or adapt every red site). Prefer
removing code over feature-gating — the fork does not keep a native mode (D01).
No new dependencies.

## Test plan

- Inherited suite: green after removals (tests of removed crates disappear with
  them; tests of remaining crates must not be weakened — deletions of tests
  outside the removed crates require per-case justification in the cycle log).
- New unit tests: none required.
- Manual evidence: daemon starts; `orcker status` responds; GUI builds.

## Acceptance checklist

- [ ] AC1 Workspace members list contains no `orcker-php`, `orcker-services`,
      `orcker-supervise` -> evidence: root `Cargo.toml`
- [ ] AC2 `cargo test --workspace` green -> evidence: gate output
- [ ] AC3 `rg -n "orcker_php|orcker_services|orcker_supervise" crates bin apps`
      returns no matches
- [ ] AC4 `cargo run -p orckerd` starts; `cargo run -p orcker -- status`
      answers without native-runtime sections -> evidence: output in cycle log
- [ ] AC5 `PROTOCOL_VERSION == 2` and wire-stability tests pass on the new
      baseline; reset recorded in `specs/DECISIONS.md`
- [ ] AC6 GUI `npm run test` and `npm run build` green
- [ ] AC7 `scripts/gate.sh specs/SPEC-0002-*.md` passes

## Out of scope

Any Docker functionality (SPEC-0003+); removing the proxy's FastCGI forward
path (kept — future use is decided later); mail sink removal (kept, D06 makes
it optional later); tunnel and MCP (kept as-is even if some MCP tools now
return errors — MCP adaptation is SPEC-0024).

## Agent notes

Read first: root `Cargo.toml`, `bin/orckerd/src` (daemon wiring), the CLI
command tree in `bin/orcker/src`, `crates/orcker-ipc/src/{request,response}.rs`
and `tests/wire_stability.rs`, GUI routes in `apps/orcker-gui/src`. Pitfalls:
`yerdd`-inherited `tests/no_runtime_deps.rs` graphs will need their forbidden/
expected crate lists updated (that is a legitimate in-surface test edit — this
spec authorizes it); the dumps extension wiring reaches into PHP install paths
(remove with R1, dumps return in FR-122/Phase 2); MCP tools map 1:1 to IPC
requests — tools for removed requests must be dropped from the catalog so the
`orcker-mcp` crate compiles.
