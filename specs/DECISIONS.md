# Cycle decisions log

Deviations, clarifications and trade-offs recorded by implementation cycles
(SDD section 6, S8). Newest first. Entry format:

```
## YYYY-MM-DD · SPEC-XXXX
- Decision: <what was decided>
- Why: <reason in 1-2 lines>
- Impact: <files/specs/requirements affected; follow-up spec id if any>
```

## 2026-08-20 · bootstrap (pre-cycle, no spec)

- Decision: the project version becomes `0.0.0` and stays there until the MVP gate.
  Added to SPEC-0001 as R10 (with AC7), executed via `cargo xtask bump 0.0.0`.
- Why: the workspace inherited Yerd's `2.1.0-rc.1`, and no spec touched it. Shipping
  from there would number Orcker's first release as a continuation of Yerd's, while
  PRD section 10 item 6 releases the MVP as `0.x` — a lower number, which every
  semver tool reads as a downgrade. `0.0.0` says "nothing released yet" honestly.
- Impact: Phases 0 and 1 carry no tags at all; the first tag is the MVP `0.x` with a
  changelog (PRD section 10.6, FR-130). `release.yml` gates on
  `xtask version-check <tag>` and is `workflow_dispatch`-only here, so nothing fires.
  Folded into SPEC-0001 rather than a separate spec because that cycle already
  rewrites every `Cargo.toml`; a second pass over the same files would be waste.

## 2026-08-20 · bootstrap (pre-cycle, no spec)

- Decision: replace `scripts/gate.sh` step 5/6. It grepped `crates` and `bin` for
  `.unwrap()` / `.expect(` / `panic!(` / `todo!(` / `dbg!(`, assuming all test code
  lives in `tests/` or `*_test.rs`. The inherited codebase puts tests in inline
  `#[cfg(test)] mod tests` blocks with an `#[allow(clippy::…)]` on top — 2587 hits
  across 192 files — so the step could never pass. Now it ratchets the per-file
  count of clippy `#[allow]`s against `scripts/clippy-allow-baseline.txt`.
- Why: `[workspace.lints.clippy]` already denies those exact six lints and step 2
  (`clippy -D warnings`) enforces them semantically — it exits 0 on this tree. A
  textual grep can only duplicate that check while being blind to the `#[allow]`
  attributes the convention requires, so it produced 2587 false positives and zero
  true ones. The escape hatch itself (an `#[allow]` added to dodge clippy) is the
  thing a grep can usefully watch, and freezing counts needs no Rust parsing.
- Impact: `scripts/gate.sh` step 5; new `scripts/clippy-allow-baseline.txt` (221
  files). This is a gate change made during bootstrap, before any spec was
  `in_progress` — inside a cycle it would be a DT7 automatic REWORK. The baseline
  is regenerated wholesale by SPEC-0001, which renames every path in it; SPEC-0002
  shrinks it further by deleting `yerd-php` / `yerd-services` / `yerd-supervise`.
- Open finding: `crates/yerd-php/src/manager.rs:611` carries `#[allow(clippy::panic)]`
  over a genuine production `panic!` ("driver invariant violated"). Left as-is —
  SPEC-0002 removes that crate. No spec needed unless it survives.

## 2026-08-20 · bootstrap (pre-cycle, no spec)

- Decision: freeze the fork at `forjedio/yerd@v2.1.0-rc.1`
  (`896c44938c555d75144ada6da1a72c7d95918a2b`), not at `main` `b7e7c1c`.
- Why: the first cycle renames 511 files, so a named, release-pipeline-tested base
  keeps a red gate an answer instead of an investigation. Of the four commits above
  the tag, two are deleted by the roadmap itself (native-PHP coverage shim, Yerd-
  branded docs site), one is cosmetic (macOS zoom), and one has durable value.
  Full reasoning in `specs/BOOTSTRAP.md` section D2.
- Impact: `docs/UPSTREAM.md` (SPEC-0001 R1) records the tag; the durable commit
  `7d69d6a` becomes SPEC-0028 under "Upstream cherry-picks" in `specs/ROADMAP.md`;
  local `main` sits four commits behind `origin/main`, so the first publish is a
  one-time `git push --force` by the human.

## 2026-08-20 · bootstrap (pre-cycle, no spec)

- Decision: version the `.claude/` harness. `.gitignore` line 48, inherited from
  Yerd, ignored `.claude` wholesale; replaced with `.claude/settings.local.json`.
- Why: SDD section 3 lists `.claude/` as a repository artifact. The settings,
  subagents and slash commands ARE the process — ignoring them means the harness
  lives on one machine and the SDD loop is unreproducible for anyone else.
- Impact: `.gitignore`; the baseline commit now carries the six `.claude/` files.
