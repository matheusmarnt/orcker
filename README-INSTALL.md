# Orcker agentic harness — install

Files ready to copy into the root of your Orcker fork. Generated 2026-08-20 from
`orcker-prd.md` v1.0 and `orcker-sdd.md` v1.0 (keep both at `docs/prd-sdd/PRD.md` and
`docs/prd-sdd/SDD.md` in the repo — the harness references those paths).

## Contents

```
CLAUDE.md                     agent memory (replaces Orcker's CLAUDE.md)
.claude/settings.json         permissions + rustfmt-on-edit hook
.claude/agents/supervisor.md  acceptance gatekeeper (SDD section 8)
.claude/agents/spec-writer.md drafts specs from PRD requirements
.claude/commands/spec-next.md /spec-next  (SDD loop S0-S3)
.claude/commands/spec-verify.md /spec-verify (SDD loop S5-S8)
.claude/commands/spec-new.md  /spec-new FR-xxx
scripts/gate.sh               deterministic gate (local == CI)
scripts/surface-check.sh      diff must stay inside the spec's surface
specs/_TEMPLATE.md            spec contract template
specs/ROADMAP.md              Phase-0 queue + Phase-1 planned backlog
specs/TRACEABILITY.md         FR <-> spec <-> tests <-> commit matrix
specs/DECISIONS.md            cycle decisions log
specs/logs/                   cycle logs (created per spec)
specs/SPEC-0001..0006-*.md    the six Phase-0 specs, fully written
```

## Install

1. Fork/clone Orcker at a stable tag; from this package's directory:
   `cp -r CLAUDE.md .claude scripts specs <fork-root>/` and
   `mkdir -p <fork-root>/docs && cp <your PRD/SDD files> <fork-root>/docs/`
   (rename to `PRD.md` and `SDD.md`).
2. `chmod +x scripts/gate.sh scripts/surface-check.sh`
3. Requirements on your machine: Rust toolchain (the repo pins it), Node 20+,
   `rg` (ripgrep), `jq` (used by the hook), Docker Engine + compose v2.
4. Review each `specs/SPEC-000X-*.md`. Flipping `status: draft` to
   `status: approved` **is your sign-off** (SDD section 5 — the only mandatory
   human transition). To approve all six after reading:
   `sed -i 's/^status: draft/status: approved/' specs/SPEC-000*.md`
5. Keep Orcker's `.github/instructions/` files — CLAUDE.md points to them;
   SPEC-0001 renames their contents.
6. Start: `claude` in the repo root, then `/spec-next`. Verify a finished spec
   with `/spec-verify SPEC-0001`. Merge/push remain manual (human) actions.

Note: `.claude/settings.json` is the project-level shared config; personal
overrides go in `.claude/settings.local.json` (gitignored by Claude Code).
