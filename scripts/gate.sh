#!/usr/bin/env bash
# Deterministic quality gate — same script locally and in CI.
# Usage: scripts/gate.sh [SPEC_FILE]
# Diff base: HEAD locally (uncommitted work); CI sets GATE_BASE=origin/main.
set -euo pipefail
GATE_BASE="${GATE_BASE:-HEAD}"

echo "[gate 1/6] rustfmt"
cargo fmt --all --check

echo "[gate 2/6] clippy (deny warnings)"
cargo clippy --workspace --all-targets -- -D warnings

echo "[gate 3/6] tests"
cargo test --workspace

echo "[gate 4/6] gui (only when touched)"
if ! git diff --quiet "$GATE_BASE" -- apps/orcker-gui/ 2>/dev/null; then
  npm --prefix apps/orcker-gui run test
  npm --prefix apps/orcker-gui run build
fi

echo "[gate 5/6] no new clippy escape hatches"
# [workspace.lints.clippy] already denies unwrap_used / expect_used / panic /
# todo / dbg_macro / indexing_slicing, and step 2 enforces it across all targets.
# Grepping for `.unwrap()` here would only duplicate that check textually, and a
# text match cannot see the `#[allow]` that test modules legitimately carry.
# So this step watches the escape hatch instead: the per-file count of clippy
# allows is frozen in the baseline, and any change has to be deliberate.
CLIPPY_ALLOW_RE='#!?\[allow\([^]]*clippy::(unwrap_used|expect_used|panic|todo|dbg_macro|indexing_slicing)'
CLIPPY_ALLOW_CURRENT="$(mktemp)"
trap 'rm -f "$CLIPPY_ALLOW_CURRENT"' EXIT
# LC_ALL=C, not a bare sort: the baseline is a checked-in artifact, and glibc's
# locale collations ignore `-` at primary strength, so `bin/orcker-helper/` sorts
# after `bin/orckerd/` under pt_BR.UTF-8 and before it under C. Same file set,
# two legal orders, and the diff below fails on whichever machine did not
# generate the baseline. Byte order is the only order every machine agrees on.
{ rg -U -c "$CLIPPY_ALLOW_RE" crates bin --glob '*.rs' || true; } |
  LC_ALL=C sort > "$CLIPPY_ALLOW_CURRENT"
if ! diff -u scripts/clippy-allow-baseline.txt "$CLIPPY_ALLOW_CURRENT"; then
  echo "[gate] clippy allow list changed"
  echo "[gate]   '-' line: an allow disappeared - good, refresh the baseline"
  echo "[gate]   '+' line: a new escape hatch - justify it in the spec first"
  echo "[gate] refresh with:"
  echo "[gate]   { rg -U -c '<see CLIPPY_ALLOW_RE in this script>' crates bin --glob '*.rs' || true; } |"
  echo "[gate]     LC_ALL=C sort > scripts/clippy-allow-baseline.txt"
  exit 1
fi

echo "[gate 6/6] surface check"
if [[ $# -ge 1 ]]; then
  scripts/surface-check.sh "$1"
else
  echo "[gate] no spec file given - surface check skipped"
fi

echo "[gate] OK"
