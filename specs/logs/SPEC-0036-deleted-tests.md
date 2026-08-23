# SPEC-0036 — deleted-test review list

Every test this spec removed, with the requirement that authorises it. The rule
is SPEC-0002's: a test may be deleted only when its **subject** is deleted. No
test was weakened or skipped to hide a failure. Where a surviving test merely
asserted on a removed control, the assertion was dropped and the test kept; two
tests were retargeted at the surviving half of what they covered. Both classes
are listed at the bottom - a retarget is a claim that needs checking, not a
detail to leave out.

## Died with their file

| file | tests | authority |
| --- | --- | --- |
| `apps/orcker-gui/src/views/LaravelDumpsView.spec.ts` | 3 | R2 - the view is gone |
| `apps/orcker-gui/src/components/PhpVersionPanel.test.ts` | 14 | R1 - orphan component, no mount point since SPEC-0002 |
| `apps/orcker-gui/src/components/AddExtensionModal.spec.ts` | 4 | R1 - orphan component |
| `apps/orcker-gui/src/components/ServiceOverridesModal.spec.ts` | 5 | R1 - orphan component |
| `apps/orcker-gui/src/views/SitesView.spec.ts` | 4 | R1 - every test was a WordPress auto-login test; `set_wordpress_auto_login` is gone |
| `apps/orcker-gui/src/lib/siteUrl.test.ts` | 2 | R1 - the file was entirely the `wpAdminLoginUrl` describe, and that function died with the mint branch |
| `apps/orcker-gui/src/lib/phpSettings.test.ts` | 17 | R1/R0b - `phpSettings.ts` had **zero production consumers** once `PhpVersionPanel.vue` went; verified export by export against non-test files. The module and its test die together: a suite that is its subject's only caller tests nothing reachable |

## Deleted from a surviving file

| file | test | authority |
| --- | --- | --- |
| `src/ipc/client.test.ts` | `listPhp passes through installed/default/updates` | R1 - `list_php` removed |
| `src/ipc/client.test.ts` | `updatePhp(null) sends a null version (update-all)` | R1 - `update_php` removed |
| `src/ipc/client.test.ts` | `setPhpVersionSettings sends the version and settings map` | R1 |
| `src/ipc/client.test.ts` | `setPhpDirectives sends the version and directives map` | R1 |
| `src/ipc/client.test.ts` | `setPhpPoolSettings sends the version and settings map` | R1 |
| `src/components/SiteCard.spec.ts` | `mints a token and opens the pre-authenticated link when bound and auto-login is on` | R1 - `mint_wordpress_login_token` removed |
| `src/components/SiteCard.spec.ts` | `falls back to the plain link when minting fails` | R1 - no mint, so no fallback to test |
| `src/components/SiteDetailsSidebar.spec.ts` | `turns auto-login back off and toasts when the admin users can't be read` | R1 |
| `src/components/SiteDetailsSidebar.spec.ts` | `ignores an admin-user failure that lands after the sidebar closes` | R1 |
| `src/components/SiteDetailsSidebar.spec.ts` | `reads no admin users at all while auto-login is off` | R1 |
| `src/components/SiteDetailsSidebar.spec.ts` | `writes nothing when the fetch the switch triggers fails` | R1 |
| `src/components/SiteDetailsSidebar.spec.ts` | `writes the change once a retried fetch succeeds` | R1 |
| `src/components/SiteDetailsSidebar.spec.ts` | `shows the user picker only once the admin list has loaded` | R1 |
| `src/components/SiteDetailsSidebar.spec.ts` | `mints a pre-authenticated WP Admin link when auto-login is on` | R1 |
| `src/components/SiteDetailsSidebar.spec.ts` | `toggles auto-login and changes the signed-in user` | R1 |
| `src/lib/shortcuts/registry.test.ts` | `gives the dumps window its tab-cycle and find/refresh, not navigation` | R2 - the dumps window is gone |
| `crates/orcker-ipc/tests/wire_stability.rs` | the `StarterKit` byte-shape case | R3 - explicit, deletion-only |

## Kept, with an assertion dropped

These tests cover surviving behaviour. Only the clause that referenced a removed
control was taken out; every other assertion in them still runs.

| file | test | dropped |
| --- | --- | --- |
| `src/components/SiteDetailsSidebar.spec.ts` | `renders site information and opens the site` | `toContain("8.3")`, `toContain("Dumps")` - the PHP row and Dumps button |
| `src/components/SiteDetailsSidebar.spec.ts` | `opens site actions and converts a picked web root to a relative path` | the Dumps-button click and its `showDumpsWindow` assertion |
| `src/components/SiteDetailsSidebar.spec.ts` | `provides General controls and keeps application details under Information` | the `Site PHP version` select and its `changePhp` emit |
| `src/components/SiteDetailsSidebar.spec.ts` | `returns to the General tab when reopened` | anchor swapped `PHP version` → `Web root`, a control that still exists |
| `src/components/SiteDetailsSidebar.spec.ts` | `hides WordPress-only controls on a non-WordPress site` | two assertions that a now-universally-absent control is absent - vacuous, not weakened |
| `src/components/SiteDetailsSidebar.spec.ts` | `hides the controls that don't apply to WordPress` | the Dumps-button absence check - the button is gone for every site, so the assertion could no longer fail. The web-root absence check, which is the test's actual subject, still runs. |
| `src/components/SiteCard.spec.ts` | `skips minting and opens the plain link in unbound mode…` | renamed and split into the bound/unbound plain-link pair; both still assert the opened URL |
| `src/lib/shortcuts/registry.test.ts` | `covers the main views in sidebar order, About excluded` | `toHaveLength(9)` → `(8)`, one nav target removed |

## Retargeted at the surviving half

| file | test | change |
| --- | --- | --- |
| `src/lib/shortcuts/registry.test.ts` | `surfaces digit navigation only in the main window` | `commandsForScope(all, "dumps")` → `"mails"`. The assertion (an auxiliary window gets no `nav:` commands) is unchanged and still meaningful; only the scope it names was deleted by R2, and `"mails"` is the one that remains. |
| `src/lib/shortcuts/registry.test.ts` | `opens the viewer windows via their chords` → `opens the viewer window via its chord` | The dumps half (`open-dumps` chord + `openDumpsWindow` call) went with the command; the mail half still asserts chord and dispatch. |
| `src/components/SiteDetailsSidebar.spec.ts` | `opens the plain WP Admin link when auto-login is off` | Lost its `expect(mintWordPressLoginToken).not.toHaveBeenCalled()` clause - the mock no longer exists. The `openInBrowser` assertion on the plain URL, which is the point of the test, still runs. |
