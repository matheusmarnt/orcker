# Crates Overview

Orcker is a single Cargo workspace. The `[workspace]` members in
[`Cargo.toml`](https://github.com/forjedio/orcker/blob/main/Cargo.toml) split into
three layers:

- **Library crates** (`crates/*`) hold all the logic. Most are *pure* - no I/O,
  no clock, no env, no async - and the few that touch the world push their side
  effects behind traits.
- **Binaries** (`bin/*`) wire those libraries together: the unprivileged daemon
  (`orckerd`), the CLI client (`orcker`), and the privileged one-shot (`orcker-helper`).
- **The desktop app** (`apps/orcker-gui`) and **build automation** (`xtask`) sit at
  the top.

This page is the index. Each entry links to its own detailed reference page.

::: info Workspace facts
All members share a single `version`, `edition = "2021"`, and `rust-version = "1.77"`
from `[workspace.package]` - except `orcker-gui`, which pins `rust-version = "1.85"`
because current Tauri v2 needs edition2024 (rustc ≥ 1.85). The workspace forbids
`unsafe_code` and denies `unwrap`/`expect`/`panic`/`todo`/`dbg!`/`indexing_slicing`
in non-test code via `[workspace.lints]`. `orcker-gui` opts out of the strict lint
set (it wraps macro-heavy generated Tauri code) but still bans
`unwrap`/`expect`/`panic` in its own bridge code.
:::

## The workspace at a glance

### Library crates

| Crate | Purpose | Pure? | Page |
|---|---|:---:|---|
| `orcker-core` | Pure domain types (`Site`, `PhpVersion`, `Tld`) and host→site routing. The foundation every other crate depends on. | pure | [orcker-core](./crates/orcker-core) |
| `orcker-ipc` | IPC protocol, framing, and codec between `orckerd` and its clients. Length-prefixed JSON frames. | pure (default) | [orcker-ipc](./crates/orcker-ipc) |
| `orcker-config` | Persisted, schema-versioned TOML configuration. Pure parse/validate/serialise plus a thin atomic load/save. | mostly pure | [orcker-config](./crates/orcker-config) |
| `orcker-tls` | Pure-Rust local CA and per-site leaf certificate issuance (`rcgen` + `ring`). No I/O, no clock. | pure | [orcker-tls](./crates/orcker-tls) |
| `orcker-dns` | Authoritative `*.test` DNS responder plus `hickory-server` wiring. | owns I/O | [orcker-dns](./crates/orcker-dns) |
| `orcker-proxy` | Hand-rolled HTTP/HTTPS reverse proxy on `hyper` + `tokio-rustls`; serves static files and forwards to PHP-FPM over FastCGI. | owns I/O (pure submodule) | [orcker-proxy](./crates/orcker-proxy) |
| `orcker-supervise` | Process-agnostic supervision substrate: trait seams, the pure restart/health state machine, and tokio impls. Shared by `orcker-php` and `orcker-services`. | owns I/O (pure submodule) | [orcker-supervise](./crates/orcker-supervise) |
| `orcker-php` | PHP-FPM pool supervision and version management; discovery, install, health-probing. | owns I/O (pure submodule) | [orcker-php](./crates/orcker-php) |
| `orcker-services` | Local database / cache supervision (Redis/Valkey, MySQL, MariaDB, Postgres), version management, and SQL database administration. | owns I/O (pure submodule) | [orcker-services](./crates/orcker-services) |
| `orcker-doctor` | Pure diagnosis and fix-planning for `orcker doctor`. Turns a `StatusReport` into findings and safe auto-fixes. | pure | [orcker-doctor](./crates/orcker-doctor) |
| `orcker-platform` | OS abstraction layer: paths, trust store, resolver installer, port binder/redirector, metrics - one impl per OS. | owns I/O (pure submodule) | [orcker-platform](./crates/orcker-platform) |
| `orcker-mail` | Built-in mail-capture SMTP sink plus its on-disk store. Accepts mail on a loopback port (Herd-style) and persists parsed messages for the GUI. Depends on `orcker-ipc` + `mail-parser`. | owns I/O | [orcker-mail](./crates/orcker-mail) |
| `orcker-mcp` | Model Context Protocol server logic: the curated tool catalog agents call, the sans-io JSON-RPC state machine, and tool-result rendering. Consumed by `orcker mcp`, which owns the stdio loop and the daemon exchange. Depends on `orcker-ipc` and `orcker-core`. | pure | [orcker-mcp](./crates/orcker-mcp) |
| `orcker-tunnel` | Cloudflare Tunnel support: pure `cloudflared` argv / `config.yml` generation and log parsing, plus a supervised `cloudflared` lifecycle. Powers public site sharing. Depends on `orcker-supervise`. | owns I/O (pure submodules) | [orcker-tunnel](./crates/orcker-tunnel) |
| `orcker-update` | Pure release-channel selection and version-decision logic for self-update: `select_target` (stable/edge resolution), artifact selection by platform, and checksum/minisign verification. | pure | [orcker-update](./crates/orcker-update) |
| `orcker-service-ctl` | Cross-platform start/stop/restart control for the `orckerd` daemon service (`launchctl`/`systemctl`), used by the self-update applier to restart onto a freshly-swapped binary. | owns I/O | [orcker-service-ctl](./crates/orcker-service-ctl) |

### Binaries

| Binary | Purpose | Page |
|---|---|---|
| `orckerd` | The unprivileged per-user daemon. Wires every library together; owns all runtime state and serves the proxy, DNS, PHP pools, and database/cache services. | [orckerd (daemon)](./binaries/orckerd) |
| `orcker` | The CLI - a thin `orcker-ipc` client that talks to `orckerd` over a per-user socket. | [orcker (CLI)](./binaries/orcker) |
| `orcker-helper` | The privileged one-shot. Validates a typed `HelperInvocation`, performs exactly one root operation (CA install, resolver install, `setcap`), and exits. | [orcker-helper (privileged)](./binaries/orcker-helper) |

### App and tooling

| Member | Purpose | Page |
|---|---|---|
| `orcker-gui` (`apps/orcker-gui`) | Tauri v2 + Vue 3 desktop/tray app - another thin `orcker-ipc` client of `orckerd`, like the CLI. | [Desktop App Internals](./gui) |
| `xtask` | Build automation invoked as `cargo xtask <cmd>`: `bump`, `version-check`. | [Build Automation (xtask)](./xtask) |

### Test-only infrastructure

| Member | Purpose | Page |
|---|---|---|
| `orcker-depcheck` | Shared `cargo metadata` dependency-graph assertions, pulled in via `[dev-dependencies]` only by six crates'/binaries' `tests/no_runtime_deps.rs` guards - never a runtime dependency of anything shipped. | [orcker-depcheck](./crates/orcker-depcheck) |

This member is deliberately excluded from the "library crates" table and the
dependency graph below: it is never reachable from any shipped binary's
`[dependencies]`, only from test targets via `[dev-dependencies]` - a
different kind of workspace member entirely, closer in spirit to `xtask`
(tooling that helps build/verify the workspace) than to a crate `orckerd`
assembles at runtime.

## Internal dependency graph

Dependencies run **downhill**: `orcker-core` is the bedrock at the bottom and has
zero internal dependencies; every arrow points down toward the things it relies
on. `orcker-tls` is the other leaf - it has no internal `orcker-*` dependencies at all.

Apps and tooling sit at the top, binaries in the middle, libraries below, with
`orcker-core` as the bedrock. `xtask` has no internal `orcker-*` deps, so it stands
alone. Arrows point from a crate to what it depends on.

```mermaid
flowchart TD
    gui["orcker-gui (app)"]
    xtask["xtask (no internal deps)"]
    orcker["orcker (CLI)"]
    orckerd["orckerd (depends on all fourteen libs)"]
    helper["orcker-helper"]

    ipc["orcker-ipc"]
    config["orcker-config"]
    dns["orcker-dns"]
    proxy["orcker-proxy"]
    php["orcker-php"]
    services["orcker-services"]
    supervise["orcker-supervise"]
    doctor["orcker-doctor"]
    tls["orcker-tls"]
    platform["orcker-platform"]
    mail["orcker-mail"]
    mcp["orcker-mcp"]
    tunnel["orcker-tunnel"]
    update["orcker-update"]
    servicectl["orcker-service-ctl"]
    core["orcker-core (pure, zero internal deps)"]

    gui --> core
    gui --> ipc
    gui --> platform
    gui --> update

    orcker --> core
    orcker --> ipc
    orcker --> platform
    orcker --> update
    orcker --> servicectl
    orcker --> mcp

    helper --> core
    helper --> platform

    orckerd --> core
    orckerd --> config
    orckerd --> ipc
    orckerd --> tls
    orckerd --> platform
    orckerd --> dns
    orckerd --> php
    orckerd --> services
    orckerd --> supervise
    orckerd --> proxy
    orckerd --> doctor
    orckerd --> mail
    orckerd --> tunnel
    orckerd --> update

    ipc --> core
    mcp --> ipc
    mcp --> core
    config --> core
    dns --> core
    proxy --> core
    doctor --> core
    doctor --> ipc
    mail --> ipc
    php --> core
    php --> platform
    php --> supervise
    services --> platform
    services --> supervise
    tunnel --> supervise
    platform --> tls
```

Read as a nested list of direct internal dependencies (each verified against the
crate's `Cargo.toml`):

- **`orcker-core`** → *(none)*
- **`orcker-tls`** → *(none - workspace leaf)*
- **`orcker-ipc`** → `orcker-core`
- **`orcker-config`** → `orcker-core`
- **`orcker-dns`** → `orcker-core`
- **`orcker-proxy`** → `orcker-core`
- **`orcker-doctor`** → `orcker-core`, `orcker-ipc`
- **`orcker-platform`** → `orcker-tls`
- **`orcker-supervise`** → *(none - workspace leaf)*
- **`orcker-php`** → `orcker-core`, `orcker-platform`, `orcker-supervise`
- **`orcker-services`** → `orcker-platform`, `orcker-supervise`
- **`orcker-mail`** → `orcker-ipc`
- **`orcker-mcp`** → `orcker-core`, `orcker-ipc`
- **`orcker-update`** → *(none - workspace leaf)*
- **`orcker-service-ctl`** → *(none - workspace leaf)*
- **`orcker-helper`** (bin) → `orcker-core`, `orcker-platform`
- **`orcker`** (bin) → `orcker-core`, `orcker-ipc` (`transport`), `orcker-platform`, `orcker-update`, `orcker-service-ctl`
- **`orcker-gui`** (app) → `orcker-core`, `orcker-ipc` (`transport`), `orcker-platform`, `orcker-update`
- **`orckerd`** (bin) → `orcker-core`, `orcker-config`, `orcker-ipc` (`transport`), `orcker-tls`, `orcker-platform`, `orcker-dns`, `orcker-supervise`, `orcker-php`, `orcker-services`, `orcker-proxy`, `orcker-doctor`, `orcker-mail`, `orcker-tunnel`, `orcker-update` - **all fourteen libraries**
- **`xtask`** → *(no internal deps; `anyhow` + `clap` only)*

::: tip The daemon is the assembly point
Only `orckerd` depends on every library. It is where the pure logic, the OS
adapters, and the network servers are stitched into one running process. The CLI,
the helper, and the GUI each pull in only the narrow slice they need. This is what
the README means by "the daemon owns state; the CLI and GUI are clients."
:::

## Pure vs. I/O-owning crates

Orcker's central design rule (from the README): **pure logic lives in library
crates; I/O and OS calls are pushed to the edges behind traits**, with one trait
implementation per OS. That makes the bulk of the codebase unit-testable in-memory
with fakes, and keeps behaviour identical across platforms.

### Fully pure

These crates do no I/O, no async, and read no clock or environment. Callers pass
in everything (timestamps, reports) explicitly.

- **`orcker-core`** - `#![forbid(unsafe_code)]`, no async, no internal deps. The
  crate header states it plainly: *"It is **pure**: no I/O, no async, no internal
  `orcker-*` dependencies. Side effects belong behind traits in `orcker-platform`."*
- **`orcker-tls`** - *"It does **no I/O**, **no clock reads**, and **no env reads** -
  callers pass timestamps via [`Validity`]."* Persistence lives in `orcker-config`;
  trust-store install lives in `orcker-platform`.
- **`orcker-doctor`** - *"runtime-free and does no I/O."* `diagnose()` maps a
  `StatusReport` to findings; `plan_auto_fixes()` returns the safe unprivileged
  `FixAction`s. The daemon performs the actual I/O and re-runs `diagnose()`
  afterwards.
- **`orcker-ipc`** - the default build is pure: *"no sockets, no async, no I/O."*
  Only the optional `transport` feature pulls in `tokio` for the shared async
  read/write helpers (the daemon and CLI both enable it).
- **`orcker-update`** - *"This crate does **no I/O**: it operates on already-fetched
  release metadata and the running version."* `select_target` (channel
  resolution) and artifact selection/verification are all pure functions over
  in-memory data; fetching releases and downloading artifacts is the caller's
  job (`orckerd` and the applier).

### Pure with a thin I/O seam

- **`orcker-config`** - *"Every function except `Config::load` and `Config::save`
  is pure."* Parse, validate, serialise, and migrate are all pure; load/save are a
  thin atomic file seam. Schema is versioned (`CURRENT_VERSION = 5`), decoupled
  from the IPC `PROTOCOL_VERSION`.

### I/O-owning, with a pure submodule

These crates genuinely own runtime side effects, but isolate their decision logic
in a `pure` module that is unit-tested in-memory.

- **`orcker-platform`** - the OS abstraction layer. Core traits (`Paths`,
  `TrustStore`, `ResolverInstaller`, `PortBinder`, `PortRedirector`) each have one
  thin `#[cfg(target_os = ...)]` impl; decision logic that needs no OS interaction
  lives in `pure`. It is **unprivileged**: operations needing root return
  `PlatformError::NeedsHelper` carrying a typed `HelperInvocation` for `orcker-helper`
  to execute - the OS impls never spawn the helper themselves.
- **`orcker-proxy`** - owns the `hyper` + `tokio-rustls` servers (`pub mod server`,
  `forward`, `tls`, `backend`), with a `pub mod pure` for the routing/decision
  logic (including `try_files` static-file resolution) and `pub mod traits`
  (`BackendResolver`, `CertStore`) at the edges.
- **`orcker-supervise`** - the shared supervision substrate. Pure `supervisor`
  (state machine), `listen`, and `error`; tokio adapters (`TokioProcessSpawner`,
  `SystemClock`, `TokioChild`) and the trait *definitions* (`ProcessSpawner`,
  `Clock`, `HealthProbe`, `Downloader`) in `real`/`traits`.
- **`orcker-php`** - supervises PHP-FPM processes. Drives the `orcker-supervise` state
  machine under `SupervisorPolicy::fpm()`; provides the `FastCgiProbe` `HealthProbe`
  impl, with `pure` holding the FPM-config and release-resolution logic.
- **`orcker-services`** - supervises database / cache engines. Drives the same
  `orcker-supervise` machine under `SupervisorPolicy::database()`; pure `service`,
  `database`, `config_render`, and `release` modules, with I/O in `manager` and
  `health`.
- **`orcker-dns`** - runs the authoritative responder on `tokio` + `hickory-server`.
- **`orcker-service-ctl`** - shells out to `launchctl`/`systemctl` (and signals a
  pid directly as a fallback) to stop/start/restart the `orckerd` service. No
  `pure` submodule of its own - the crate is small enough that the OS-mechanics
  functions are the whole surface - but no `unsafe` either: process signalling
  goes through `nix`'s safe wrappers.

::: details Where the traits are implemented
The trait *definitions* live in the library crates (e.g. `orcker-supervise`'s
`ProcessSpawner`, `orcker-platform`'s `TrustStore`). The library also ships the
production impls (`TokioProcessSpawner`, the per-OS `TrustStore`). Tests substitute
in-memory fakes. The privileged half of `orcker-platform`'s work is executed out of
process by `orcker-helper`, which the daemon or `sudo orcker elevate` invokes.
:::

## Where to go next

- For the runtime picture - how `orckerd` boots, supervises, and serves - see
  [Architecture](./architecture) and [The Daemon](../guide/daemon).
- For the wire contract between client and daemon, see
  [IPC Protocol](./ipc-protocol) and [orcker-ipc](./crates/orcker-ipc).
- For the per-OS adapter model, see [Cross-Platform Model](./cross-platform) and
  [orcker-platform](./crates/orcker-platform).
- To build the workspace, see [Building from Source](./building).
