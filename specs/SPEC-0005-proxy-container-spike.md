---
id: SPEC-0005
title: Phase-0 spike - serve a containerized Laravel app at https://spike.test through the inherited proxy
phase: 0
covers: [FR-003]
depends_on: [SPEC-0002, SPEC-0037]
surface:
  - docs/spike/
  - crates/orcker-core/
  - crates/orcker-config/
  - bin/orckerd/
  - bin/orcker/
status: draft
attempts: 0
---

## Context

The product thesis in one experiment: the inherited rustls proxy + local CA +
embedded DNS can front a project running in containers. The reference stack
(app + nginx + postgres from `referenciadockerlaravel.md`) is hand-written for
this spike — generation comes later. Yerd already ships an HTTP forward path
(`forward/http.rs`, used by its custom-proxies feature) and WebSocket upgrades
(`forward/upgrade.rs`); the spike leans on them with the smallest possible code
delta. Findings feed SPEC-0006 and the Phase-1 routing specs.

## Requirements

- R1. `docs/spike/PHASE0-SPIKE.md`: a reproducible runbook containing the
  hand-written reference stack (compose + Dockerfile.dev + nginx confs +
  supervisord + init.sql, faithful to the reference doc, nginx published on
  `127.0.0.1:18080`), the exact commands, and a results section.
- R2. Minimal code delta (only if the inherited custom-proxy/route mechanism
  cannot already express it): allow registering a site whose upstream is
  `http://127.0.0.1:<port>` via CLI. Acceptable UX for the spike:
  `orcker link spike --upstream 127.0.0.1:18080` or the inherited
  proxy-route command — whichever needs less new code. Any new IPC message is
  additive.
- R3. HTTPS: `orcker secure spike` issues the leaf; browser shows a valid
  padlock (CA already trusted via `elevate`).
- R4. WebSockets: Vite dev server (5173) reachable through the stack as the
  reference doc prescribes; HMR round-trip works (documented evidence; a
  direct-port fallback is acceptable if proxying HMR needs Phase-1 work — then
  recorded as a finding, not silently skipped).
- R5. Results section answers, with evidence: request latency overhead through
  the proxy (rough curl timing), any header/websocket issues, UID/permission
  behavior of the mounted volume, and a findings list feeding SPEC-0006.

## Design & contracts

This is an evidence-driven spec: the deliverable is the runbook + findings +
the minimal enabling delta. No new dependencies. No template generation.

## Test plan

- Unit: only for whatever small routing/config delta R2 introduces (table-driven
  on the router mapping).
- E2E/manual (the core of this spec — unavoidable, that is the point of a
  spike): the runbook executed top to bottom on Linux; macOS execution noted if
  available, else deferred to Phase 1 with a note.

## Acceptance checklist

- [ ] AC1 Runbook executes from scratch to a green padlock ->
      evidence: terminal transcript + screenshot refs in `docs/spike/`
- [ ] AC2 `https://spike.test` returns 200 with a leaf issued by the local CA ->
      evidence: `curl -sv https://spike.test` output (issuer line) in the runbook
- [ ] AC3 Laravel welcome page served by nginx->php-fpm inside containers
      (supervisor programs running: fpm + horizon + schedule) ->
      evidence: `docker compose ps` + supervisor status in the runbook
- [ ] AC4 WebSocket/HMR result documented (works, or finding recorded) ->
      evidence: runbook results section
- [ ] AC5 Findings list present and copied into SPEC-0006's context if it
      changes that spec -> evidence: runbook + specs/DECISIONS.md entry
- [ ] AC6 `scripts/gate.sh specs/SPEC-0005-*.md` passes

## Out of scope

Stack generation (SPEC-0003/0007); port allocation (SPEC-0006); services
network `development` beyond what the hand-written compose declares; macOS
blocking validation; performance targets (NFR-02 is Phase 1).

## Agent notes

Read first: `docs/guide/proxies.md` (inherited custom-proxy feature — the
spike's likely zero-code path), `crates/orcker-proxy/src/forward/http.rs` and
`upgrade.rs`, `crates/orcker-core/src/{router,route_rule,proxy}.rs`. Pitfalls:
the app container must run with the host UID (reference doc build arg) or the
mounted volume ends up root-owned; Horizon needs Redis — for the spike, add a
redis service inside the hand-written compose (the shared `development`
network arrives in Phase 1); remember `docker network create development` is
NOT needed if the spike compose keeps everything internal.
