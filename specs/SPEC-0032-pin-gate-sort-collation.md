---
id: SPEC-0032
title: Pin the gate's clippy-allow sort to the C collation
phase: 0
covers: [FR-001]
depends_on: [SPEC-0001]
surface:
  - scripts/
status: accepted
attempts: 0
---

## Context

Gate step 5/6 freezes the per-file count of clippy allows by diffing a freshly
built list against `scripts/clippy-allow-baseline.txt`. The list is built with a
bare `sort`, so its order follows the machine's `LC_COLLATE`. Glibc's
`pt_BR.UTF-8` collation ignores `-` at primary strength, so `bin/orcker-helper/`
compares as `binorckerhelper` and lands *after* `bin/orckerd/`; the C collation
compares raw bytes, where `-` (0x2D) precedes every letter, so the same paths
land before. Identical file set, two legal orders, and `diff -u` fails on the
one the baseline was not generated on.

`.github/workflows/gate.yml` (added by SPEC-0001) ran the gate on a foreign
machine for the first time, and both CI legs failed step 5 against a baseline
generated under `pt_BR.UTF-8`. This is a latent bootstrap defect in
`scripts/gate.sh`, not fallout from the rename: the same failure would have
appeared on any contributor's machine with a non-C locale.

The surface is `scripts/` because the fix is in the gate script itself.
`scripts/gate.sh` normally stays outside every surface (DT7), and DT7 forbids
*weakening* the gate. Pinning the collation strengthens it: the check becomes
locale-independent and therefore reproducible, and the set of files it watches
is unchanged.

## Requirements

- R1. Build the clippy-allow list under a fixed collation in `scripts/gate.sh`,
  so the produced order is identical on every machine.
- R2. Carry the same fixed collation into the refresh command the step prints on
  failure, so a developer following that hint regenerates a baseline the gate
  will accept.
- R3. Regenerate `scripts/clippy-allow-baseline.txt` under that collation. The
  line set must not change: only the order may.

## Acceptance checklist

- [x] AC1 `scripts/gate.sh` contains no unqualified `sort` on the clippy-allow
      pipeline: `grep -n 'sort' scripts/gate.sh` shows every occurrence pinned
      -> evidence: command output.
- [x] AC2 the baseline is byte-identical to a fresh C-collated build:
      `{ rg -U -c "$CLIPPY_ALLOW_RE" crates bin --glob '*.rs' || true; } |
      LC_ALL=C sort | diff -u scripts/clippy-allow-baseline.txt -` is empty
      -> evidence: command output.
- [x] AC3 the baseline's line *set* is unchanged from `HEAD`:
      `diff <(sort scripts/clippy-allow-baseline.txt)
      <(git show HEAD:scripts/clippy-allow-baseline.txt | sort)` is empty
      -> evidence: command output.
- [x] AC4 the gate passes under a locale that is not C:
      `LC_ALL=pt_BR.UTF-8 scripts/gate.sh specs/SPEC-0032-pin-gate-sort-collation.md`
      exits 0 -> evidence: gate output.
- [x] AC5 the gate passes under C:
      `LC_ALL=C scripts/gate.sh specs/SPEC-0032-pin-gate-sort-collation.md`
      exits 0 -> evidence: gate output.
- [ ] AC6 both `gate` legs of `.github/workflows/gate.yml` are green on the
      pull request -> evidence: `gh pr checks`.
