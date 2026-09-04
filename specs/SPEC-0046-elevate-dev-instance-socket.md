---
id: SPEC-0046
title: elevate cannot reach a dev instance isolated by XDG_* overrides
phase: 0
covers: [FR-001]
depends_on: [SPEC-0001]
surface:
  - bin/orcker/
  - docs/
status: draft
attempts: 0
---

## Context

`docs/developer/building.md` documents running a parallel dev instance by
overriding `XDG_CONFIG_HOME`/`XDG_DATA_HOME`/`XDG_STATE_HOME`/`XDG_CACHE_HOME`/
`XDG_RUNTIME_DIR`. `bin/orcker/src/elevate.rs:499` (`socket_candidates`)
deliberately ignores the environment when `SUDO_UID` is set and rebuilds
uid-derived paths, so `sudo env XDG_RUNTIME_DIR=… orcker elevate` fails with
`daemon not running` against a live dev daemon. Found while running SPEC-0005
(finding F8); worked around there with a symlink.

## Requirements

- R1. `elevate` reaches a daemon whose runtime dir was overridden, without
  weakening the home-independent reconstruction that protects the sudo path.
- R2. `docs/developer/building.md` and the elevation guide agree with the code.

## Design & contracts

`bin/orcker/src/elevate.rs`, `unix_impl` module:

```rust
fn socket_candidates() -> Vec<PathBuf> {
    use orcker_platform::{ActivePaths, Paths};
    if let Some(uid) = sudo_uid() {
        return user_socket_candidates(uid, std::env::var("XDG_RUNTIME_DIR").ok().as_deref());
    }
    match ActivePaths::new().resolve() {
        Ok(dirs) => vec![dirs.runtime.join("orcker.sock")],
        Err(_) => Vec::new(),
    }
}

fn user_socket_candidates(uid: u32, xdg_runtime_dir: Option<&str>) -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    if let Some(dir) = xdg_runtime_dir.filter(|d| !d.is_empty()) {
        candidates.push(PathBuf::from(dir).join("orcker").join("orcker.sock"));
    }
    candidates.push(PathBuf::from(format!("/run/user/{uid}/orcker/orcker.sock")));
    candidates.push(PathBuf::from(format!("/tmp/orcker-{uid}/orcker.sock")));
    candidates
}
```

`user_socket_candidates` gains a second parameter instead of reading the env
var itself, so it stays a pure, table-testable function exactly like today
(the existing `user_socket_candidates_are_uid_based` test becomes its `None`
case). Only `socket_candidates`, already the sole caller, reads
`std::env::var`, mirroring how it already reads `sudo_uid()`.

The new candidate is tried first by both existing call sites (`fetch_facts`
at L138 and L448 both loop `for sock in socket_candidates()`, first
successful `transport::exchange_at` wins), so neither call site changes.

Layout mirrors `PlatformDirs`'s own non-sudo path (`runtime.join("orcker.sock")`,
where Linux `runtime` is `XDG_RUNTIME_DIR/orcker` per `paths.rs`), and closes
the gap `PlatformDirs::for_user`'s own doc comment already names (`paths.rs`
L38-51): a caller that wants the `XDG_RUNTIME_DIR` location "must add it
itself, since the real value can't be recovered from a stripped sudo
environment."

## Test plan

- Unit (pure, table-driven): `user_socket_candidates(uid, xdg_runtime_dir)`.
  `Some(path)` prepends `{path}/orcker/orcker.sock`; `None` and `Some("")`
  both fall back to today's two uid-based candidates, unchanged and in
  today's order.
- Integration (side effects behind traits, tested with fakes): none needed.
  `socket_candidates`/`fetch_facts` already retry every candidate against a
  real (or absent) socket; no new I/O seam is introduced.
- E2E / manual (only when unavoidable, must say why): follow
  `docs/developer/building.md`'s dev-instance recipe, then run
  `sudo env XDG_RUNTIME_DIR=/tmp/orcker-dev/run orcker elevate trust`
  against it. Unavoidable because the fix's value is sudo's real
  env-scrubbing behavior, which a unit test cannot reproduce.

## Acceptance checklist

- [ ] AC1 `user_socket_candidates(uid, Some(xdg_dir))` returns
  `{xdg_dir}/orcker/orcker.sock` first, then the two existing uid-based
  candidates unchanged and in order → test: `orcker elevate::unix_impl::tests::user_socket_candidates_prefers_xdg_runtime_dir_override`
- [ ] AC2 `user_socket_candidates(uid, None)` and
  `user_socket_candidates(uid, Some(""))` both return exactly today's two
  candidates, in today's order → test: `orcker elevate::unix_impl::tests::user_socket_candidates_without_xdg_override_is_unchanged`
- [ ] AC3 `sudo env XDG_RUNTIME_DIR=<scratch> orcker elevate trust` reaches a
  dev daemon started per `docs/developer/building.md`'s XDG recipe, instead
  of failing with `daemon not running` → evidence: manual run transcript
  showing `==> trust:` narration and no `daemon not running` line
- [ ] AC4 `docs/developer/building.md`'s dev-instance section documents that
  `elevate` needs the override passed through sudo explicitly
  (`sudo env XDG_RUNTIME_DIR=... orcker elevate`, not a plain
  `sudo orcker elevate` after `export`) → evidence: doc diff review
- [ ] AC5 `scripts/gate.sh specs/SPEC-0046-elevate-dev-instance-socket.md` passes

## Out of scope

- macOS: `XDG_RUNTIME_DIR` plays no part in macOS's resolution
  (`crates/orcker-platform/src/os/macos.rs`), and `building.md`'s
  dev-instance-via-XDG recipe is Linux-only. No macOS behavior changes.
- `orcker uninstall`'s use of `PlatformDirs::for_user` (file deletion, not a
  live daemon dial). Unaffected, out of scope.
- sudo's own env-passthrough policy (`sudoers`, `env_keep`). This spec only
  makes `elevate` use the var when it is already present in its own process
  environment.

## Agent notes

Read first: `bin/orcker/src/elevate.rs` L496-521 (`socket_candidates`,
`user_socket_candidates`, `sudo_uid`) and its `tests` module (L598 on) for
the existing test style; `crates/orcker-platform/src/paths.rs` L38-51
(`PlatformDirs::for_user` doc comment, already names this exact gap);
`docs/developer/building.md` L218-235 (the dev-instance XDG recipe this spec
makes `elevate` reach).

Pitfall: sudo's default `env_reset` strips the invoking shell's exported
vars. A plain `export XDG_RUNTIME_DIR=...` then `sudo orcker elevate` will
NOT reproduce the fix; the var must be passed through explicitly, e.g.
`sudo env XDG_RUNTIME_DIR=... orcker elevate` or `sudo -E orcker elevate`
with `XDG_RUNTIME_DIR` in `env_keep`. Use the explicit-passthrough form in
tests, docs and evidence, not a plain `export` + `sudo`.
