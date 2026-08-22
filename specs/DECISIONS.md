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

## 2026-08-22 · SPEC-0001 (cycle)

- Decision: amend AC3's exclusion set mid-cycle, adding `CLAUDE.md`,
  `docs/PRD.md`, `docs/SDD.md`, `specs/**` and `**/package-lock.json` to the
  three files it already exempted. Approved by the human during the cycle.
- Why: as written, AC3 was unsatisfiable. Meeting it required editing
  `docs/PRD.md`, which `CLAUDE.md` forbids outright, and rewriting the fork's own
  lineage and process documents, which name Yerd deliberately, to the point of
  retitling SPEC-0001 itself "Rebrand the Orcker fork". The two `package-lock.json`
  hits are the substring `YerD` inside the npm integrity hash
  `sha512-NxnomyxYerDh5n4i...`, editable only by corrupting the lockfile. The
  amendment removes an impossible clause; the product outcome is unchanged, and
  the supervisor verified nothing in `crates/`, `bin/`, `apps/`, `xtask/`,
  `scripts/`, `packaging/`, `.github/` or the docs site still carries the brand.
- Impact: AC3 in `specs/SPEC-0001-fork-bootstrap.md` now carries the wider glob set
  and its own rationale inline. The canonical check is `git grep`, not `rg`: two
  `rg` runs of the same query deadlocked in `unix_stream_data_wait` at 0s CPU in
  this environment.

- Decision: delete `.github/workflows/sonarqube.yml` alongside the
  `sonar-project.properties` that R5 names.
- Why: the workflow consumes only that file. Keeping it would leave a permanently
  red CI job pointed at a deleted config.
- Impact: one workflow removed beyond R5's literal list.

- Decision: rewrite upstream URLs mechanically and leave them pointing at Orcker
  paths that do not exist yet.
- Why: AC3 forces the brand out of `release.yml`, `build-cdn.yml`, `cdn-sync.yml`,
  `xtask/src/cdn.rs`, `scripts/release.sh` and `packaging/arch/*`; choosing the real
  hosts and repositories is a product decision outside a rename spec. Every one of
  those workflows is `workflow_dispatch`-only, so nothing fires meanwhile.
- Impact: queued as `specs/SPEC-0031-repoint-release-and-cdn-automation.md`
  (`draft`). The first release attempt fails until that spec lands.

- Decision: leave the binary GUI icons as the inherited "Y" artwork.
- Why: they carry no `yerd` string, so AC3 passes, and new visual assets are
  explicitly out of scope for SPEC-0001. Replacing `.icns` / `.ico` / the
  `Square*Logo` and Android mipmap sets with "text placeholders" is not possible
  in those formats.
- Impact: the four `.svg` source marks became text placeholders; the rendered
  binaries did not. Queued as
  `specs/SPEC-0029-replace-binary-brand-icons.md` (`draft`).

- Decision: repair two tests rather than the code they cover.
- Why: `dns_probe::tests::query_encodes_probe_name_and_a_question` hard-coded DNS
  wire offsets that shifted by 2 bytes when `PROBE_LABEL` grew from
  `yerd-resolver-probe` to `orcker-resolver-probe`; `compose_query` is
  length-driven and was already correct. `self_update::tests::current_version_parses`
  asserted `current_version() != 0.0.0`, but `0.0.0` is that function's
  "unparseable semver" fallback sentinel and, after R10, also the real pinned
  version, so the guard could no longer express its own intent.
- Impact: no production code changed. The supervisor confirmed neither test is
  weakened: the DNS test still pins the exact byte layout, and
  `assert!(declared.is_ok())` is exactly what the `!=` sentinel was a proxy for.
  The `directories` qualifier also moved `io/yerd/Yerd` -> `io/orcker/Orcker`, so
  pre-existing local Yerd state is orphaned rather than migrated. Intended for a
  fork; no migration path exists or was tested.

- Decision: pin the gate's clippy-allow `sort` to the C collation, in a spec of
  its own, instead of merging PR #1 with a red gate or regenerating the baseline
  to match one machine.
- Why: `scripts/gate.sh` built the list with a bare `sort`, whose order follows
  `LC_COLLATE`. Glibc's `pt_BR.UTF-8` ignores `-` at primary strength, so
  `bin/orcker-helper/` sorts after `bin/orckerd/`; the C collation compares raw
  bytes, where `-` (0x2D) precedes every letter, so it sorts before. One file
  set, two legal orders, and step 5 fails on whichever machine did not generate
  the baseline. A checked-in artifact cannot depend on the author's locale.
- Impact: `scripts/clippy-allow-baseline.txt` was regenerated; 221 lines before
  and after, identical set, only the `bin/orcker*` block relocated. The defect is
  inherited from the bootstrap gate, not from the rename: SPEC-0001 merely added
  `.github/workflows/gate.yml`, which ran the gate off this machine for the first
  time and exposed it.

- Decision: put `scripts/gate.sh` inside SPEC-0032's surface, which DT7 normally
  keeps outside every surface.
- Why: DT7 forbids *weakening* the gate. Pinning the collation makes step 5
  locale-independent and therefore reproducible. The supervisor verified the
  claim against the diff: `CLIPPY_ALLOW_RE`, the `rg` invocation and its scope,
  `diff -u` and `exit 1` are byte-identical, and a bare `pt_BR` sort still fails
  the new baseline, so the check was not silenced.
- Impact: the precedent is narrow. Touching `scripts/gate.sh` needs a spec that
  says so and a supervisor who confirms the change strengthens the check. The
  declared surface `scripts/` was broader than the one file the diff needed; a
  future spec of this shape should name `scripts/gate.sh`.

- Decision: land SPEC-0032 on `feat/SPEC-0001-fork-bootstrap` rather than on a
  branch of its own.
- Why: AC6 is "both CI gate legs green". `main` carries no
  `.github/workflows/gate.yml` — SPEC-0001 adds it — so a `feat/SPEC-0032-*`
  branch cut from `main` would run no gate job, prove nothing, and leave PR #1
  red anyway.
- Impact: the branch-per-spec rule bends for a cycle whose acceptance depends on
  CI that only exists on another branch. Per-spec commit atomicity is preserved:
  the pull request carries two commits, one per spec.
