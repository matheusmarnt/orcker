# Spec queue — Orcker

Ordered queue. `/spec-next` picks the first spec whose **committed** status is
`approved` and whose `depends_on` are all `accepted`. Statuses live in each
spec's front matter; this table is the index (keep both in sync at S8).
`planned` rows have no spec file yet — draft them with `/spec-new` when the
queue approaches them.

## Phase 0 — validation (drafted, awaiting human approval)

| # | Spec | Status | Covers | Depends on |
|---|------|--------|--------|------------|
| 1 | SPEC-0001-fork-bootstrap | accepted | FR-001 | — |
| 2 | SPEC-0002-remove-native-runtime | accepted | FR-002 | SPEC-0001 |
| 33 | SPEC-0033-remove-stale-php-instruction-file | accepted | FR-002 | SPEC-0002 |
| 36 | SPEC-0036-remove-dead-gui-surfaces | accepted | FR-002 | SPEC-0002 |
| 37 | SPEC-0037-restore-coverage-deleted-by-spec-0002 | accepted | FR-002 | SPEC-0002 |
| 3 | SPEC-0003-stack-compose-renderer | accepted | FR-022 (partial) | SPEC-0001 |
| 38 | SPEC-0038-tauri-host-coverage | draft | FR-002 | SPEC-0002, SPEC-0036 |
| 4 | SPEC-0004-engine-docker-detection | draft | FR-010 | SPEC-0001, SPEC-0037 |
| 5 | SPEC-0005-proxy-container-spike | accepted | FR-003 | SPEC-0002, SPEC-0037 |
| 6 | SPEC-0006-link-loopback-port | draft | FR-021, FR-013 (partial) | SPEC-0004, SPEC-0005, SPEC-0037 |

Phase 0 exit: FR-001..003 accepted + process retrospective (SDD section 11-12).

## Phase 1 — MVP backlog (planned; draft via /spec-new after Phase 0 retro)

| # | Spec (planned) | Covers | Depends on |
|---|----------------|--------|------------|
| 7 | SPEC-0007-stack-reference-preset | FR-022 | SPEC-0003 |
| 8 | SPEC-0008-stack-mysql-variant | FR-020 (partial), FR-022 | SPEC-0007 |
| 9 | SPEC-0009-catalog-services-model | FR-050, FR-053 | SPEC-0003 |
| 10 | SPEC-0010-engine-compose-lifecycle | FR-011 | SPEC-0004 |
| 11 | SPEC-0011-engine-observation-events | FR-012 | SPEC-0010 |
| 12 | SPEC-0012-daemon-project-registry | FR-040 (partial), FR-024 | SPEC-0006, SPEC-0010 |
| 13 | SPEC-0013-cli-new | FR-020 | SPEC-0007, SPEC-0012 |
| 14 | SPEC-0014-cli-link-park | FR-021, FR-030 | SPEC-0012 |
| 15 | SPEC-0015-proxy-websocket-integration | FR-031 | SPEC-0012 |
| 16 | SPEC-0016-secure-localhost-regression | FR-032, FR-033 | SPEC-0015 |
| 17 | SPEC-0017-lifecycle-logs-artisan-init | FR-041, FR-042, FR-043 | SPEC-0012 |
| 18 | SPEC-0018-global-services-runtime | FR-051, FR-052 | SPEC-0009, SPEC-0010 |
| 19 | SPEC-0019-php-versions-eol | FR-060, FR-061, FR-062 | SPEC-0012 |
| 20 | SPEC-0020-doctor-docker-checks | FR-070, FR-071 | SPEC-0004 |
| 21 | SPEC-0021-gui-projects-services | FR-080 | SPEC-0012, SPEC-0018 |
| 22 | SPEC-0022-gui-onboarding | FR-081 | SPEC-0021 |
| 23 | SPEC-0023-eject | FR-090 | SPEC-0013 |
| 24 | SPEC-0024-mcp-adaptation | FR-100 | SPEC-0012 |
| 25 | SPEC-0025-images-pipeline-ci | FR-110 | SPEC-0007 |
| 26 | SPEC-0026-tunnel-regression | FR-120 | SPEC-0015 |
| 27 | SPEC-0027-docs-quickstart | FR-130 | SPEC-0013..0023 |

Phase 2/3 items (FR-082, FR-101, FR-111, FR-121, FR-122 and phase-2/3 catalog
services) enter this table only after the MVP release gate (PRD section 10).

## Upstream cherry-picks

Deliberate, individually specified imports from `upstream` (see `docs/UPSTREAM.md`).
Never a merge, never a bulk sync. Numbered above the reserved Phase-1 range.

| # | Spec | Status | Covers | Depends on |
|---|------|--------|--------|------------|
| 28 | SPEC-0028-cherry-pick-networkmanager-reload-fix | draft | FR-030 | SPEC-0001 |
| 29 | SPEC-0029-replace-binary-brand-icons | draft | FR-001 | SPEC-0001, SPEC-0036 |
| 30 | SPEC-0030-fix-stale-daemon-run-command-in-docs | draft | FR-001 | SPEC-0001 |
| 31 | SPEC-0031-repoint-release-and-cdn-automation | draft | FR-001 | SPEC-0001 |
| 32 | SPEC-0032-pin-gate-sort-collation | accepted | FR-001 | SPEC-0001 |
| 34 | SPEC-0034-ipc-version-skew-handshake | draft | FR-002 | SPEC-0002 |
| 35 | SPEC-0035-retire-config-native-runtime-sections | draft | FR-002 | SPEC-0002 |
| 39 | SPEC-0039-retire-unreachable-gui-host-commands | draft | FR-002 | SPEC-0036 |
| 40 | SPEC-0040-dead-export-ratchet | draft | FR-002 | SPEC-0036 |
| 41 | SPEC-0041-retire-orphaned-exec-help-text | draft | FR-002 | SPEC-0002 |
| 42 | SPEC-0042-restore-missing-reference-doc | accepted | FR-022 | SPEC-0001 |
| 43 | SPEC-0043-orcker-stack-instruction-file | draft | FR-022 | SPEC-0003 |
| 44 | SPEC-0044-fix-dangling-prd-related-documents | draft | FR-001 | SPEC-0042 |
| 45 | SPEC-0045-verifiable-human-approval | accepted | FR-001 | SPEC-0042 |
| 46 | SPEC-0046-elevate-dev-instance-socket | draft | FR-001 | SPEC-0001 |
| 47 | SPEC-0047-traceability-record-repair | approved | FR-001 | SPEC-0045 |
