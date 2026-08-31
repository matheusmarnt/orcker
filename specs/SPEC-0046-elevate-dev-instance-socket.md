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
