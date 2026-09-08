/**
 * SPEC-0040: the standing gate. `dead-export-delta.mjs` reports what *this
 * diff* orphaned (cycle evidence, depends on HEAD); this test polices the
 * absolute count so it can only shrink.
 *
 * A dead export must be either:
 * - exempt: an `src/ipc/types.ts` name that still mirrors a live
 *   `crates/orcker-ipc::Response` variant (computed fresh every run - see
 *   `contractMirror.mjs` - so the exemption can never go stale), or
 * - listed in `dead-export-allowlist.txt` with a written reason.
 *
 * Both directions are checked: an unlisted, non-exempt dead export fails (a
 * new one was introduced), and an allowlist entry that is no longer dead also
 * fails (the list must shrink, not just never grow).
 */
import { execFileSync } from "node:child_process";
import { readdirSync, readFileSync, statSync } from "node:fs";
import { dirname, join, relative } from "node:path";
import { fileURLToPath } from "node:url";
import { expect, it } from "vitest";
import { isContractMirror } from "./contractMirror.mjs";
import { deadExports } from "./deadExports.mjs";

const HERE = dirname(fileURLToPath(import.meta.url));
const ROOT = execFileSync("git", ["rev-parse", "--show-toplevel"], { encoding: "utf8" }).trim();
const SUBTREE = "apps/orcker-gui/src";
const TYPES_FILE = "apps/orcker-gui/src/ipc/types.ts";
const RUST_DIR = "crates/orcker-ipc/src";
const ALLOWLIST_FILE = join(HERE, "dead-export-allowlist.txt");

function walk(dir, matches) {
  return readdirSync(dir).flatMap((entry) => {
    const path = join(dir, entry);
    if (statSync(path).isDirectory()) return walk(path, matches);
    return matches.test(entry) ? [path] : [];
  });
}

function workingTreeFiles() {
  const files = new Map();
  for (const path of walk(join(ROOT, SUBTREE), /\.(ts|vue)$/)) {
    files.set(relative(ROOT, path), readFileSync(path, "utf8"));
  }
  return files;
}

function rustSource() {
  return walk(join(ROOT, RUST_DIR), /\.rs$/)
    .map((p) => readFileSync(p, "utf8"))
    .join("\n");
}

/** `path<TAB>name` lines; `#`-led and blank lines are comments. */
function parseAllowlist() {
  const entries = new Set();
  for (const line of readFileSync(ALLOWLIST_FILE, "utf8").split("\n")) {
    const trimmed = line.trim();
    if (!trimmed || trimmed.startsWith("#")) continue;
    const [path, name] = trimmed.split("\t");
    entries.add(`${path}\t${name}`);
  }
  return entries;
}

it("has no new dead GUI export outside the allowlist, and no stale allowlist entry", () => {
  const dead = deadExports(workingTreeFiles());
  const typesSource = readFileSync(join(ROOT, TYPES_FILE), "utf8");
  const rust = rustSource();

  const unlisted = [];
  const liveKeys = new Set();
  for (const [name, home] of dead) {
    if (home === TYPES_FILE && isContractMirror(name, typesSource, rust)) continue;
    const key = `${home}\t${name}`;
    liveKeys.add(key);
    unlisted.push(key);
  }

  const allowlist = parseAllowlist();
  const newDead = unlisted.filter((key) => !allowlist.has(key));
  const stale = [...allowlist].filter((key) => !liveKeys.has(key));

  expect(newDead, "new dead export - delete it, un-export it, or add it to dead-export-allowlist.txt with a reason").toEqual([]);
  expect(stale, "allowlist entry no longer dead - remove this line from dead-export-allowlist.txt").toEqual([]);
});
