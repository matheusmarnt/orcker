/**
 * Shared scanner behind `dead-export-delta.mjs` (cycle evidence) and
 * `deadExportRatchet.test.ts` (the standing gate). One implementation so a fix
 * to a blind spot in the algorithm applies to both callers.
 *
 * A top-level `export` of a named binding. Re-exports and `export default` are
 * out: neither introduces a name this scan can resolve consumers for.
 */
export const EXPORT =
  /^export\s+(?:async\s+)?(?:abstract\s+)?(?:const|let|var|function|class|type|interface|enum)\s+([A-Za-z0-9_$]+)/gm;

const TEST_FILE = /\.(test|spec)\.ts$/;

/**
 * Blank out line comments, block comments, and string/template literals,
 * preserving line breaks so `EXPORT`'s `^`/`$` anchors stay accurate. A name
 * that only ever appears in prose or a literal is not a real reference - see
 * `formatLoadAvg` and `invalidate`, alive today only in a doc comment naming
 * them.
 *
 * Heuristic, not a parser: nesting of `${}` inside a template literal is not
 * tracked, so a name mentioned only inside an interpolation can still slip
 * through as "used" - the same class of imprecision the word-boundary scan
 * already accepts everywhere else in this file.
 */
function stripNonCode(text) {
  return text.replace(
    /\/\/.*|\/\*[\s\S]*?\*\/|`(?:\\.|[^`\\])*`|"(?:\\.|[^"\\])*"|'(?:\\.|[^'\\])*'/g,
    (match) => match.replace(/[^\n]/g, " "),
  );
}

/**
 * Names exported from exactly one file and referenced from no other file.
 *
 * "Referenced from another file" is the test, not "imported": a Vue SFC uses a
 * name in its template, and a word-boundary scan sees that where an import
 * graph would need the template compiled. A name exported from two files is
 * skipped - the scan cannot tell which one a consumer meant.
 *
 * Test files (`*.test.ts`, `*.spec.ts`) are dropped before either pass: an
 * export kept alive only by its own test is not a production consumer
 * (SPEC-0036 hit this with `TEXT_SETTINGS`). Usage inside the home file itself
 * counts too, past the declaration's own occurrence - a private helper another
 * function in the same file calls (SPEC-0036's `jobStatus`) is not dead, it is
 * over-exported.
 */
export function deadExports(files) {
  const prod = [...files]
    .filter(([path]) => !TEST_FILE.test(path))
    .map(([path, text]) => [path, stripNonCode(text)]);

  const homes = new Map();
  for (const [path, text] of prod) {
    for (const [, name] of text.matchAll(EXPORT)) {
      homes.set(name, [...(homes.get(name) ?? []), path]);
    }
  }

  const dead = new Map();
  for (const [name, paths] of homes) {
    if (paths.length !== 1) continue;
    const home = paths[0];
    const pattern = new RegExp(`\\b${name.replace(/\$/g, "\\$")}\\b`, "g");
    let hits = 0;
    for (const [path, text] of prod) {
      const count = (text.match(pattern) ?? []).length;
      hits += path === home ? Math.max(0, count - 1) : count;
    }
    if (hits === 0) dead.set(name, home);
  }
  return dead;
}
