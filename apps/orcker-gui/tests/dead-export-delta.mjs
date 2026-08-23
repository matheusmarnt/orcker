/**
 * R1's completeness check: which exports did THIS diff orphan?
 *
 * SPEC-0036's R1 says "remove everything only they reach", and the spec's other
 * completeness check - `rg -f specs/logs/SPEC-0002-removed-symbols.txt` - cannot
 * answer that: it holds the symbols SPEC-0002 removed, so by construction it
 * finds SPEC-0002's orphans and never a symbol SPEC-0036 orphaned. This script
 * is the missing half.
 *
 * It reports `dead(working tree) \ dead(HEAD)` - exports with no consumer now
 * that had one at HEAD. Scoping to the delta is the point: the tree carries ~96
 * exports that were already dead before this branch, and they are not this
 * spec's debt to pay (see SPEC-0037). An absolute "no dead exports" gate was
 * tried and rejected - it needs a ~84-entry allowlist, it flags same-file
 * helpers like `jobStatus` that `pollJobToEnd` still calls, and it demands
 * deleting the `ipc/types.ts` wire-contract mirror, including the `JobState`
 * that SPEC-0036 R3 explicitly orders to keep.
 *
 * Deliberately NOT a vitest file (`vitest.config.ts` collects only
 * `*.{test,spec}.ts`): the answer depends on HEAD, so it is cycle evidence to
 * record in the log, not a standing gate. A standing ratchet over the ~96
 * inherited exports needs the contract-mirror exemption designed properly and
 * belongs to its own spec.
 *
 * Usage: node tests/dead-export-delta.mjs
 * Exit 0 always - this reports, it does not judge. Judgement is the spec's.
 */
import { execFileSync } from "node:child_process";
import { readFileSync, readdirSync, statSync } from "node:fs";
import { join, relative } from "node:path";

const ROOT = execFileSync("git", ["rev-parse", "--show-toplevel"], { encoding: "utf8" }).trim();
const SUBTREE = "apps/orcker-gui/src";
const SOURCE = /\.(ts|vue)$/;

// A top-level `export` of a named binding. Re-exports and `export default` are
// out: neither introduces a name this scan can resolve consumers for.
const EXPORT =
  /^export\s+(?:async\s+)?(?:abstract\s+)?(?:const|let|var|function|class|type|interface|enum)\s+([A-Za-z0-9_$]+)/gm;

function walk(dir) {
  return readdirSync(dir).flatMap((entry) => {
    const path = join(dir, entry);
    if (statSync(path).isDirectory()) return walk(path);
    return SOURCE.test(entry) ? [path] : [];
  });
}

function workingTree() {
  const files = new Map();
  for (const path of walk(join(ROOT, SUBTREE))) {
    files.set(relative(ROOT, path), readFileSync(path, "utf8"));
  }
  return files;
}

function atHead() {
  const listed = execFileSync("git", ["ls-tree", "-r", "--name-only", "HEAD", "--", SUBTREE], {
    cwd: ROOT,
    encoding: "utf8",
  });
  const files = new Map();
  for (const path of listed.split("\n").filter((p) => SOURCE.test(p))) {
    files.set(path, execFileSync("git", ["show", `HEAD:${path}`], { cwd: ROOT, encoding: "utf8" }));
  }
  return files;
}

/**
 * Names exported from exactly one file and referenced from no other file.
 *
 * "Referenced from another file" is the test, not "imported": a Vue SFC uses a
 * name in its template, and a word-boundary scan sees that where an import
 * graph would need the template compiled. The cost is false negatives on
 * same-file-only helpers, which is why this reports rather than fails.
 * A name exported from two files is skipped - the scan cannot tell which one a
 * consumer meant.
 */
function deadExports(files) {
  const homes = new Map();
  for (const [path, text] of files) {
    for (const [, name] of text.matchAll(EXPORT)) {
      homes.set(name, [...(homes.get(name) ?? []), path]);
    }
  }
  const dead = new Map();
  for (const [name, paths] of homes) {
    if (paths.length !== 1) continue;
    const home = paths[0];
    const pattern = new RegExp(`\\b${name.replace(/\$/g, "\\$")}\\b`);
    let used = false;
    for (const [path, text] of files) {
      if (path !== home && pattern.test(text)) {
        used = true;
        break;
      }
    }
    if (!used) dead.set(name, home);
  }
  return dead;
}

const now = deadExports(workingTree());
const head = deadExports(atHead());
const orphaned = [...now].filter(([name]) => !head.has(name));
const revived = [...head].filter(([name]) => !now.has(name));

console.log(`dead at HEAD: ${head.size}   dead now: ${now.size}`);
console.log(`this diff orphaned ${orphaned.length}, resolved ${revived.length}\n`);
if (orphaned.length === 0) {
  console.log("no export lost its last consumer to this diff");
} else {
  console.log("orphaned by this diff - delete, or justify each keep in the cycle log:");
  for (const [name, home] of orphaned.sort()) console.log(`  ${name}\t${home}`);
}
