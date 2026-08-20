---
id: SPEC-0001
title: Rebrand the Yerd fork into a compilable Orcker workspace with a CI gate
phase: 0
covers: [FR-001]
depends_on: []
surface:
  - ./
status: approved
attempts: 0
---

## Context

First spec of the fork. The repository is a fresh clone of Yerd at the frozen
upstream tag (recorded here in `docs/UPSTREAM.md`). Goal: a fully renamed,
compilable, tested Orcker workspace with the deterministic gate running in CI —
zero behavior change. Surface is the whole repo by necessity (rename touches
everything); this is the only spec with an unbounded surface besides SPEC-0002.

## Requirements

- R1. Record the upstream freeze in `docs/UPSTREAM.md`, containing: upstream
  repo URL, tag, commit hash, freeze date, and the policy
  ("changes concentrate in new crates; upstream merges are deliberate,
  cherry-picked events").
- R2. Rename the workspace: crates `yerd-*` -> `orcker-*`; binaries
  `yerdd` -> `orckerd`, `yerd` -> `orcker`, `yerd-helper` -> `orcker-helper`;
  GUI dir `apps/yerd-gui` -> `apps/orcker-gui`; Rust identifiers
  `yerd_*` -> `orcker_*`; package names in `Cargo.toml` files,
  `package.json`, Tauri config (product name, identifiers) and `xtask`
  packaging metadata.
- R3. Rename runtime identity strings: per-user IPC socket/pipe name, config
  directory app name (the `directories` crate qualifier), log file names,
  helper binary name expected by elevation, CLI `bin` name. `.test` TLD and
  `PROTOCOL_VERSION` stay unchanged.
- R4. Licensing and lineage: keep `Copyright (c) 2026 Forjed` intact in
  `LICENSE.md`, adding a second copyright line for Orcker modifications.
  Replace `README.md` with a minimal Orcker README containing: one-paragraph
  description, a Lineage/Credits section (fork of Yerd, MIT), and a
  non-affiliation disclaimer regarding Docker, Inc.
- R5. Remove or neutralize Yerd-only meta: `sonar-project.properties` (remove),
  `SECURITY.md`/`CONTRIBUTING.md` (rewrite minimally for Orcker), Yerd brand
  asset files in the GUI replaced by text placeholders (final logo assets are a
  later spec).
- R6. Add `.github/workflows/gate.yml`: runs `scripts/gate.sh` on ubuntu-latest
  and macos-latest with `GATE_BASE=origin/main` on pull requests and pushes to
  `main` (checkout with full history for the diff base).
- R7. Rename the `.github/instructions/*.instructions.md` files and their
  contents to match the new crate names (keep their rules verbatim otherwise).
- R8. No behavior change: no logic edits beyond renames and the files above.
- R9. Regenerate `scripts/clippy-allow-baseline.txt` after the rename: every path
  in it starts with `crates/yerd-*` or `bin/yerd*`, so the gate's step 5 fails
  until the file is rebuilt. Regenerate it, do not hand-edit it, and do not touch
  `scripts/gate.sh` itself. The counts must be identical to the pre-rename
  baseline (paths only) — a changed count means the rename altered code, which
  violates R8.
- R10. Set the project version to `0.0.0` by running
  `cargo run -p xtask -- bump 0.0.0`, which rewrites the three manifests that
  declare it (workspace `Cargo.toml`, `apps/orcker-gui/src-tauri/tauri.conf.json`,
  `apps/orcker-gui/package.json`). The inherited `2.1.0-rc.1` is Yerd's version and
  carrying it over would number Orcker's first release as a continuation of Yerd's.
  Orcker stays pre-release until the MVP gate tags `0.x` (PRD section 10, item 6).
  Do not hand-edit the manifests, do not create a tag, and do not add a changelog
  in this cycle.

## Design & contracts

Pure mechanical rename. Suggested order: filesystem renames -> `Cargo.toml`
members/names -> `cargo build` driven identifier fixes -> string identities (R3)
-> GUI (`npm run build` driven) -> meta files. No new dependencies. IPC wire
tags: internal `snake_case` type tags in `orcker-ipc` messages are part of the
wire contract — do NOT rename wire tags that embed no brand; if any wire tag or
test fixture embeds the literal `yerd`, update the wire-stability tests in the
same commit and note it (authorized here as part of the fork's contract reset;
`PROTOCOL_VERSION` stays 1).

## Test plan

- Unit/integration: the full inherited suite must stay green (that IS the test).
- New: none required beyond CI wiring.
- Manual evidence: daemon/CLI round trip.

## Acceptance checklist

- [ ] AC1 `cargo test --workspace` green after rename -> evidence: gate output
- [ ] AC2 `cargo run -p orckerd` starts and `cargo run -p orcker -- ping`
      answers -> evidence: command output in the cycle log
- [ ] AC3 `rg -i "yerd" --hidden -g '!.git' -g '!docs/UPSTREAM.md' -g '!README.md' -g '!LICENSE.md'`
      returns no matches (README/LICENSE mention Yerd only in Lineage/copyright)
      -> evidence: command output
- [ ] AC4 `docs/UPSTREAM.md` exists with repo, tag, commit, date, policy
- [ ] AC5 CI workflow file present and valid (`gate.yml` runs the gate on both
      OSes) -> evidence: file + (post-push) green run link added by the human
- [ ] AC7 `cargo run -q -p xtask -- print-version` prints `0.0.0`, and
      `rg -n '2\.1\.0-rc\.1' Cargo.toml apps/orcker-gui/package.json
      apps/orcker-gui/src-tauri/tauri.conf.json` returns no matches
      -> evidence: command output in the cycle log
- [ ] AC6 `scripts/gate.sh specs/SPEC-0001-*.md` passes

## Out of scope

New logo/visual assets; any behavior change; removal of native runtime crates
(SPEC-0002); Windows; changing `.test` TLD or protocol version.

## Agent notes

Read first: root `Cargo.toml` (workspace members), `bin/*/Cargo.toml`,
`apps/yerd-gui/package.json` + `src-tauri/tauri.conf.json`, `xtask/`
(packaging names), `.github/instructions/`. Pitfalls: `yerd-ipc` wire-stability
tests pin JSON tags (`tests/wire_stability.rs`) — check for brand literals
before renaming; the `directories` app qualifier defines the config path — an
inconsistent rename splits state across two dirs; macOS-specific code paths
must be renamed blind (CI on macOS is the check). The helper's argv contract
(`HelperInvocation::from_argv`) references the binary name — keep it consistent.

R9 in practice: leave the baseline for last, after the tree compiles under the new
names, and regenerate it with the same command the gate prints on failure:

```sh
{ rg -U -c '#!?\[allow\([^]]*clippy::(unwrap_used|expect_used|panic|todo|dbg_macro|indexing_slicing)' \
    crates bin --glob '*.rs' || true; } | sort > scripts/clippy-allow-baseline.txt
```

Then `diff` the old and new files with the paths normalised (`sed 's/yerd/orcker/g'`)
and confirm they match: same file set, same counts. The pre-rename baseline is in
the parent commit (`git show HEAD:scripts/clippy-allow-baseline.txt`), 221 lines.
Rationale for the whole check lives in `specs/DECISIONS.md`.

R10 in practice: `xtask` already owns this — `xtask/src/version.rs` edits only the
single version line in each of the three manifests, so there is nothing to hand-roll
and nothing else to grep. Run it after the GUI directory rename, or it will write to
the old path. `release.yml` calls `xtask version-check <tag>` as its release gate;
that workflow is `workflow_dispatch`-only in this fork, so nothing fires on `0.0.0`.
