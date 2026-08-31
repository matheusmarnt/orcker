# Phase-0 spike — a containerized Laravel app at `https://spike.test`

Runbook and findings for SPEC-0005 (FR-003). The question it answers: can the
proxy, local CA and `.test` DNS inherited from Yerd front an app that runs in
containers, with no runtime of our own?

Short answer up front: **yes, and with no code change at all** — the inherited
whole-host proxy (`orcker proxy add`) already expresses it.

- Stack sources: `stack/` beside this file (hand-written; generation is
  SPEC-0003/0007)
- Evidence: `evidence/`
- Reference the stack follows: `../referencia-docker-laravel.md`

## What runs where

| Piece | Where | Why |
|-------|-------|-----|
| nginx | container, published on `127.0.0.1:18080` | the single entry point the proxy talks to |
| php-fpm + horizon + schedule | container, supervisord | the reference stack's process model |
| postgres 18, redis | containers, `app-network` only | not published to the host |
| orcker daemon | host, unprivileged | terminates TLS, resolves `.test`, forwards to 18080 |

Nothing is published on `0.0.0.0`, nothing runs `privileged: true`, and the
Docker socket is never mounted — the repository's hard rules for generated
stacks, honoured here by hand.

## Prerequisites

- Docker Engine + Compose v2 (verified on Engine 29.7.2 / Compose v5.5.0)
- `libnss3-tools` (`certutil`), or `elevate trust` cannot reach the browsers
- A Laravel checkout outside this repository; the runbook uses
  `~/orcker-spike/app`

## 1. Build the binaries and run a dev daemon

The dev instance is isolated from any production install
(`../developer/building.md`), so nothing here touches a real setup:

```sh
cargo build -p orckerd -p orcker -p orcker-helper

export XDG_CONFIG_HOME=/tmp/orcker-dev/config
export XDG_DATA_HOME=/tmp/orcker-dev/data
export XDG_STATE_HOME=/tmp/orcker-dev/state
export XDG_CACHE_HOME=/tmp/orcker-dev/cache
export XDG_RUNTIME_DIR=/tmp/orcker-dev/run
mkdir -p /tmp/orcker-dev/{config,data,state,cache,run}

./target/debug/orckerd serve &      # NOT `orckerd -v`, see Findings F1
./target/debug/orcker ping          # -> pong
```

Rootless, the daemon binds `127.0.0.1:8080` and `127.0.0.1:8443`.

## 2. Create the application

```sh
mkdir -p ~/orcker-spike
docker run --rm -v ~/orcker-spike:/src -u "$(id -u):$(id -g)" -w /src \
  composer:latest create-project laravel/laravel app --no-interaction --prefer-dist

cp -n ~/orcker-spike/app/.env.example ~/orcker-spike/app/.env
# then set: APP_URL=https://spike.test, DB_CONNECTION=pgsql, DB_HOST=postgres,
# DB_DATABASE/USERNAME/PASSWORD=spike, REDIS_HOST=redis, QUEUE_CONNECTION=redis
```

## 3. Bring the stack up

```sh
export SPIKE_APP=~/orcker-spike/app
export SPIKE_UID=$(id -u) SPIKE_GID=$(id -g)   # UID is read-only in bash, see F2
docker compose -f docs/spike/stack/compose.yaml build app
docker compose -f docs/spike/stack/compose.yaml run --rm --no-deps --entrypoint "" app \
  sh -lc "composer require laravel/horizon --no-interaction && php artisan horizon:install"
docker compose -f docs/spike/stack/compose.yaml up -d
docker compose -f docs/spike/stack/compose.yaml exec app php artisan key:generate
docker compose -f docs/spike/stack/compose.yaml exec app php artisan migrate --force
```

Check the stack on its own port before involving orcker at all:

```sh
curl -s -o /dev/null -w '%{http_code}\n' http://127.0.0.1:18080   # -> 200
```

## 4. Put `spike.test` in front of it

This is the whole delta the spike needed — two commands, no new code:

```sh
./target/debug/orcker proxy add spike http://127.0.0.1:18080
./target/debug/orcker secure spike
```

## 5. Elevate (the one privileged step)

`.test` DNS, the CA in the system trust store, and binding 80/443 need root
once. Passing the `XDG_*` variables through `sudo env` does **not** work - see F8:
under `sudo`, `elevate` ignores the environment and probes uid-derived socket
paths. Point one of those at the dev instance instead:

```sh
ln -sfn /tmp/orcker-dev/run/orcker /tmp/orcker-$(id -u)   # see F8
sudo ./target/debug/orcker elevate
rm /tmp/orcker-$(id -u)                                   # afterwards
```

On Linux `elevate ports` sets `cap_net_bind_service` on the `orckerd` binary,
so restart the daemon afterwards for 80/443 to bind. Reverse everything with
`sudo … orcker unelevate`.

## Results

Executed on Linux (kernel 6.8, Docker Engine 29.7.2, Compose v5.5.0), 2026-08-31.
Raw output in `evidence/`.

### The thesis holds, with zero code delta

`orcker proxy add spike http://127.0.0.1:18080` plus `orcker secure spike` was the
entire configuration. No new IPC message, no change to `orcker-core`,
`orcker-config`, `orckerd` or the CLI. R2's conditional delta never triggered.

| Stage | Result | Evidence |
|-------|--------|----------|
| before anything | `spike.test` unresolved, nothing on 18080 | `evidence/red-before-spike.txt` |
| proxy registered, stack down | TLSv1.3, `subject: CN=spike.test`, `issuer: CN=Orcker Local CA`, verify ok, `502 Bad Gateway` | `evidence/proxy-registered-upstream-down.txt` |
| stack up | `200`, `<title>Laravel</title>`, fpm + horizon + schedule RUNNING | `evidence/green-stack-up.txt` |
| websocket | `101 Switching Protocols` through the proxy, `vite-hmr` subprotocol preserved | `evidence/websocket-hmr.txt` |
| elevated, real `:443` | `200` with **no `--cacert`** - the system trust store validates the leaf; `http://` answers `301` to `https://` | `evidence/elevated-443.txt` |
| browser | Laravel 13 welcome page over `https://spike.test`, no certificate interstitial | `evidence/browser-spike-test.jpg` |

### Latency overhead

Measured twice. The first pass ran against cold opcache and showed nothing;
once PHP was warm the proxy's cost became visible. Medians of 20 warm requests,
**debug build** of `orckerd`:

| Path | Median |
|------|--------|
| direct to nginx, `http://127.0.0.1:18080` | 31.3 ms |
| through orcker, `https://spike.test` (`:443`) | 113-125 ms |

The gap is the TLS handshake, not forwarding: `time_connect` is ~3 ms while
`time_appconnect` (handshake complete) is 40-76 ms, and curl performs a fresh
handshake per invocation. This is an **unoptimised debug build** - a release
build and connection reuse both cut directly into that number, and a browser
holds the connection open. NFR-02 is a Phase-1 concern; the honest Phase-0
statement is *the handshake dominates and has not been measured on a release
build*.

### Headers

`nginx` logs `xfp`/`xfh` (added to the log format for this spike). 23 proxied
requests carried `xfp=https xfh=spike.test:8443`; the 21 direct requests carried
neither. The proxy preserves `Host` and adds `X-Forwarded-*` exactly as
`../guide/proxies.md` documents.

### Volume ownership

The `UID`/`GID` build args did their job: `id` inside the container is
`uid=1000(orcker) gid=1000(orcker)`, and every file the app writes
(`storage/logs/laravel.log`) lands as `1000:1000` on the host. No root-owned
files, no `chown` dance.

### Websockets / HMR

Vite v8.2.2 upgrades cleanly **through** the proxy once the app allows the
hostname - see F5. `sec-websocket-accept` and the `vite-hmr` subprotocol both
survive the hop, so `forward/upgrade.rs` needs nothing for this case.

## Findings

Ordered by what they cost a future spec.

- **F5 - a generated stack must configure Vite's host allowlist.** Vite 5+
  rejects any `Host` it does not know: `Blocked request. This host
  ("vite.test") is not allowed.` This is not a proxy defect - hitting Vite
  *directly* with `Host: vite.test` returns the same 403. The stack must emit
  `server.allowedHosts` for the project's `.test` domain, and
  `server.hmr.clientPort` must match the port the browser actually reaches
  (`8443` rootless, `443` elevated). Feeds **SPEC-0007** (preset) and
  **SPEC-0006**.
- **F6 - `X-Forwarded-Host` carries the rootless port** (`spike.test:8443`).
  Laravel's URL generation follows that header, so a rootless install builds
  `:8443` URLs. Worth an explicit decision in Phase 1 rather than a surprise.
- **F7 - SPEC-0006 may not need a new routing mechanism.** It is queued as
  "link/loopback port", but the inherited whole-host proxy already accepts an
  arbitrary `http://127.0.0.1:<port>` upstream. What is actually missing is
  *port allocation and a project registry*, not routing. SPEC-0006 should be
  re-read with that in mind before it is drafted further.
- **F8 - `elevate` cannot reach a dev instance.** `docs/developer/building.md`
  documents isolating a parallel instance with the `XDG_*` variables, but
  `bin/orcker/src/elevate.rs:499` deliberately ignores the environment when
  `SUDO_UID` is set and rebuilds uid-derived paths
  (`/run/user/$uid/orcker/orcker.sock`, then `/tmp/orcker-$uid/orcker.sock`).
  Passing the variables through `sudo env` therefore changes nothing and elevate
  reports `daemon not running`. The two documented workflows are incompatible;
  the spike worked around it with a symlink
  (`ln -sfn /tmp/orcker-dev/run/orcker /tmp/orcker-1000`). Queued as SPEC-0046.
- **F4 - `postgres:18` moved its data directory.** The volume must mount at
  `/var/lib/postgresql`, not `/var/lib/postgresql/data`; with the old path the
  container exits 1 on first boot. The reference document predates the change.
- **F2 - the reference dependency list is missing `libonig-dev`.** Without it
  `docker-php-ext-install mbstring` fails with `Package 'oniguruma', required
  by 'virtual:world', not found`. Any preset derived from the reference
  inherits the bug.
- **F3 - `UID` is read-only in bash.** `UID=$(id -u) docker compose ...` fails
  with `UID: readonly variable` before docker even starts. The compose file
  uses `SPIKE_UID`/`SPIKE_GID`; a generated stack should avoid `UID` too.
- **F1 - `../developer/building.md` documents `orckerd -v`,** which the binary
  rejects (`error: unexpected argument '-v' found`); the command is
  `orckerd serve`. Already queued as SPEC-0030 - this run is a second
  confirmation, not new work.

### Deviations from the reference stack

- `php:8.4-fpm`, not `php:8.4-rc-fpm` - 8.4 is stable and the `-rc` tag no
  longer publishes.
- No `laravel-pulse` supervisor program - Pulse is not in a fresh Laravel
  install, and AC3 asks only for fpm + horizon + schedule.
- No external `development` network - the spike keeps everything inside
  `app-network`; the shared network is Phase 1.
- A `redis` service inside the stack - Horizon needs it and there is no global
  services runtime yet.
- Postgres credentials inline in `compose.yaml` rather than `env_file: .env` -
  the compose file lives in this repository while the Laravel `.env` lives in
  the scratch checkout.
