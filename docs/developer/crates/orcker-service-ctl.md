# orcker-service-ctl

`orcker-service-ctl` is the **start/stop/restart control** for the `orckerd`
daemon's OS service registration. It exists so the self-update applier
([`orcker`](../binaries/orcker)) can restart the daemon onto a freshly-swapped
binary without re-implementing the platform service mechanics - the applier
cannot depend on the GUI binary (dependencies only flow downhill), so this
logic lives in its own library crate instead.

::: info Crate metadata
`description`: *Cross-platform start/stop/restart control for the orckerd
daemon service.* `#![forbid(unsafe_code)]`. Depends on `thiserror`, plus `nix`
(`user` feature, for `getuid`/`kill`) on Unix. No internal `orcker-*`
dependencies and no async runtime - it shells out to `launchctl`/`systemctl`
and uses `nix`'s safe wrappers, so its own dependency graph stays minimal.
:::

See also the [Crates overview](../crates), [`orcker-update`](./orcker-update)
(the version-decision logic the applier resolves before calling into this
crate), and the [Self-Update CLI reference](../../reference/cli/update).

## Module map

The whole crate is one module, split by concern rather than by file:

```text
src/
└── lib.rs   # ServiceCtl, ServiceError, plus the per-OS stop/start/restart mechanics
```

[Browse the source on GitHub.](https://github.com/forjedio/orcker)

## Per-OS mechanics

`ServiceCtl` is constructed with the path to the `orckerd` binary (used only by
the Linux no-systemd fallback) and exposes `stop`, `start`, and `restart`:

- **macOS** - `stop` sends `launchctl kill SIGTERM gui/$uid/dev.orcker.daemon`;
  `start` and `restart` both use `launchctl kickstart -k …`, which kills and
  restarts the already-registered `LaunchAgent` in one step. Registering the
  `SMAppService` job itself is the GUI's job (it owns the Swift/objc
  bindings) - this crate only drives `launchctl` against the already-known
  label.
- **Linux** - when a systemd `--user` instance is reachable
  (`systemctl --user show-environment` exits `0`), `stop`/`start`/`restart`
  map directly onto `systemctl --user {stop,start,restart} orcker`. Otherwise
  `restart` falls back to `stop` → poll for the process to exit (bounded to
  ~5s) → `start`, and `start` spawns a detached `orckerd serve` in its own
  process group so it survives the caller exiting.
- **Every OS** - `stop` also SIGTERMs any still-running `orckerd` pid found via
  `pgrep -x orckerd -U <uid>`, covering a bare `cargo run` / `orckerd serve` that
  no service manager supervises. This is best-effort: a `pgrep` failure or no
  match is treated as "nothing to signal," never an error.

Any platform without a supported service mechanism (i.e. not macOS or Linux)
returns `ServiceError::Unsupported` rather than silently doing nothing.

## Why `restart` differs by OS

macOS registers `orckerd` as a `LaunchAgent`, so `launchctl kickstart -k` is a
single atomic kill-then-restart of that job - there is no window where the
daemon is definitively "down" that the caller needs to wait out. Linux without
systemd has no equivalent primitive: the applier must confirm the old process
has actually released its executable inode and any bound sockets/ports before
spawning a new one, hence the explicit stop → `wait_for_exit` → start sequence
and the `ServiceError::Tool` returned if the daemon doesn't exit before the
timeout - starting a new daemon on top of a still-running old one would just
fail to bind.

## Public API

```rust
pub struct ServiceCtl { /* orckerd_path */ }

impl ServiceCtl {
    pub fn new(orckerd_path: impl Into<PathBuf>) -> Self;
    pub fn stop(&self);
    pub fn start(&self) -> Result<(), ServiceError>;
    pub fn restart(&self) -> Result<(), ServiceError>;
}
```

| Item | Role |
|------|------|
| `ServiceCtl::new(orckerd_path)` | Construct a controller; `orckerd_path` is only used by the Linux no-systemd detached-spawn fallback. |
| `ServiceCtl::stop` | Best-effort stop: asks the service manager, then SIGTERMs any still-running `orckerd`. Infallible. |
| `ServiceCtl::start` | Start via the service manager, or a detached spawn when none is available. |
| `ServiceCtl::restart` | Restart onto a freshly-swapped binary: `launchctl kickstart -k` on macOS, `systemctl --user restart` or stop-wait-start on Linux. |
| `ServiceError` | `Spawn` (couldn't launch the tool/binary), `Tool` (the tool ran and failed), `Unsupported` (no service mechanism on this platform). `#[non_exhaustive]`. |
