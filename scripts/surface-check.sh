#!/usr/bin/env bash
# Every changed path must start with a prefix declared in the spec's `surface:`
# front-matter list. specs/ and docs/ are always allowed.
# Usage: scripts/surface-check.sh SPEC_FILE   (env: GATE_BASE, default HEAD)
set -euo pipefail
SPEC="$1"
GATE_BASE="${GATE_BASE:-HEAD}"

# specs/ and docs/ are always allowed, and so is the generated clippy allow
# baseline: any spec that adds, deletes, moves or renames a .rs file has to
# regenerate it for step 5 to pass. The file only, never scripts/ as a whole —
# scripts/gate.sh stays outside every surface (DT7).
ALLOWED=("specs/" "docs/" "scripts/clippy-allow-baseline.txt")
ALLOW_ALL=0
while IFS= read -r p; do
  p="${p%\"}"; p="${p#\"}"
  [[ "$p" == "./" || "$p" == "." ]] && ALLOW_ALL=1 && continue
  [[ -n "$p" ]] && ALLOWED+=("$p")
done < <(awk '/^surface:/{f=1;next} f&&/^ *- /{sub(/^ *- */,"");print;next} f&&/^[a-zA-Z_]+:/{exit}' "$SPEC")

if [[ $ALLOW_ALL -eq 1 ]]; then
  echo "[surface] spec declares unbounded surface (./) - all paths allowed"
  exit 0
fi

echo "[surface] allowed prefixes: ${ALLOWED[*]}"
FAIL=0
while IFS= read -r file; do
  ok=0
  for p in "${ALLOWED[@]}"; do
    [[ "$file" == "$p"* ]] && ok=1 && break
  done
  if [[ $ok -eq 0 ]]; then
    echo "[surface] VIOLATION: $file"
    FAIL=1
  fi
done < <(git diff --name-only "$GATE_BASE")
exit $FAIL
