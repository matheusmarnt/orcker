import { readFileSync, readdirSync, statSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";

import dangling from "../src/ipc/dangling-commands.json";

/**
 * Every Tauri command the GUI invokes must be registered in `generate_handler!`.
 *
 * Nothing else in the toolchain checks this. `vue-tsc` does not validate command
 * name strings, the unit tests mock the IPC client, and the Rust side is
 * internally consistent whether or not the GUI agrees with it. SPEC-0002 removed
 * 49 handlers while their call sites stayed, and every automated check stayed
 * green - a user only finds out by clicking.
 *
 * `dangling-commands.json` is a ratchet, not a permission slip: the known 49 are
 * listed so this test can fail on the *next* one. Emptying it is SPEC-0036's
 * acceptance criterion.
 */

// vitest runs with the package root as cwd; anchoring there avoids the
// import.meta.url path differences between a dev run and CI.
const ROOT = process.cwd();
const SRC = join(ROOT, "src");
const MAIN_RS = join(ROOT, "src-tauri", "src", "main.rs");

function sourceFiles(dir: string): string[] {
  return readdirSync(dir).flatMap((entry) => {
    const path = join(dir, entry);
    if (statSync(path).isDirectory()) return sourceFiles(path);
    return /\.(ts|vue)$/.test(entry) ? [path] : [];
  });
}

function registeredCommands(): Set<string> {
  const main = readFileSync(MAIN_RS, "utf8");
  const block = /generate_handler!\[([\s\S]*?)\]/.exec(main);
  if (!block) throw new Error("generate_handler![] not found in main.rs");
  return new Set(
    [...block[1].matchAll(/(?:[a-z_]+::)*([a-z_0-9]+)\s*(?=,|\])/g)].map((m) => m[1]),
  );
}

function invokedCommands(): Map<string, string> {
  const found = new Map<string, string>();
  for (const file of sourceFiles(SRC)) {
    const text = readFileSync(file, "utf8");
    for (const m of text.matchAll(/(?:invoke|call)\s*(?:<[^>]*>)?\s*\(\s*"([a-z_0-9]+)"/g)) {
      const line = text.slice(0, m.index).split("\n").length;
      if (!found.has(m[1])) found.set(m[1], `${file.slice(ROOT.length + 1)}:${line}`);
    }
  }
  return found;
}

describe("Tauri command contract", () => {
  it("invokes no command the daemon host does not register", () => {
    const registered = registeredCommands();
    const known = new Set(dangling.known_dangling);
    const unexpected = [...invokedCommands()]
      .filter(([name]) => !registered.has(name) && !known.has(name))
      .map(([name, where]) => `${name} (${where})`);
    expect(unexpected, "new dangling Tauri command - register it or remove the call site").toEqual(
      [],
    );
  });

  it("keeps the dangling list a ratchet: no entry that is already registered", () => {
    const registered = registeredCommands();
    const stale = dangling.known_dangling.filter((name) => registered.has(name));
    expect(stale, "these are registered again - drop them from dangling-commands.json").toEqual([]);
  });

  it("keeps the dangling list honest: every entry is still invoked somewhere", () => {
    const invoked = invokedCommands();
    const orphans = dangling.known_dangling.filter((name) => !invoked.has(name));
    expect(orphans, "no longer invoked - drop them from dangling-commands.json").toEqual([]);
  });
});
