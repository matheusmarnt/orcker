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
status: accepted
attempts: 6
---

## Context

Orcker's runtime is Docker; Yerd's native runtime must go. This spec removes
`orcker-php`, `orcker-services` and `orcker-supervise` (post-rename names) from
the workspace and every wiring/command/page that depends on them, leaving a
smaller but fully green workspace. Per the viability analysis, "PHP version per
site" becomes "PHP image per project" later (SPEC-0019); nothing replaces the
removed features in this spec.

## Requirements

- R1. Delete crates `orcker-php`, `orcker-services` and `orcker-supervise` and
  the dumps `ext_install`/`dump_server` wiring that depends on installed PHP
  builds; remove them from workspace members and all dependents.
  `orcker-supervise` has one surviving consumer, `orcker-tunnel`, which this
  spec keeps (FR-120/SPEC-0026), so deleting the crate means **folding** its
  five modules into `crates/orcker-tunnel/src/supervise/` rather than dropping
  the code (amended at S2, attempt 1; diagnosis and the two rejected
  alternatives in `specs/logs/SPEC-0002.md`). The fold keeps the pure state
  machine pure and the tokio impls at the I/O edge; `orcker-tunnel` absorbs the
  `tokio` features, `async-trait` and `nix` that `orcker-supervise` declared.
- R2. Daemon (`bin/orckerd`): remove FPM pool startup, native service manager
  wiring and PHP release/update checking. Also remove every handler that spawns
  the installed PHP binary, which PRD FR-002 AC2 forbids outright: `create_site/`
  and its Laravel/WordPress scaffolders, `tools/{laravel,wp_cli}.rs`,
  `wordpress_users.rs`, `wordpress_url_sync.rs` and `wordpress_versions.rs`.
  Site creation returns via containers under PRD FR-020 (amended at S4;
  diagnosis and the rejected stub alternative in `specs/logs/SPEC-0002.md`).
  The generic job machinery (`JobStatus`, `JobCancel`) stays - tool and
  cloudflared installs still stream through it. The daemon must still start, own
  the router/DNS/proxy/cert store, and serve IPC.
- R2b. `orcker-php` also held the workspace's download/unpack substrate, whose
  surviving consumers this spec keeps (self-update, tool installs, the
  cloudflared install). Deleting the crate therefore means **moving** that
  substrate, not dropping it: `Os`, `Arch`, target detection and the tar
  zip-slip guard go to `crates/orcker-platform/src/artifact.rs`; the
  `Downloader` trait, `DownloadError` and the `reqwest` implementation go to
  `bin/orckerd/src/download.rs`. `Downloader` and `DownloadError` are deleted
  from `crates/orcker-tunnel/src/supervise/`, which never used them (added at
  S4, same log).
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

No new workspace dependencies: `tokio` (features `process`, `net`, `rt`,
`io-util`, `sync`), `async-trait` and `nix` are already pinned in
`[workspace.dependencies]`; R1's fold only moves those edges from the deleted
`orcker-supervise` manifest onto `orcker-tunnel`. `Cargo.lock` loses three
workspace packages and gains none.

**AC3 is a symbol-level check, not a crate-name grep.** The original wording
grepped the three deleted crate *names*. That cannot see `Request::CreateSite`,
`invoke("list_php")` or `Tool::Composer`, so it passed green while 25 dangling
references survived in files this diff never touched - including
`SitesView.vue`'s two site-creation wizards, live and routed, calling a Tauri
command R2 deleted. `npm test` mocks the client, `npm build` does not validate
Tauri command names, and the Rust side is self-consistent, so nothing in the gate
could see it.

`specs/logs/SPEC-0002-removed-symbols.txt` holds the replacement: every removed
`Request` / `Response` / `DiagnosisCode` / `Tool` / `Command` variant and every
removed MCP tool name, each qualified so it cannot match a surviving item of the
same name, generated from the diff rather than by hand. Of the 25 it found, 13
are closed in this cycle (stale doc prose, three vacuous MCP test rows, four
callerless client exports); the remaining 12 are the GUI surfaces whose backend
this spec removed, delegated to SPEC-0036, which blocks SPEC-0003 and SPEC-0005.
Build output under `apps/orcker-gui/dist/` is excluded - it is regenerated, not
source.

`tests/wire_stability.rs` is edited by **deletion only, with one named
exception** — the `#[test]` functions of the removed variants go, every
surviving literal stays byte-identical. The file is never regenerated: a
regenerated baseline can bury a typo in a kept variant's literal and ship a
silent protocol bug, which is the one failure mode this file exists to catch.

The exception: a test that pins a *surviving* response but uses a **removed enum
variant as its payload** cannot be left byte-identical, because the variant no
longer exists. Such a test is retargeted to a surviving variant rather than
deleted, since deleting it would drop coverage of the surviving response.
Exactly one test qualifies: `response_doctor_fix_byte_shape`, whose
`FixResult.code` moves from `fpm_pool_failed` to `resolver_not_installed` (and
its message with it).

Two surviving literals also lose key groups by **within-line deletion**, which is
still deletion, not a rewrite: `response_status_byte_shape` drops `"default_php"`
and `"php"` because the fields no longer exist on `StatusReport`. Final shape of
the file's diff: 8 added lines against 1403 removed. Five of the eight are
within-line deletions (four import-list reflows and the status literal); the
other three are the one authorized `response_doctor_fix_byte_shape` retarget.

## Test plan

- Inherited suite: green after removals (tests of removed crates disappear with
  them).
- **Coverage of tests deleted from surviving files is delegated to SPEC-0037**
  (human decision at `attempts = 3`, recorded in `specs/DECISIONS.md`). The
  original clause here required a per-case written justification for each such
  deletion; three supervisor passes showed that clause cannot be met inside this
  spec, because it is a 136-item assertion-level review that no acceptance
  criterion of this spec expresses and no mechanical filter can triage (measured:
  a compiler-based classifier has a 4-in-5 false-negative rate against the
  supervisor's own ground truth). The 136 are enumerated in
  `specs/logs/SPEC-0002-deleted-tests.md`; SPEC-0037 makes their disposition its
  acceptance criterion. This spec is therefore accepted on runtime removal alone.
- New unit tests: none required.
- Manual evidence: daemon starts; `orcker status` responds; GUI builds.

## Acceptance checklist

- [x] AC1 Workspace members list contains no `orcker-php`, `orcker-services`
      or `orcker-supervise` -> evidence: root `Cargo.toml` (PRD FR-002 AC1)
- [x] AC2 `cargo test --workspace` green -> evidence: gate output
- [x] AC3a `rg -f specs/logs/SPEC-0002-removed-symbols.txt crates bin --glob '!dist' -l`
      returns nothing: no Rust-side reference to a removed symbol survives
- [x] AC3b The GUI's Tauri command contract is enforced by a test, not a grep:
      `apps/orcker-gui/tests/commandContract.test.ts` cross-checks every
      invoked command against `generate_handler![]` and fails on any name absent
      from the enumerated `ipc/dangling-commands.json`. RED evidence in the cycle
      log. The 49 known-dangling names are delegated to SPEC-0036, whose
      acceptance is emptying that file
- [x] AC4 `cargo run -p orckerd` starts; `cargo run -p orcker -- status`
      answers without native-runtime sections -> evidence: output in cycle log
- [x] AC5 `PROTOCOL_VERSION == 2` and wire-stability tests pass on the new
      baseline; reset recorded in `specs/DECISIONS.md`
- [x] AC6 GUI `npm run test` and `npm run build` green
- [x] AC7 `scripts/gate.sh specs/SPEC-0002-*.md` passes

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
