---
id: SPEC-0030
title: Fix the stale daemon run command in the building guide
phase: 0
covers: [FR-001]
depends_on: [SPEC-0001]
surface:
  - docs/
status: accepted
attempts: 1
---

## Context

`docs/developer/building.md` tells the reader to start the daemon with
`cargo run -p orckerd -- -v`. `orckerd` accepts no top-level `-v`: it exits
with `error: unexpected argument '-v' found`, `Usage: orckerd [COMMAND]`.
Inherited from upstream and unrelated to the rename, found while gathering
AC2 evidence for SPEC-0001. Re-confirmed 2026-09-04 while manually verifying
SPEC-0046's AC3: `bin/orckerd/src/args.rs` has since gained an explicit
`serve` subcommand (`Cli.command: Option<Command>`), and `verbose` lives on
`ServeArgs`, not the top-level `Cli` — so the fix is `orckerd -- serve -v`,
not a bare `orckerd -- serve`. Dropping `-v` (the earlier framing here) would
silence the verbose logging the command was written to enable.

## Requirements

- R1. Replace every `orckerd -- -v` occurrence with `orckerd -- serve -v`
  and verify each command block in the guide actually runs.

## Design & contracts

No code, no crates, no dependencies. One file, three spots:

| File | Operation |
|------|-----------|
| `docs/developer/building.md` | 3 command lines carrying a `ServeArgs` flag (`-v`, `--config`) without the explicit `serve` token; each gets `serve` inserted. Two adjacent sentences are corrected: the one motivating the first command (see below) and the one at "Pointing at a different config file" that said `orckerd` itself takes `--config` (`-c`) — it is `serve` that takes it (`bin/orckerd/src/args.rs:37-38`) |

Round-1 self-review wrongly asserted `serve` is *not* the default subcommand
and edited the guide to say so. `bin/orckerd/src/main.rs:20`
(`cli.command.unwrap_or_else(|| Command::Serve(ServeArgs::default()))`) and
`args.rs:18`'s own doc comment both say it is, for the zero-argument case
(`orckerd serve &` and bare `orckerd` are equivalent). The real mechanism:
`Cli`'s clap parser has no `-v`/`--config` fields of its own, so it cannot
route a bare flag into the default `Serve` variant — those flags exist only
on `ServeArgs`, reachable only once `serve` is named. The guide is corrected
to say exactly that, not that `serve` is merely "the subcommand that starts
it".

`docs/spike/PHASE0-SPIKE.md` mentions `orckerd -v` as the historical record of
this very defect (Findings F1) and is not touched, for the same reason
SPEC-0042 R5 leaves past records alone: it states what was true when it was
written.

## Test plan

Docs-only spec: no unit or integration surface exists. Every AC is verified by
a command whose output is quoted in `specs/logs/SPEC-0030.md`, run before the
fix (RED) and after (GREEN).

- E2E / manual (unavoidable, and stated as such): a pattern-based `grep` over
  the guide that catches *any* `orckerd -- <flag>` invocation missing `serve`
  (not just the two originally-named ones — round 1's literal-string ACs
  passed while a third stale block, `--config`, survived undetected), plus
  actually running each corrected command against a scratch XDG instance to
  prove it parses and starts.

## Acceptance checklist

- [ ] AC1 (R1) no `orckerd` invocation in the guide passes a flag without
      naming `serve` → evidence: `grep -cP 'orckerd -- (?!serve)'
      docs/developer/building.md` prints `0` (RED, at `HEAD`: `3`)
- [ ] AC2 (R1) every such invocation now names `serve` explicitly → evidence:
      `grep -c 'orckerd -- serve'` on the same file prints `3`
- [ ] AC3 (R1) each of the three fixed commands actually runs → evidence: each
      run against a scratch XDG tree starts the daemon (`orckerd::startup`
      log lines) with no `unexpected argument` error — RED showed `error:
      unexpected argument '-v' found` / `'--config' found`, `Usage: orckerd
      [COMMAND]`, exit 2, for all three prior to the fix
- [ ] AC4 the historical spike record is untouched → evidence: `git diff
      --exit-code HEAD -- docs/spike/PHASE0-SPIKE.md` exits 0
- [ ] AC5 `scripts/gate.sh specs/SPEC-0030-fix-stale-daemon-run-command-in-docs.md`
      passes

## Out of scope

- `docs/spike/PHASE0-SPIKE.md` — a historical record of this defect, not a
  live instruction; correcting it would falsify what was true when it was
  written.
- Running the guide's command blocks that carry no `orckerd` invocation and
  are not runnable in this environment regardless (macOS `launchctl`,
  `sudo apt-get`, packaging, Apple secret rotation, …). AC1's pattern already
  covers every `orckerd` block in the file, so this carve-out is narrower
  than round 1's — it excludes only blocks the AC does not reach at all, not
  ones judged "unrelated to the `-v` defect".
- Any change to `bin/orckerd/src/args.rs` or other CLI behavior — the bug is
  in the docs, not the binary.

## Agent notes

Read first: this file, `docs/developer/building.md`, `bin/orckerd/src/args.rs`
and `bin/orckerd/src/main.rs:20` (the `unwrap_or_else` that makes `serve` the
true default subcommand — round 1 misread this). No crate instruction file
applies — the diff touches no Rust. `specs/SPEC-0042-*.md` is the precedent
for a docs-only spec's RED/GREEN evidence and Acceptance checklist shape.

Pitfall (round 1's own mistake): a literal-string grep AC (`orckerd -- -v`)
only checks the exact pattern named in the Context, and this guide had a third
occurrence (`orckerd -- --config ... -v`) the Context never named. Prefer a
pattern general enough to catch the whole defect class, per R1's "every"
and "each command block" wording.
