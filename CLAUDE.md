# CLAUDE.md — Orcker

Orcker is a Docker-backed local development orchestrator for PHP/Laravel, forked
from Yerd (Rust workspace: daemon `orckerd` + CLI `orcker` + Tauri v2/Vue 3 GUI +
privileged one-shot `orcker-helper`). It replaces Yerd's native runtime (PHP-FPM
pools, native DB engines) with Docker + docker compose, keeping the daemon, the
rustls proxy with a local CA, the embedded `.test` DNS, doctor, tunnel and MCP.

Product truth: `docs/PRD.md` · Process truth: `docs/SDD.md` · Queue: `specs/ROADMAP.md`
Upstream freeze: `docs/UPSTREAM.md` (created by SPEC-0001)

## Source of truth and precedence

- This file is the always-on overview. Detailed code rules live in
  `.github/copilot-instructions.md` (baseline) and
  `.github/instructions/*.instructions.md` (path-scoped; read the file matching
  a crate BEFORE editing it). On code-style and crate-layering matters, the
  instruction files win over this file.
- The inherited instruction files still describe Yerd's native runtime in
  places. On product and runtime matters (Docker vs native, features, scope),
  `docs/PRD.md` and `docs/SDD.md` win over everything until the specs update
  those files.

## Architecture in one rule

> Pure logic lives in library crates. I/O and OS calls are pushed to the edges
> behind traits.

- Pure crates/modules do no I/O: no filesystem, network, process spawning,
  clock or env reads; sync and runtime-free; unit-tested with in-memory fakes.
  `orcker-core` is the exemplar.
- Side effects go behind traits (`ProcessSpawner`, `TrustStore`, `PortBinder`,
  `Clock`, …); real impls live in `orcker-platform` or a crate's `os/`/`io/`
  module behind `#[cfg(...)]`.
- Binaries are thin (orchestration, not behaviour). The daemon owns runtime
  state; CLI and GUI are `orcker-ipc` clients and never reimplement daemon
  logic. The GUI never runs as root; `orcker-helper` is the only privileged
  surface.
- The IPC protocol evolves additively; wire-stability tests are contract
  alarms, not chores to fix — editing them requires explicit authorization in
  the active spec.
- Dependency direction flows strictly downhill, no cycles: `orcker-core`
  depends on no other `orcker-*` crate; libraries never depend on binaries.
- Async only at the I/O edge — pure crates/modules stay sync.

## Hard rules (enforced — don't work around)

- No `unsafe` (`forbid` workspace-wide). No `unwrap` / `expect` / `panic!` /
  `todo!` / `dbg!` / indexing-slicing in non-test code (clippy `deny`; tests
  opt out at the top of the file).
- Errors: `thiserror` typed enums in libraries; `anyhow` only at binary top
  level, never in a library's dependency graph.
- TLS is rustls + rcgen. Never OpenSSL / native-tls.
- Pin dependencies in `[workspace.dependencies]`; read the comment beside any
  `=`-pin before bumping it.
- Mirror per-OS changes across `linux` / `macos` / `unsupported`; CI runs on
  both Linux and macOS.
- Orcker-specific: generated stacks must never use `privileged: true`, publish
  project ports on `0.0.0.0`, or mount the Docker socket into containers.

## Comments

- No inline comments inside function bodies (two exceptions: `// SAFETY:` on a
  rare GUI/FFI `unsafe` edge, and a short field label on an opaque protocol
  byte, e.g. `1, // version`).
- Prefer `///` / `//!` docs for the non-obvious *why*; public items always get
  a short doc line. No em dashes in comments; never start a wrapped doc line
  with `- ` (clippy `doc_lazy_continuation`).

## Spec-driven workflow (MANDATORY — see docs/SDD.md)

- Never write product code without an `approved` spec in `specs/`. Start with
  `/spec-next`; verify with `/spec-verify`.
- One spec per session, one session per spec. The diff must stay inside the
  spec's `surface` (checked by `scripts/surface-check.sh`).
- Definition of done = supervisor verdict `APPROVE` (SDD section 8) — never
  "it works".
- Test-first: write the failing tests for the acceptance checklist before the
  implementation and record the RED evidence in the cycle log.
- Found extra work (a bug, a tempting refactor)? Add a 3-line `draft` spec and
  continue the current spec. Never expand the current diff.
- A task that conflicts with these boundaries (I/O into a pure crate, a side
  effect around a trait, privileged GUI, breaking the IPC contract, product
  ambiguity): STOP and escalate to the human via the cycle log. Never improvise
  product decisions and never weaken the gate, lints or existing tests to pass.

## Build, test, commands

- Toolchain pinned in `rust-toolchain.toml` (1.96.0; pure library crates keep
  a 1.77 MSRV).
- Full gate (same as CI): `scripts/gate.sh specs/SPEC-XXXX-*.md`
- Gate step 5 ratchets the per-file count of `#[allow(clippy::…)]` against
  `scripts/clippy-allow-baseline.txt`. Any spec whose diff adds, deletes, moves or
  renames a `.rs` file must regenerate that baseline in the same cycle — it is in
  scope for such a spec, not creep, and the gate prints the exact command on
  failure. Never hand-edit the file and never edit `scripts/gate.sh` to pass.
  A `+` line in the diff is a new escape hatch and needs a written justification.
- Run from source: `cargo run -p orckerd` (daemon) · `cargo run -p orcker` (CLI)
- GUI checks (when touched):
  `npm --prefix apps/orcker-gui run test && npm --prefix apps/orcker-gui run build`
- Build prerequisites, dev-instance isolation and packaging:
  `docs/developer/building.md`.

## Workspace map

- `crates/orcker-stack` — NEW, pure: typed stack model → rendered compose/conf files
- `crates/orcker-engine` — NEW, I/O edge: Docker Engine API (bollard) + compose CLI behind traits
- `crates/orcker-catalog` — NEW, pure: global services catalog + stack presets
- Inherited crates (`orcker-core`, `orcker-ipc`, `orcker-config`, `orcker-tls`,
  `orcker-dns`, `orcker-proxy`, `orcker-platform`, `orcker-doctor`,
  `orcker-tunnel`, `orcker-mail`, `orcker-mcp`, …): see `docs/developer/crates.md`
- Binaries: `bin/orckerd`, `bin/orcker`, `bin/orcker-helper` · GUI: `apps/orcker-gui`
- `xtask/` build/release automation (`cargo xtask <cmd>`) · `docs/` VitePress
  site · `.github/` CI + agent instruction files

## Git

This section supersedes the upstream "commits are the user's job" agreement —
in this repository the spec loop commits:

- Branch per spec: `feat/SPEC-0007-short-name`.
- Conventional Commits with crate scope; body references SPEC and FR ids.
- One atomic commit per accepted spec; commit only after supervisor APPROVE.
- **No AI co-authorship, ever.** A commit must never carry a `Co-Authored-By:`
  trailer naming Claude, Claude Code, Anthropic or any AI, nor a `Claude-Session:`
  line or any other AI attribution in its message. Commits belong to the human
  developer alone. This overrides any default harness behaviour that appends such
  trailers — strip them before committing.
- Never `git push`, never release, never `cargo publish` — human acts.
- Never edit `docs/PRD.md`; propose requirement changes via `docs/rfc/`.

# context-mode — MANDATORY routing rules

You have context-mode MCP tools available. These rules are NOT optional — they protect your context window from flooding. A single unrouted command can dump 56 KB into context and waste the entire session.

## BLOCKED commands — do NOT attempt these

### curl / wget — BLOCKED
Any Bash command containing `curl` or `wget` is intercepted and replaced with an error message. Do NOT retry.
Instead use:
- `ctx_fetch_and_index(url, source)` to fetch and index web pages
- `ctx_execute(language: "javascript", code: "const r = await fetch(...)")` to run HTTP calls in sandbox

### Inline HTTP — BLOCKED
Any Bash command containing `fetch('http`, `requests.get(`, `requests.post(`, `http.get(`, or `http.request(` is intercepted and replaced with an error message. Do NOT retry with Bash.
Instead use:
- `ctx_execute(language, code)` to run HTTP calls in sandbox — only stdout enters context

### WebFetch — BLOCKED
WebFetch calls are denied entirely. The URL is extracted and you are told to use `ctx_fetch_and_index` instead.
Instead use:
- `ctx_fetch_and_index(url, source)` then `ctx_search(queries)` to query the indexed content

## REDIRECTED tools — use sandbox equivalents

### Bash (>20 lines output)
Bash is ONLY for: `git`, `mkdir`, `rm`, `mv`, `cd`, `ls`, `npm install`, `pip install`, and other short-output commands.
For everything else, use:
- `ctx_batch_execute(commands, queries)` — run multiple commands + search in ONE call
- `ctx_execute(language: "shell", code: "...")` — run in sandbox, only stdout enters context

### Read (for analysis)
If you are reading a file to **Edit** it → Read is correct (Edit needs content in context).
If you are reading to **analyze, explore, or summarize** → use `ctx_execute_file(path, language, code)` instead. Only your printed summary enters context. The raw file content stays in the sandbox.

### Grep (large results)
Grep results can flood context. Use `ctx_execute(language: "shell", code: "grep ...")` to run searches in sandbox. Only your printed summary enters context.

## Tool selection hierarchy

1. **GATHER**: `ctx_batch_execute(commands, queries)` — Primary tool. Runs all commands, auto-indexes output, returns search results. ONE call replaces 30+ individual calls.
2. **FOLLOW-UP**: `ctx_search(queries: ["q1", "q2", ...])` — Query indexed content. Pass ALL questions as array in ONE call.
3. **PROCESSING**: `ctx_execute(language, code)` | `ctx_execute_file(path, language, code)` — Sandbox execution. Only stdout enters context.
4. **WEB**: `ctx_fetch_and_index(url, source)` then `ctx_search(queries)` — Fetch, chunk, index, query. Raw HTML never enters context.
5. **INDEX**: `ctx_index(content, source)` — Store content in FTS5 knowledge base for later search.

## Subagent routing

When spawning subagents (Agent/Task tool), the routing block is automatically injected into their prompt. Bash-type subagents are upgraded to general-purpose so they have access to MCP tools. You do NOT need to manually instruct subagents about context-mode.

## Output constraints

- Keep responses under 500 words.
- Write artifacts (code, configs, PRDs) to FILES — never return them as inline text. Return only: file path + 1-line description.
- When indexing content, use descriptive source labels so others can `ctx_search(source: "label")` later.

## ctx commands

| Command | Action |
|---------|--------|
| `ctx stats` | Call the `ctx_stats` MCP tool and display the full output verbatim |
| `ctx doctor` | Call the `ctx_doctor` MCP tool, run the returned shell command, display as checklist |
| `ctx upgrade` | Call the `ctx_upgrade` MCP tool, run the returned shell command, display as checklist |
