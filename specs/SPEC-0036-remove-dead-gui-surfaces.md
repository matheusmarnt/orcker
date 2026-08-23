---
id: SPEC-0036
title: Remove the GUI surfaces SPEC-0002 left without a backend
phase: 0
covers: [FR-002]
depends_on: [SPEC-0002]
surface:
  - apps/orcker-gui/
  - crates/orcker-ipc/
status: accepted
attempts: 3
---

## Context

SPEC-0002 deleted daemon handlers and IPC requests but its R5 named only two GUI
pages, so several surfaces survived pointing at commands that no longer exist.
They are invisible to every automated check: `npm run test` passes because the
IPC client is mocked, `npm run build` passes because TypeScript does not validate
Tauri command names, and the Rust side is internally consistent. The failure only
appears when a user clicks.

Found by SPEC-0002's amended AC3, a symbol-level scan of the whole tree against
the set of identifiers that spec removed
(`specs/logs/SPEC-0002-removed-symbols.txt`). Widened from "dead dumps UI" to
cover the whole class after that scan found the site-creation wizards.

**The site-creation wizards are the serious half.** `SitesView.vue:944` is a live,
routed page that renders `CreateLaravelWizard` and `CreateWordPressWizard`. Both
call `createSite()`, which invokes the Tauri command `create_site` - deleted by
SPEC-0002 R2, because site creation returns via containers under PRD FR-020.
`CreateWordPressWizard` additionally calls `listServices()`. A user who opens
Sites, starts a wizard and submits gets an IPC error.

## Requirements

- R0b. "Everything only they reach" (R1) needs a check that can see it. The
  scan named under Design & contracts cannot: it holds the symbols **SPEC-0002**
  removed, so by construction it finds SPEC-0002's orphans and never one this
  spec creates. `apps/orcker-gui/tests/dead-export-delta.mjs` is the missing
  half - it reports `dead(working tree) \ dead(HEAD)`, the exports that lost
  their last consumer to this diff. Every name it reports is either deleted or
  gets a written justification in the cycle log; a silent keep is a fail.
  Scoped to the delta on purpose: ~96 exports were already dead at `HEAD` and
  are SPEC-0037's debt, not this spec's.
- R0. The authoritative work list is
  `apps/orcker-gui/src/ipc/dangling-commands.json`: 49 Tauri commands the GUI
  still invokes after SPEC-0002 removed their handlers, 24 of them reached from
  live callers in 17 files. Derive the call sites with the cross-check in
  `tests/commandContract.test.ts`; do not work from the requirement list below alone,
  which names the clusters, not every site.
- R1. Remove the site-creation wizards and everything only they reach:
  `components/site-create/CreateLaravelWizard.vue`,
  `CreateWordPressWizard.vue`, their mount points in `SitesView.vue`, the
  `createSite` / `listServices` / `listPhp` client functions, and the
  `listPhp` case in `ipc/client.test.ts`. `SitesView` itself stays: link, park,
  secure and the sites table are all live - but its `setPhp` call (line ~160) and
  `setWordpressAutoLogin` call (line ~467) invoke deleted commands and must go
  with the controls that trigger them. Other named callers: `WelcomeView.vue`
  (`availablePhp`), `GeneralView.vue` (`dumpsStatus`), `SideNav.vue` (`listPhp`),
  `PhpVersionPanel.vue`, `AddExtensionModal.vue`, `ServiceOverridesModal.vue`,
  `SiteDetailsSidebar.vue`, `lib/wpAdmin.ts`, `composables/useFallbackPorts.ts`,
  and the `.spec.ts` files that mock them.
- R2. Remove the dumps UI: `LaravelDumpsView.vue`, `DumpsWindowView.vue`, the
  `/dumps` and `/dumps-window` routes, the dumps shortcut, the `App.vue` branch,
  `showDumpsWindow` and the dumps client functions, plus
  `LaravelDumpsView.spec.ts`.
- R3. Remove the now-unreachable wire types in `crates/orcker-ipc/src/create.rs`
  (`CreateSiteSpec`, `Framework`, `LaravelOptions`, `WordPressOptions`,
  `StarterKit`, `Testing`, `JsRuntime`, `AuthProvider`, `Database`,
  `WordPressDatabase`, `WordPressDbEngine`) and their `lib.rs` re-exports. `JobId`
  and `JobState` **stay** - `Request::JobStatus` and the streamed tool installs
  still use them. Delete the matching `wire_stability.rs` tests, deletion-only,
  under the same rule SPEC-0002 recorded.
- R4. Anything a removed surface uniquely owned in `ipc/types.ts` goes with it.
- R5. A user-facing entry point may be **stubbed** instead of removed if the
  feature returns in phase 1 (PRD FR-020 rebuilds `orcker new` via containers);
  a stub must say so, and must not call a command that does not exist.

## Design & contracts

Completeness is checked from **both ends**, because either alone is blind:

1. Inherited rot - SPEC-0002's AC3 pattern file, re-run at the end:
   `rg -f specs/logs/SPEC-0002-removed-symbols.txt crates bin apps --glob '!dist'`
   must return no matches. Build artefacts under `apps/orcker-gui/dist/` are
   excluded - they are regenerated, not source.
2. Rot this spec creates - `node tests/dead-export-delta.mjs` from
   `apps/orcker-gui`, per R0b. Check 1 cannot detect this class at all: its
   pattern file lists SPEC-0002's symbols.

An absolute "no dead exports anywhere" gate was tried and rejected on its own
output: it needs a ~84-entry allowlist of inherited debt, it flags same-file
helpers that are still called (`jobStatus`, reached by `pollJobToEnd`), and it
demands deleting the `ipc/types.ts` wire-contract mirror - including the
`JobState` that R3 orders kept. A standing ratchet needs that exemption designed
deliberately and belongs to its own spec, not to this one.

## Test plan

- GUI `npm run test` and `npm run build` green.
- `cargo test --workspace` green after R3.
- Manual evidence: open Sites in the built GUI; no control offers an action whose
  command was deleted.

## Acceptance checklist

- [ ] AC1 `apps/orcker-gui/src/ipc/dangling-commands.json` has an empty
      `known_dangling` array, and `tests/commandContract.test.ts` passes with it empty:
      every Tauri command the GUI invokes is registered in `generate_handler![]`
- [ ] AC2 GUI `npm run test` and `npm run build` green
- [ ] AC3 `cargo test --workspace` green
- [ ] AC4 No route, button or menu item in the built GUI invokes a Tauri command
      that is not registered -> evidence: the invoke-name list cross-checked
      against `src-tauri/src/main.rs`'s handler list, in the cycle log
- [ ] AC5 `scripts/gate.sh specs/SPEC-0036-*.md` passes
- [ ] AC6 `node tests/dead-export-delta.mjs` (from `apps/orcker-gui`) reports every
      export this diff orphaned, and each one is either deleted or carries a
      written justification in the cycle log -> evidence: the script's output,
      before and after, in the log

## Out of scope

Rebuilding site creation - that is PRD FR-020, phase 1. This spec removes or
stubs the broken entry points; it does not replace them.
