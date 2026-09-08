import { describe, expect, it } from "vitest";
import { deadExports } from "./deadExports.mjs";

/**
 * Unit coverage for the blind spots SPEC-0036's supervisor found in the
 * original delta scan, plus the same-file-usage case SPEC-0036 hit with
 * `jobStatus`. Each case is a minimal in-memory file map - no git, no disk.
 */
describe("deadExports", () => {
  it("does not count a mention inside a comment as usage", () => {
    const files = new Map([
      ["a.ts", "export function helper() {}\n"],
      ["b.ts", "// call helper() before shipping\n"],
    ]);
    expect(deadExports(files)).toEqual(new Map([["helper", "a.ts"]]));
  });

  it("does not count a mention inside a string literal as usage", () => {
    const files = new Map([
      ["a.ts", "export function helper() {}\n"],
      ["b.ts", 'const note = "helper is unused";\n'],
    ]);
    expect(deadExports(files)).toEqual(new Map([["helper", "a.ts"]]));
  });

  it("does not count a mention only inside the export's own test file as usage", () => {
    const files = new Map([
      ["a.ts", "export function helper() {}\n"],
      ["a.test.ts", 'import { helper } from "./a";\nhelper();\n'],
    ]);
    expect(deadExports(files)).toEqual(new Map([["helper", "a.ts"]]));
  });

  it("does not report an export real code in its own home file calls", () => {
    const files = new Map([
      ["a.ts", "export function helper() { return 1; }\nfunction outer() { return helper(); }\n"],
    ]);
    expect(deadExports(files)).toEqual(new Map());
  });
});
