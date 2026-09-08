/**
 * SPEC-0040 R3: `src/ipc/types.ts` mirrors the `crates/orcker-ipc::Response`
 * wire contract, so a name that is dead in the GUI is not necessarily dead
 * protocol - it stands or falls with the Rust side. The exemption is computed
 * live against the Rust source rather than written down once, so it expires
 * by itself the moment a spec deletes the Rust variant it mirrors (unlike a
 * static allowlist line, which would keep reading as "fine" forever).
 *
 * Two declaration shapes need different lookups:
 * - `type FooResponse = Extract<Response, {type: "foo_bar"}>` - the wire tag
 *   is `foo_bar`; the Rust variant is its PascalCase form, `FooBar`
 *   (`#[serde(rename_all = "snake_case")]`, no per-variant renames today).
 * - anything else (`interface`, plain `type`, string-literal union) - the
 *   Rust identifier has the same name as the TS one.
 */
const EXTRACT_ALIAS = new RegExp(
  String.raw`\btype\s+(NAME)\s*=\s*Extract<\s*Response\s*,\s*\{\s*type:\s*"([a-z0-9_]+)"`,
);

function toPascalCase(snakeCase) {
  return snakeCase
    .split("_")
    .filter(Boolean)
    .map((word) => word[0].toUpperCase() + word.slice(1))
    .join("");
}

/** The Rust identifier a `src/ipc/types.ts` export would need to still exist. */
export function rustCounterpartName(name, typesSource) {
  const pattern = new RegExp(EXTRACT_ALIAS.source.replace("NAME", name));
  const match = typesSource.match(pattern);
  return match ? toPascalCase(match[2]) : name;
}

export function isContractMirror(name, typesSource, rustSource) {
  const rustName = rustCounterpartName(name, typesSource);
  return new RegExp(`\\b${rustName}\\b`).test(rustSource);
}
