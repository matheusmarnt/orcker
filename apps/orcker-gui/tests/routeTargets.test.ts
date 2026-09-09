import { readFileSync, readdirSync, statSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";

import { VIEW_TARGETS } from "../src/lib/shortcuts/registry";

/**
 * Every navigation target must resolve to a route `router.ts` defines.
 *
 * No other check sees this class. vue-router does not fail a build on an
 * unmatched `to`, the Tauri command contract test only looks at command names,
 * and nothing before this scanned the tray's own `nav:` ids or the window url in
 * `tauri.conf.json` - which is how a tray item pointing at a deleted route
 * survived five review rounds of SPEC-0002.
 *
 * The scan deliberately covers **both sides**: the Vue/TS navigation forms and
 * the Rust tray's `nav:` menu ids. It does not scan the tray's `emit("navigate", ...)`
 * call sites directly (SPEC-0038 R3) - `tray.rs`'s own tests now prove every
 * `nav:` id dispatches to the `Navigate` action with its route intact, which is
 * the property this file cannot see once a payload stops being a string literal.
 *
 * Lives in `tests/` rather than beside the code because it needs `node:fs`, and
 * `tsconfig.json` typechecks `src/**` without node types.
 */

const ROOT = process.cwd();

function definedRoutes(): Set<string> {
  const src = readFileSync(join(ROOT, "src", "router.ts"), "utf8");
  return new Set([...src.matchAll(/path:\s*"([^"]+)"/g)].map((m) => m[1]));
}

function sourceFiles(dir: string, exts: RegExp): string[] {
  return readdirSync(dir).flatMap((entry) => {
    const path = join(dir, entry);
    if (statSync(path).isDirectory()) return sourceFiles(path, exts);
    return exts.test(entry) ? [path] : [];
  });
}

/** Every literal route path the front end navigates to, with where it came from. */
function frontEndTargets(): [string, string][] {
  const out: [string, string][] = [];
  const patterns = [
    /\bto:\s*"(\/[^"]*)"/g, // { to: "/x" } in a nav array
    /\bto="(\/[^"]*)"/g, //    <RouterLink to="/x">
    /router\.push\(\s*"(\/[^"]*)"/g,
    // No `path:` pattern here - VIEW_TARGETS is imported and checked directly
    // below. A regex for it also matched filesystem paths in test fixtures.
  ];
  for (const file of sourceFiles(join(ROOT, "src"), /\.(ts|vue)$/)) {
    if (file.endsWith(join("src", "router.ts"))) continue; // the definition itself
    const text = readFileSync(file, "utf8");
    for (const p of patterns) {
      for (const m of text.matchAll(p)) {
        const line = text.slice(0, m.index).split("\n").length;
        out.push([m[1], `${file.slice(ROOT.length + 1)}:${line}`]);
      }
    }
  }
  return out;
}

/** Route paths the Rust tray navigates to, via its `nav:` menu ids. */
function trayTargets(): [string, string][] {
  const file = join(ROOT, "src-tauri", "src", "tray.rs");
  const text = readFileSync(file, "utf8");
  const out: [string, string][] = [];
  for (const m of text.matchAll(/"nav:(\/[^"]*)"/g)) {
    const line = text.slice(0, m.index).split("\n").length;
    out.push([m[1], `src-tauri/src/tray.rs:${line}`]);
  }
  return out;
}

/** Route paths a Tauri window's `url` navigates to (`index.html#/route`). */
function tauriConfTargets(): [string, string][] {
  const file = join(ROOT, "src-tauri", "tauri.conf.json");
  const conf = JSON.parse(readFileSync(file, "utf8"));
  const windows: unknown[] = conf?.app?.windows ?? [];
  const out: [string, string][] = [];
  windows.forEach((w, i) => {
    const url = (w as { url?: unknown }).url;
    if (typeof url !== "string") return;
    const hash = url.indexOf("#");
    if (hash === -1) return;
    out.push([url.slice(hash + 1), `src-tauri/tauri.conf.json:app.windows[${i}].url`]);
  });
  return out;
}

function dangling(targets: [string, string][], defined: Set<string>): string[] {
  return targets.filter(([path]) => !defined.has(path)).map(([p, where]) => `${p} (${where})`);
}

describe("navigation targets", () => {
  it("navigates only to paths the router defines, from the front end", () => {
    const targets: [string, string][] = [
      ...frontEndTargets(),
      ...VIEW_TARGETS.map((v): [string, string] => [v.path, "lib/shortcuts/registry.ts"]),
    ];
    expect(dangling(targets, definedRoutes())).toEqual([]);
  });

  it("navigates only to paths the router defines, from the Rust tray", () => {
    const targets = trayTargets();
    expect(targets.length).toBeGreaterThan(0);
    expect(dangling(targets, definedRoutes())).toEqual([]);
  });

  it("navigates only to paths the router defines, from tauri.conf.json", () => {
    const targets = tauriConfTargets();
    expect(targets.length).toBeGreaterThan(0);
    expect(dangling(targets, definedRoutes())).toEqual([]);
  });

  it("keeps the digit chords an unbroken run from 1, with no duplicates", () => {
    const digits = VIEW_TARGETS.filter((v) => v.digit !== undefined).map((v) => v.digit!);
    expect(new Set(digits).size).toBe(digits.length);
    expect([...digits].sort((a, b) => a - b)).toEqual(
      Array.from({ length: digits.length }, (_, i) => i + 1),
    );
  });
});
