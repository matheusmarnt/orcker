---
id: SPEC-0039
title: Retire the two Tauri host commands no GUI surface invokes
phase: 0
covers: [FR-002]
depends_on: [SPEC-0036]
surface:
  - apps/orcker-gui/
status: draft
attempts: 0
---

## Context

SPEC-0036's AC4 cross-check found the mirror image of the bug that spec fixed:
`daemon_installed` and `get_site_ide_overrides` are registered in
`generate_handler![]` but invoked by nothing in `src/`. Neither is in
SPEC-0002's removed set, so this pre-dates that spec.

**`job_cancel` joins them, and that one is SPEC-0036's doing.** Its only client
wrapper, `jobCancel`, was reached solely from the two site-creation wizards that
spec deleted, so the wrapper went with them and the handler is now unreachable.
Recorded here rather than left for the next reader to rediscover. Note the
sibling `job_status` is **not** in this class: it is still invoked, by
`pollJobToEnd` inside `client.ts`.

## Requirements

- R1. Establish for each whether it is dead or a caller was lost, then remove it
  or restore the caller.
- R2. Extend `tests/commandContract.test.ts` with the reverse direction, so a
  registered-but-uninvoked command fails the way a dangling one already does.
- R3. Decide the fate of `SiteCard.vue`'s WPA chip, deferred out of SPEC-0036.
  It is honest since that spec (it opens the plain WP Admin link and says so),
  but it is still gated on `v-if="site.wp_auto_login"` - a flag nothing in the
  GUI can set now that `set_wordpress_auto_login` is gone, so the control is
  unreachable on any new site. Either gate it on `site.is_wordpress` (a WP Admin
  link is useful for every WordPress site) or remove it. Which sites offer the
  link is a product call: check `docs/PRD.md` FR-020 before choosing, and
  escalate rather than improvise if it is not settled there.
