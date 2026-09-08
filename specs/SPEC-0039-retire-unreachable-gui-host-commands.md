---
id: SPEC-0039
title: Retire the two Tauri host commands no GUI surface invokes
phase: 0
covers: [FR-002]
depends_on: [SPEC-0036]
surface:
  - apps/orcker-gui/
status: in_progress
attempts: 0
---

## Context

SPEC-0036's AC4 cross-check found `daemon_installed` registered in
`generate_handler![]` but invoked by nothing in `src/`. It is not in
SPEC-0002's removed set, so this pre-dates that spec.

**`job_cancel` joins it, and that one is SPEC-0036's doing.** Its only client
wrapper, `jobCancel`, was reached solely from the two site-creation wizards that
spec deleted, so the wrapper went with them and the handler is now unreachable.
Recorded here rather than left for the next reader to rediscover. Note the
sibling `job_status` is **not** in this class: it is still invoked, by
`pollJobToEnd` inside `client.ts`.

**`get_site_ide_overrides` was flagged alongside them at draft time but is not
dead.** `SiteDetailsSidebar.vue` calls it live (`getSiteIdeOverrides()`,
alongside `getPreferredIde()`) to feed the per-site IDE picker, which also
still calls `setSiteIdeOverride()`. The cross-check's premise was wrong for
this one command; it is out of scope here.

## Requirements

- R1. `daemon_installed` and `job_cancel` are dead (no `src/` caller for
  either) - remove both the command and its `generate_handler![]` registration.
- R2. Extend `tests/commandContract.test.ts` with the reverse direction, so a
  registered-but-uninvoked command fails the way a dangling one already does.
- R3. Decide the fate of `SiteCard.vue`'s WPA chip, deferred out of SPEC-0036.
  It is honest since that spec (it opens the plain WP Admin link and says so),
  but it is still gated on `v-if="site.wp_auto_login"` - a flag nothing in the
  GUI can set now that `set_wordpress_auto_login` is gone, so the control is
  unreachable on any new site. Decided: gate it on `site.is_wordpress` instead
  - a WP Admin link is useful for every WordPress site, and `openWpAdmin()`
  already opens the plain (non-auto-login) login screen regardless of
  `wp_auto_login`. `docs/PRD.md` FR-020, cited when this spec was drafted, is
  `orcker new` and does not cover this; no FR does. Escalated and decided with
  the human rather than improvised, per this spec's own instruction.
