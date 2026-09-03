---
id: SPEC-0053
title: A linked project can claim any port, including Orcker's own listener
phase: 0
covers: [FR-013, FR-021]
depends_on: [SPEC-0006]
surface:
  - crates/orcker-config/
  - bin/orckerd/
status: accepted
attempts: 0
---

## Context

SPEC-0006 shipped `orcker link --port <n>` with no validation of `<n>`, and
routed the resulting project through `plan_proxies` without the routing-loop
guard every other proxy path uses. Three holes, found by inspection and
reproduced against a daemon that had actually bound its listeners:

**H1 — the routing-loop guard is bypassed.** `bin/orckerd/src/ipc_server.rs`
applies `is_self_forward` to `Request::AddProxy` and `Request::AddProxyRule`
only. `Request::LinkProject` reaches `handle_link_project` directly. Same
daemon, same upstream, opposite outcomes:

```
$ orcker proxy add loopb http://127.0.0.1:8443
error (InvalidPath): proxy target points at orcker's own listening port (routing loop)

$ orcker link /tmp/orcker-dev/projects/loop --name loop --port 8443
linked loop -> http://loop.test (upstream 127.0.0.1:8443)
```

`loop.test` now forwards into Orcker's own HTTPS listener, which re-resolves it
and forwards again.

**H2 — `--port` is not range-checked.** R2 fixes the range at
`20000..=29999`, but `link::plan_link` only checks that no other project holds
the port. Port 0, a privileged port, or a port outside the range are all
accepted, which also makes `ports::taken_ports` an incomplete account of what is
allocated.

**H3 — `Config::validate` has no project invariants.** `parse::validate` calls
`validate_proxies` but nothing for `[[projects]]`. A hand-edited or
partially-written config with two projects sharing a name and a port loads
clean:

```
ACCEPTED: 2 projects, names=["dup", "dup"], ports=[20000, 20000]
```

`plan_proxies` then drops the second with a warning, so the entry is invisible
while still consuming its port in `taken_ports`, and `apply_unlink`'s
`retain(|p| p.name() != name_lc)` removes both at once. `ProxyNameCollision` is
the standing precedent for exactly this class of check.

None of this is reachable by a user who never passes `--port`, which is why
SPEC-0006's acceptance run did not surface it.

## Requirements

- R1. `LinkProject` refuses an upstream that points at a bound Orcker listener,
      reusing `is_self_forward` rather than a second implementation, and returns
      the same `InvalidPath` code `AddProxy` returns.
- R2. `plan_link` rejects a `--port` outside `FIRST_PROJECT_PORT..=LAST_PROJECT_PORT`
      with a typed error naming the range. The spike flow SPEC-0006 needed the
      flag for is inside that range, so nothing legitimate regresses.
- R3. `Config::validate` gains project invariants, mirroring `validate_proxies`:
      no two projects share a name, no two share a port, no project name
      collides with a linked site or a proxy name, and no port is 0.
- R4. The new `ValidateErrorReason` variants are additive; existing config files
      that satisfy the invariants keep loading byte-identically.

## Test plan

- Pure, table-driven in `orcker-config`: the four R3 invariants, each with a
  passing and a failing config, asserted on the typed reason.
- `orckerd`: `plan_link` rejects an out-of-range port and a zero port; a
  `LinkProject` whose port is a bound listener is refused with `InvalidPath`.

## Acceptance checklist

- [x] AC1 (R1) `orcker link --port <bound listener>` is refused with the same
      code and message class as `orcker proxy add` -> test in `bin/orckerd`
- [x] AC2 (R2) out-of-range and zero ports are refused -> test:
      `link::tests::port_outside_the_range_is_rejected`
- [x] AC3 (R3) the four invariants -> test:
      `orcker_config::parse::tests::validate_projects_matrix`
- [x] AC4 (R4) `tests/toml_byte_shape.rs` unchanged and green
- [x] AC5 `scripts/gate.sh specs/SPEC-0053-*.md` passes

## Out of scope

Making the range configurable (SPEC-0006 already excluded it). Reserving ports
against configured service instances: the TCP probe covers a running service,
and a stopped one is the `PortReserved` problem the services code already owns.

## Agent notes

`is_self_forward` needs the daemon's **bound** ports, so it lives in the handler,
not in the pure `plan_link`. Note that it is correctly inert when the daemon
bound nothing (`BothPairsFailed`) - an early reproduction attempt read that as a
broken guard; check `state.http.bound` before concluding anything from it.
