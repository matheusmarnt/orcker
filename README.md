# Orcker

Orcker is a Docker-backed local development orchestrator for PHP and Laravel.
It runs your sites in containers behind a local HTTPS proxy with its own
certificate authority and an embedded `.test` DNS resolver, so every project
gets a real hostname and a trusted certificate without you touching
`/etc/hosts`, a global PHP install, or a hand-written `docker-compose.yml`.
A background daemon owns the runtime; a CLI and a desktop GUI drive it.

> **Status: pre-release.** The version is pinned at `0.0.0` until the MVP gate.
> The Docker runtime is being built spec by spec — see `specs/ROADMAP.md`.

## Documentation

- Product requirements: `docs/PRD.md`
- Development process: `docs/SDD.md`
- Building from source: `docs/developer/building.md`
- Upstream freeze and merge policy: `docs/UPSTREAM.md`

## Lineage and credits

Orcker is a fork of [Yerd](https://github.com/forjedio/yerd) by Forjed, taken at
tag `v2.1.0-rc.1` (commit `896c449`). Yerd is MIT licensed, and Orcker inherits
its daemon, IPC protocol, rustls proxy, local CA, `.test` DNS, doctor, tunnel
and MCP surfaces largely intact. What Orcker changes is the runtime underneath:
Docker and Compose in place of Yerd's native PHP-FPM pools and service engines.

Thank you to the Yerd authors — this project would not exist without their work.
The exact freeze point and the policy for taking further upstream changes are
recorded in `docs/UPSTREAM.md`.

## Not affiliated with Docker, Inc.

Orcker is an independent project. It is not affiliated with, endorsed by, or
sponsored by Docker, Inc. "Docker" and the Docker logo are trademarks of
Docker, Inc., used here only to describe the software Orcker interoperates with.

## Licence

MIT — see `LICENSE.md`. The copyright notice carries both Forjed's line for the
inherited Yerd code and the Orcker line for modifications.
