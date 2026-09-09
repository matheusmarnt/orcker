// Pure extractors over source text (no fs, no git) for the `sites-intent`
// contract SPEC-0038 R5 pins: the tray emits it, `App.vue` whitelists it,
// `SitesView.consumeIntent` branches on it, and `SitesIntent` names it. Mirrors
// the shape of `contractMirror.mjs`/`deadExports.mjs`: a scanner module plus a
// thin vitest wrapper.

/** Every `MenuAction::SitesIntent("...")` literal the tray dispatcher builds. */
export function trayEmittedIntents(traySource) {
  return [...traySource.matchAll(/SitesIntent\("([^"]+)"\)/g)].map((m) => m[1]);
}

/** Every intent literal `App.vue`'s tray-event whitelist checks against. */
export function appWhitelistedIntents(appVueSource) {
  return [...appVueSource.matchAll(/payload !== "([^"]+)"/g)].map((m) => m[1]);
}

/** Every intent literal `SitesView.consumeIntent` branches on. */
export function consumeIntentHandledIntents(sitesViewSource) {
  return [...sitesViewSource.matchAll(/intent === "([^"]+)"/g)].map((m) => m[1]);
}

/** The `SitesIntent` union's member literals. */
export function sitesIntentTypeMembers(sitesIntentSource) {
  const union = sitesIntentSource.match(/type SitesIntent = ([^;]+);/);
  if (!union) return [];
  return [...union[1].matchAll(/"([^"]+)"/g)].map((m) => m[1]);
}
