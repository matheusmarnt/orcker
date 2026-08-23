---
applyTo: "bin/orckerd/**/*.rs"
---

# orckerd — the daemon

Runs **as the user, unprivileged**, and is the single source of truth for
runtime state. It wires the library crates together and serves the IPC protocol.

**Layer:** thin orchestration only. Behaviour belongs in the crates; this binary
coordinates them.

## Owns (orchestration)

- Load `orcker-config`; build a `SiteRouter` from config + a filesystem scan of
  parked paths (`startup.rs`, `state.rs`).
- Start `orcker-dns` and `orcker-proxy`, and apply doctor fix plans; own the
  cert store (`cert_store.rs`) and backend resolution (`backend_resolver.rs`).
- Serve the `orcker-ipc` server transport (`ipc_server.rs`), handle mutations
  (`mutate.rs`), signals, and single-instance enforcement.
- Install the `tracing` subscriber + rolling file appender (`tracing_init.rs`).
  Libraries only emit spans/events; the daemon owns the subscriber.

## Privileged work

- For anything needing root, go through `orcker-platform` → `HelperInvocation` and
  invoke `orcker-helper`. The daemon owns that `Command::new(...)`; it never does
  the privileged effect inline.

## Must not

- Reimplement crate logic. If a handler grows real behaviour, push it into the
  relevant crate with tests and call it from here.
- Run as root, or assume elevation. Unprivileged operation is the default; fall
  back to `8080`/`8443` when the user declines elevation.
- Install runtimes on the host. PHP and the databases run in containers; the
  daemon orchestrates them through the engine, it never provisions a native
  runtime.

## Tests / invariants

- `tests/lifecycle.rs` — bring the daemon up on temp paths + ephemeral ports.
- `tests/no_runtime_deps.rs` — dependency-graph guard.

## Review checklist

- [ ] New behaviour lives in a crate, not in a handler.
- [ ] Privileged work routed through the helper; daemon stays unprivileged.
- [ ] Runtimes stay in containers; the daemon installs nothing on the host.
- [ ] Daemon remains the single source of truth (no client-side state authority).
