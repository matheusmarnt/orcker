import { readFileSync, readdirSync, statSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";

import { VIEW_TARGETS } from "../src/lib/shortcuts/registry";

/**
 * Every navigation target must resolve to a route `router.ts` defines.
 *
 * No other check sees this class. vue-router does not fail a build on an
 * unmatched `to`, the Tauri command contract test only looks at command names,
 * and `src-tauri/` has no test at all - which is how a tray item pointing at a
 * deleted route survived five review rounds of SPEC-0002.
 *
 * The scan deliberately covers **both sides**: the Vue/TS navigation forms and
 * the Rust tray, which emits `navigate` events the app turns into
 * `router.push`. A guard that reads only the language it is written in is
 * theatre.
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

/** Route paths the Rust tray navigates to, via `nav:` ids and `navigate` emits. */
function trayTargets(): [string, string][] {
  const file = join(ROOT, "src-tauri", "src", "tray.rs");
  const text = readFileSync(file, "utf8");
  const out: [string, string][] = [];
  const patterns = [/"nav:(\/[^"]*)"/g, /emit\(\s*"navigate"\s*,\s*"(\/[^"]*)"/g];
  for (const p of patterns) {
    for (const m of text.matchAll(p)) {
      const line = text.slice(0, m.index).split("\n").length;
      out.push([m[1], `src-tauri/src/tray.rs:${line}`]);
    }
  }
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
    expect(dangling(trayTargets(), definedRoutes())).toEqual([]);
  });

  it("keeps the digit chords an unbroken run from 1, with no duplicates", () => {
    const digits = VIEW_TARGETS.filter((v) => v.digit !== undefined).map((v) => v.digit!);
    expect(new Set(digits).size).toBe(digits.length);
    expect([...digits].sort((a, b) => a - b)).toEqual(
      Array.from({ length: digits.length }, (_, i) => i + 1),
    );
  });
});
