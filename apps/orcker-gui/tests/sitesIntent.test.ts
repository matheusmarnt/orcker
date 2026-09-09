import { readFileSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";

import {
  appWhitelistedIntents,
  consumeIntentHandledIntents,
  sitesIntentTypeMembers,
  trayEmittedIntents,
} from "./sitesIntent.mjs";

describe("sitesIntent extractors", () => {
  it("trayEmittedIntents finds every SitesIntent(...) literal", () => {
    const src = `
      "sites:link" => MenuAction::SitesIntent("link"),
      "sites:park" => MenuAction::SitesIntent("park"),
    `;
    expect(trayEmittedIntents(src)).toEqual(["link", "park"]);
  });

  it("appWhitelistedIntents finds every payload !== literal", () => {
    const src = `if (event.payload !== "link" && event.payload !== "park") { return; }`;
    expect(appWhitelistedIntents(src)).toEqual(["link", "park"]);
  });

  it("consumeIntentHandledIntents finds every intent === literal", () => {
    const src = `if (intent === "link") linkOpen.value = true; else if (intent === "park") void onPark();`;
    expect(consumeIntentHandledIntents(src)).toEqual(["link", "park"]);
  });

  it("sitesIntentTypeMembers reads the union's member literals", () => {
    const src = `export type SitesIntent = "link" | "park";`;
    expect(sitesIntentTypeMembers(src)).toEqual(["link", "park"]);
  });
});

/**
 * The four places that must agree on the `sites-intent` payload set. Nothing
 * before this checked them against each other: SPEC-0036 shipped a REWORK
 * because the tray kept emitting `"create"` after `consumeIntent` stopped
 * handling it - a menu item that showed the window and did nothing, with
 * `[gate] OK` throughout.
 */
describe("sites-intent contract", () => {
  const ROOT = process.cwd();

  it("the tray, App.vue's whitelist, SitesView.consumeIntent, and SitesIntent agree", () => {
    const tray = trayEmittedIntents(
      readFileSync(join(ROOT, "src-tauri", "src", "tray.rs"), "utf8"),
    );
    const whitelist = appWhitelistedIntents(readFileSync(join(ROOT, "src", "App.vue"), "utf8"));
    const handled = consumeIntentHandledIntents(
      readFileSync(join(ROOT, "src", "views", "SitesView.vue"), "utf8"),
    );
    const typeMembers = sitesIntentTypeMembers(
      readFileSync(join(ROOT, "src", "lib", "shortcuts", "sitesIntent.ts"), "utf8"),
    );

    for (const set of [tray, whitelist, handled, typeMembers]) {
      expect(set.length).toBeGreaterThan(0);
    }
    const sorted = (xs: string[]) => [...new Set(xs)].sort();
    expect(sorted(whitelist)).toEqual(sorted(tray));
    expect(sorted(handled)).toEqual(sorted(tray));
    expect(sorted(typeMembers)).toEqual(sorted(tray));
  });
});
