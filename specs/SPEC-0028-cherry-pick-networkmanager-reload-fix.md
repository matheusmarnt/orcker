---
id: SPEC-0028
title: Cherry-pick the upstream NetworkManager reload-flags fix into the helper
phase: 1
covers: [FR-030]
depends_on: [SPEC-0001]
surface:
  - bin/orcker-helper/
status: draft
attempts: 0
---

## Context

The fork froze at Yerd `v2.1.0-rc.1` (`896c449`), so upstream `7d69d6a`
("Pass NetworkManager reload flags as one argument", PR #211, 2026-08-18) is not in
the base. It is the only one of the four commits above the tag with durable value for
Orcker: a correctness fix in the privileged helper's Linux resolver, which is what
makes the inherited `.test` DNS (FR-030) work under NetworkManager. The other three
are the native-PHP coverage shim (deleted by SPEC-0002), the Yerd-branded docs site
(replaced by E14) and a cosmetic macOS GUI animation.

This spec exists because `docs/UPSTREAM.md` states the policy: upstream merges are
deliberate, cherry-picked events. Pulling it in during bootstrap would have been an
improvisation; pulling it in through the loop is that policy working as designed.

Deferred on purpose: it does not block SPEC-0001's rename, and the path it touches is
renamed by SPEC-0001 first (hence `depends_on`).

## Requirements

- R1. Cherry-pick upstream `7d69d6a` onto the renamed tree, adapting identifiers to
  the post-SPEC-0001 names (`yerd-helper` -> `orcker-helper`, `yerd-` config file
  prefixes -> `orcker-`). Behaviour must match upstream exactly.
- R2. Pass the NetworkManager reload flags as a single argument, matching upstream:
  `nmcli general reload conf,dns-full` — not as separate argv entries.
- R3. Record the cherry-pick in `docs/UPSTREAM.md` (commit sha, date, one-line
  reason), establishing the log format for future upstream picks.

## Design & contracts

No new dependencies. No public API change. The change is confined to the helper's
Linux resolver op and the constant holding the reload arguments.

## Test plan

- Unit: the reload argv is built as a single `conf,dns-full` argument, not split.
- Integration: resolver apply and rollback both invoke the reload through the fake
  `ProcessSpawner`, asserting the exact argv.

## Acceptance checklist

- [ ] AC1 reload argv is one argument -> test: `resolver::reload_args_single_argument`
- [ ] AC2 rollback path also reloads with the same argv -> test:
      `resolver::rollback_reloads_networkmanager`
- [ ] AC3 `docs/UPSTREAM.md` lists the cherry-pick (sha, date, reason)
- [ ] AC4 `scripts/gate.sh specs/SPEC-0028-*.md` passes

## Out of scope

The other three commits above the tag; any further upstream sync; macOS resolver
paths; changing the `.test` TLD.

## Agent notes

Read first: the upstream diff via `git show 7d69d6a` (the `upstream` remote is
configured), then the renamed resolver op in `bin/orcker-helper/src/ops/resolver.rs`.
Pitfall: on the frozen tag the constant does not exist yet — the upstream diff
introduces `NETWORKMANAGER_RELOAD_ARGS`; do not hand-roll a different shape.
