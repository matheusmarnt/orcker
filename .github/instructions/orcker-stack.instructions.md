---
applyTo: "crates/orcker-stack/**/*.rs"
---

# orcker-stack — typed stack model → rendered compose

Renders a Docker Compose file from a typed, validated stack configuration.
No I/O, no engine calls — it produces a `String`; callers write it to disk.

**Layer:** strictly pure. No async, no I/O, no `orcker-*` dependency. Crate
root carries `#![forbid(unsafe_code)]`. Only dependency is `thiserror`.

## Owns

- `render_compose(&StackConfig) -> Result<String, StackError>`: the only
  render entry point, returns a `docker-compose.yml` as a `String`.
- `StackConfig` (site, php, db, preset, ports, uid, gid; private fields,
  getters only), built via validated constructors.
- `Ports::new(http_loopback, vite)` (rejects `0` and a duplicate pair),
  `SiteName` (DNS label: `[a-z0-9-]`, no leading/trailing hyphen, ≤63 chars),
  `PhpVersion` (closed enum, V81..V85).
- `DbEngine` and `Preset` (`#[non_exhaustive]`, currently `Postgres` /
  `Reference`).
- The `StackError` family with reason enums: `SiteNameErrorReason`,
  `PhpVersionErrorReason`, `PortField`, `PortErrorReason`.

## Must not

- Perform any I/O: never write the rendered file, read the filesystem, the
  network, the clock, or the environment.
- Depend on `tokio`, `serde`, or any other `orcker-*` crate.
- Publish a project port on `0.0.0.0`, set `privileged: true`, or mount the
  Docker socket in a generated stack — these are workspace-wide bans
  (`CLAUDE.md`), and this is the crate that would introduce them.
- Serialise the compose output through a map or a YAML library: the template
  is one `format!` literal and its byte-for-byte determinism is a pinned
  invariant.
- Use a single `$` in the healthcheck template — Compose needs `$$` there.
- Reorder the checks inside `validate()`; the order is contract
  (`site_name.rs`), not incidental.
- Add a `PhpVersion` variant without a published image for it.
- Make a `StackConfig` field public; callers get typed getters, not the
  struct shape.

## Conventions

- New validation rules are pure functions with table-driven tests, one
  `StackError` reason per failure mode.
- `DbEngine` / `Preset` are `#[non_exhaustive]`: adding a variant is additive
  and must not break an existing `match`.

## Tests / invariants

- `tests/validate.rs` — pins the error *reason* and *order* per invalid
  input, and the accepted `PhpVersion` set (8.1–8.5).
- `tests/compose.rs` — snapshot vs `tests/fixtures/compose_reference_postgres.yml`;
  byte-identical determinism across renders; and the security invariants: no
  `0.0.0.0`, no `privileged`, no `docker.sock`, every published port starts
  with `127.0.0.1:`.

## Review checklist

- [ ] Change is pure — no I/O, clock, env, or async crept in.
- [ ] No new dependency, especially no `orcker-*` crate or `serde`.
- [ ] Rendered stack still bans `0.0.0.0` publishing, `privileged: true`, and
      the Docker socket mount.
- [ ] `validate()` order unchanged unless the change is deliberate and
      re-pinned in `tests/validate.rs`.
- [ ] Snapshot and determinism tests updated only for an intended render
      change.
