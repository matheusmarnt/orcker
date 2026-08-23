---
id: SPEC-0038
title: Give the Tauri host real coverage, not just a text scan
phase: 0
covers: [FR-002]
depends_on: [SPEC-0002, SPEC-0036]
surface:
  - apps/orcker-gui/
status: draft
attempts: 0
---

## Context

`apps/orcker-gui/src-tauri/` has **no behavioural test**. The GUI suite mocks the
Tauri boundary, so nothing in the gate has ever executed the tray, the menu-event
dispatcher or the command handlers.

That is why a tray item pointing at a route SPEC-0002 had deleted survived five
supervisor rounds of that spec: `[gate] OK` every time. It was eventually caught
by reading the source, not by running anything.

SPEC-0002 closed the instance and added a **text scan**
(`apps/orcker-gui/tests/routeTargets.test.ts`) that greps `tray.rs` for
`"nav:/x"` and `emit("navigate", "/x")` literals. The supervisor accepted that as
sufficient for SPEC-0002 - requiring real coverage there would have added an
acceptance criterion that spec never expressed - while stating plainly that **no
spec owns the gap**. SPEC-0037 does not: its list holds two `src-tauri` rows, both
`default_php_choices_*`, tests of a function SPEC-0002 deleted, already
dispositioned `drop`.

A text scan breaks the moment an id stops being a literal.

## Requirements

- R1. Unit-test the tray's pure parts: the menu-id dispatcher (id in, action out),
  `NAV_ITEMS`, badge and state derivation. Restructure only as far as testability
  requires; the Tauri handle stays behind a seam.
- R2. Cover the `commands.rs` handlers whose logic is not a straight IPC pass
  through, with a faked IPC client.
- R3. Once the dispatcher is tested by behaviour, narrow or retire
  `routeTargets.test.ts`'s tray half. Two guards for one property, one of which
  can silently stop matching, is worse than one that runs.
- R4. Widen the route guard to the references it does not see today:
  `src-tauri/tauri.conf.json` (`index.html#/mails-viewer`) and `main.rs`'s
  `WebviewUrl::App`. Both resolve now; nothing would catch a break.
- R5. Guard the **event-payload** contract, not just route ids. `tray.rs` emits
  `sites-intent` payloads that `App.vue` whitelists and `SitesView.consumeIntent`
  branches on; nothing checks the three agree. SPEC-0036 shipped a REWORK because
  of exactly this: it deleted the site-creation wizards, so `consumeIntent`
  stopped handling `"create"`, but the tray kept a "New Laravel site…" item
  emitting it - a menu entry that showed the window and then did nothing, with
  `[gate] OK` throughout. The emitted set, the whitelist and the handled set must
  be pinned to `SitesIntent`.

## Design & contracts

The seam is the point. `tray.rs` mixes decisions (which id maps to which action)
with effects (menu building, `emit`). Only the decisions need testing, so extract
them rather than reaching for a Tauri test harness.

## Test plan

- Each new test gets RED evidence: break the behaviour, watch it fail, restore.
- The gate stays green throughout.

## Acceptance checklist

- [ ] AC1 The tray menu-id dispatcher has behavioural tests, RED recorded
- [ ] AC2 `cargo test -p orcker-gui` runs a non-zero number of tests for the
      tray and command modules -> evidence: the count, before and after
- [ ] AC3 The route guard covers `tauri.conf.json` and `main.rs`; RED recorded by
      pointing one of them at a route that does not exist
- [ ] AC4 Every `sites-intent` payload the tray emits is a member of `SitesIntent`
      and is handled by `SitesView.consumeIntent`; RED recorded by emitting a
      payload nothing consumes (the SPEC-0036 failure, reproduced)
- [ ] AC5 `scripts/gate.sh specs/SPEC-0038-*.md` passes

## Out of scope

End-to-end GUI testing (a driver, a real window). This is unit coverage of the
host's decisions.
