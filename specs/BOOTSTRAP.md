# Bootstrap readiness — Orcker fork

Pre-cycle checklist. The harness install is the only work outside the SDD loop
(SDD section 12). Nothing here is product code; no spec covers it.

Audited 2026-08-20 · executed the same day · freeze point `896c449` (`v2.1.0-rc.1`)

## Status

| Item | State |
|---|---|
| B1 `.claude/` harness | **done** — 6 files written from SDD 9.2-9.4 |
| B2 `rg` (ripgrep) | **done** — ripgrep 13.0.0 at `/usr/bin/rg` |
| B3 spec sign-off | **pending human** — all six specs still `draft` |
| D1 PRD/SDD location | **done** — `docs/PRD.md`, `docs/SDD.md`, `.gitignore` clean |
| D2 upstream freeze | **done** — base is `origin/main` `b7e7c1c`; `upstream` remote added |
| D3 inherited CI | **done** — 3 workflows neutralized |
| D4 GitHub Issues | decided: stay disabled (spec-file-driven queue) |
| D5 `.claude/` versioned | **done** — Yerd's `.gitignore` ignored the whole harness |
| B4 gate step 5 unrunnable | **done** — replaced with a clippy-allow ratchet |
| Baseline commit | pending — after the gate proves green |

## B1 — `.claude/` harness (done)

`README-INSTALL.md` step 1 copies `CLAUDE.md .claude scripts specs`; only three of
the four had landed. Written from SDD sections 9.2-9.4:

```
.claude/settings.json           permissions (deny git push / gh release / cargo publish
                                / Read .env) + rustfmt-on-edit PostToolUse hook
.claude/agents/supervisor.md    acceptance gatekeeper: DT1-DT8, JG1-JG8, decision rule,
                                mandatory 8.4 verdict block; model: opus
.claude/agents/spec-writer.md   drafts specs from PRD requirements; never sets `approved`
.claude/commands/spec-next.md   /spec-next   — loop S0-S3
.claude/commands/spec-verify.md /spec-verify — loop S5-S8
.claude/commands/spec-new.md    /spec-new FR-xxx
```

## B2 — ripgrep (done)

`scripts/gate.sh` step 5/6 shells out to `rg` under `set -euo pipefail`, so the gate
died before the surface check. Installed: ripgrep 13.0.0. Rest of the prerequisites
verified: cargo 1.96.0 (pinned), node v24.16.0, npm 11.13.0, jq 1.6, Docker 29.7.2,
compose v5.5.0, gh 2.97.0 (authenticated as `matheusmarnt`).

## B3 — spec sign-off (pending, human only)

All six Phase-0 specs are `status: draft`. S0 picks the first `approved` spec whose
`depends_on` are `accepted`, so `/spec-next` still selects nothing. `draft ->
approved` is the only exclusively human transition (SDD section 5).

```bash
sed -i 's/^status: draft/status: approved/' specs/SPEC-0001-*.md
```

Approve one at a time. Batch-approving all six removes the review the sign-off
exists for.

## D1 — PRD/SDD location (done)

Files live at `docs/PRD.md` and `docs/SDD.md` (untracked, entering the baseline
commit). `.gitignore` no longer excludes them, and `CLAUDE.md:9` already points at
those paths. No further edit needed.

## D2 — upstream freeze point: the tag (settled)

Base: **`forjedio/yerd@v2.1.0-rc.1` (`896c449`)**. `upstream` remote configured.
The head `b7e7c1c` was evaluated and rejected; the reasoning is kept below because
`docs/UPSTREAM.md` (SPEC-0001 R1) has to justify the freeze, not just state it.

**Cost accepted:** `origin/main` still publishes `b7e7c1c`, so local `main` is four
commits behind it. The first publish is therefore `git push --force` — a human act,
on a fork nothing else depends on yet. Do it before anyone clones the repo.

**Consequence handled:** the one commit above the tag with durable value
(`7d69d6a`, NetworkManager reload flags) is now `specs/SPEC-0028-cherry-pick-networkmanager-reload-fix.md`
(`draft`, depends on SPEC-0001), listed under "Upstream cherry-picks" in the roadmap.
That is the `UPSTREAM.md` policy working as designed: upstream code enters through a
spec, never through a bulk sync.

### Why NOT to fork from the newest point

1. **The freeze loses its name.** SPEC-0001 R1 wants repo + tag + commit in
   `docs/UPSTREAM.md`. `main@b7e7c1c` is an arbitrary instant in someone else's
   history, not a named point. "Orcker forked Yerd v2.1.0-rc.1" is a fact a reader
   can check; "Orcker forked commit b7e7c1c" is a fact they have to go look up.
2. **Unreleased state carries unmeasured risk.** The tag went through Yerd's release
   pipeline (multi-platform build, signing, packaging). `b7e7c1c` only went through
   `ci.yml` (fmt, clippy, tests). SPEC-0001 AC1 demands green on Linux *and* macOS —
   inheriting an unreleased head means inheriting risk nobody measured.
3. **Noise in the first cycle's bisect.** SPEC-0001 renames 511 files. If the gate
   goes red, the question is "did my rename break this, or was it already broken?".
   A released tag makes the answer trivially "my rename". A moving head makes it an
   investigation — and the first cycle is exactly where the process is being
   calibrated, so noise there is expensive.
4. **Half the delta is work this roadmap has already condemned:**

| Commit | Touches | Fate in Orcker |
|---|---|---|
| `b7e7c1c` pcov / `YERD_COVER` | `bin/yerd/src/cover_shim.rs`, `cli_shim.rs`, tests (+450 lines) | coverage shim for the **native PHP** runtime. SPEC-0002 removes that runtime; PHP moves into containers. Likely deleted |
| `525da8a` Site Redesign | `docs/.vitepress/**`, ~30 files | Yerd-branded VitePress site. SPEC-0001 R5 strips the brand; E14 rewrites the docs. Deleted |
| `1911d3e` macOS zoom | `apps/yerd-gui/src-tauri/src/mac_zoom.rs` (+303) | GUI is kept, but this is cosmetic and macOS-only |
| `7d69d6a` NetworkManager flags | `bin/yerd-helper/src/ops/resolver.rs` (+14) | **real Linux DNS fix** in the privileged helper. Orcker keeps this code |

   Two of four get deleted by the roadmap itself, one is cosmetic, and exactly one
   has durable value — and that one fits in `git cherry-pick 7d69d6a`.

### What the head had going for it, and why it lost

State the counter-case honestly — it is not empty:

1. `origin/main` already publishes `b7e7c1c`, so the tag costs a `--force` on the
   first push. **Answered:** the fork is four days old, unreferenced by anyone, and
   the force push happens once, now, before any clone exists.
2. `7d69d6a` would already be in, with no cherry-pick to remember. **Answered:** a
   remembered cherry-pick is a spec file in the queue, not a memory — SPEC-0028.
3. The delta is four commits over five days, two of them docs-only. Small.
   **Answered:** small is exactly why the loss is cheap. The argument cuts both ways,
   and the tie breaks toward the named, released point.
4. `v2.1.0-rc.1` is a release candidate, not a final. **Conceded:** it is a weaker
   stability marker than a final release. It is still the strongest named point that
   exists, and it did pass the release pipeline that `main` did not.

The decisive item is 4 read together with reason 3 above: the first cycle renames 511
files, and calibrating a brand-new process against a moving base means every red gate
starts as an investigation. A named, pipeline-tested base makes it an answer.

None of this replaces the real control: **`docs/UPSTREAM.md` recording the exact SHA
and the cherry-pick policy.** The base being a tag makes the record cleaner; the
record is what makes future upstream merges deliberate.

### Values for `docs/UPSTREAM.md` (SPEC-0001 R1's deliverable, not written here)

```
repo:   https://github.com/forjedio/yerd
tag:    v2.1.0-rc.1
commit: 896c44938c555d75144ada6da1a72c7d95918a2b
date:   2026-08-15
policy: changes concentrate in new crates; upstream merges are deliberate,
        cherry-picked events (queue them under "Upstream cherry-picks" in
        specs/ROADMAP.md — first entry: SPEC-0028)
```

## D3 — inherited CI (done, scope corrected)

The first report claimed five workflows would fail. Checking the actual `on:` blocks,
only three fire on their own or publish into Yerd's infrastructure:

| Workflow | Was | Now | Why |
|---|---|---|---|
| `sonarqube.yml` | `pull_request` + `push main` | `workflow_dispatch` | `SONAR_*` secrets are Yerd's; every run was red |
| `release.yml` | `push tags v*` | `workflow_dispatch` | signs with Yerd's minisign key, uploads to Yerd's Bunny CDN, posts to Yerd's Discord |
| `docs.yml` | `push tags` + dispatch | `workflow_dispatch` | Pages target and site content are Yerd's |

Left untouched: `build.yml` (`workflow_call` only — inert unless invoked),
`build-cdn.yml` and `cdn-sync.yml` (already `workflow_dispatch` only), `ci.yml` and
`pr-title.yml` (fire on PR/push but use no secrets and are legitimate CI).

Note for SPEC-0001: R6 adds `.github/workflows/gate.yml`, which overlaps `ci.yml`.
Deduplicating them is not in R1-R8 — park it as a `draft` spec rather than widening
the SPEC-0001 diff.

## D4 — GitHub Issues (decided: leave disabled)

Issues off, no milestones, stock labels, zero PRs. The process is spec-file-driven
(`specs/ROADMAP.md`), so a second queue would compete with it. Re-enable later with
`gh repo edit matheusmarnt/orcker --enable-issues` if third-party intake is wanted.
The `module: M1..M8` / `phase: 0..3` label scheme in the parent-directory `CLAUDE.md`
describes the pre-fork Orcker project and does not apply to this repository.

## Also done

- `CLAUDE.md` Git section: explicit ban on AI co-authorship trailers
  (`Co-Authored-By:` naming Claude/Anthropic, `Claude-Session:`), repeated in the S8
  step of `/spec-verify` so the rule sits where the commit is actually made.
- `.gitignore`: `node-compile-cache/` (Node 24 bytecode cache) excluded.

## Remaining, in order

```bash
# 1. gate green on the untouched tree (fmt, clippy -D warnings, full suite)
scripts/gate.sh

# 2. baseline commit — harness only, no product code
git add CLAUDE.md .claude scripts specs docs/PRD.md docs/SDD.md \
        README-INSTALL.md .gitignore .github/workflows
git commit -m "chore(harness): install spec-driven agentic harness"

# 3. human sign-off on the first spec
sed -i 's/^status: draft/status: approved/' specs/SPEC-0001-*.md

# 4. start the loop — first product code of the project
/spec-next        # -> SPEC-0001, branch feat/SPEC-0001-fork-bootstrap
```

Step 4 is where coding starts: **SPEC-0001 fork-bootstrap** (covers FR-001) —
mechanical rename of `yerd-*` to `orcker-*` across 511 files, plus `docs/UPSTREAM.md`
and `.github/workflows/gate.yml`, with zero behaviour change.
