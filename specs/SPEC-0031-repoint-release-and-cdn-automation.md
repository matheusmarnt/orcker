---
id: SPEC-0031
title: Repoint the release and CDN automation at real Orcker infrastructure
phase: 0
covers: [FR-001]
depends_on: [SPEC-0001]
surface:
  - xtask/
  - scripts/
  - packaging/
status: draft
attempts: 0
---

## Context

SPEC-0001's rename rewrote every upstream URL mechanically, so
`xtask/src/cdn.rs`, `scripts/release.sh`, `.github/workflows/release.yml`,
`build-cdn.yml`, `cdn-sync.yml` and `packaging/arch/*` now name Orcker hosts and
repositories that do not exist (`files.orcker.app`, `matheusmarnt/orcker`
release assets). All of them are `workflow_dispatch`-only, so nothing fires
today, but the first release attempt fails until they are repointed. Raised by
the SPEC-0001 supervisor as regression note 3.

## Requirements

- R1. Decide the real hosts and repositories, then repoint every reference.
- R2. Prove it: a dry-run of the release path that reaches the upload step.
