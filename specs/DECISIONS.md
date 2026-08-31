# Cycle decisions log

Deviations, clarifications and trade-offs recorded by implementation cycles
(SDD section 6, S8). Newest first. Entry format:

```
## YYYY-MM-DD · SPEC-XXXX
- Decision: <what was decided>
- Why: <reason in 1-2 lines>
- Impact: <files/specs/requirements affected; follow-up spec id if any>
```

## 2026-08-30 · SPEC-0042 — historical records are never rewritten

- Decision: `specs/TRACEABILITY.md` and `specs/logs/*.md` keep the wrong
  filename `referenciadockerlaravel.md` where they already carry it; only live
  citations (`docs/PRD.md` via RFC, SPEC-0003, SPEC-0005) are repointed.
- Why: those two files state what was true when SPEC-0003 closed, and at that
  moment the reference document genuinely was not in the repository. Editing
  them would falsify a closed record rather than fix a citation.
- Impact: repo-wide policy, not a SPEC-0042 detail. Known cost, named by the
  supervisor: `specs/TRACEABILITY.md:16` asserts the document is "absent from
  the repo", which is false as a present-tense reading from this cycle onward.
  It is left standing as a statement about SPEC-0003's cycle, not about today.
  Nothing in `scripts/gate.sh` enforces this policy; a future cycle can
  reintroduce the wrong name in a new file unnoticed. SPEC-0040
  (`dead-export-ratchet`) is the precedent if a guard is wanted.

## 2026-08-30 · SPEC-0042 — the reference document stays at `docs/`, not `docs/reference/`

- Decision: `docs/referencia-docker-laravel.md` is committed where it already
  lives, and is not moved under `docs/reference/`.
- Why: SPEC-0042's `HEAD` Context left this open ("either import the document
  under `docs/reference/` or replace every citation with the real source"). The
  uncommitted Context this cycle inherited had already chosen the second branch.
  A move would invalidate the very citations this spec repoints, and `docs/`
  already holds `PRD.md`, `SDD.md` and `UPSTREAM.md` as peers.
- Impact: settles an either/or that the human wrote into the spec. This entry is
  the record of that choice, and the supervisor's ESCALATE (SDD section 8.3)
  turns on whether the cycle had the authority to make it. If the human prefers
  `docs/reference/`, this entry and SPEC-0042's R1/R2 are what change.


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

- Decision: SPEC-0002 deletes `orcker-supervise` by **folding** its five modules
  into `crates/orcker-tunnel/src/supervise/`, rather than keeping the crate.
- Why: PRD FR-002 names all three crates and its AC1 requires the workspace green
  "sem as três crates", but `orcker-tunnel` — kept by the spec's Out of scope and
  scheduled by FR-120/SPEC-0026 — is a live compile-level consumer. Keeping the
  crate was proposed first and withdrawn: it fails FR-002's main clause (the
  substrate would stay in the runtime graph via the tunnel) and AC1 literally,
  and `docs/PRD.md:246` bars agents from editing the PRD, so it would have
  required a `docs/rfc/` proposal and a PRD minor bump before this spec could be
  accepted. Folding satisfies FR-002 AC1 and AC2, keeps the tunnel, and needs no
  PRD change.
- Impact: `orcker-tunnel` absorbs the `tokio` features, `async-trait` and `nix`
  that `orcker-supervise` declared — no new workspace dependency, all three are
  already pinned in `[workspace.dependencies]`. If a second consumer appears
  (SPEC-0010's compose lifecycle is the plausible one), the substrate is
  extracted back into its own crate by that spec, not carried speculatively.

- Decision: grant SPEC-0002 an explicit exception to SDD §8.1 **DT5**
  ("wire-stability tests untouched; protocol changes additive only") for its R4
  wire reset: `PROTOCOL_VERSION` 1 -> 2 and deletion of the removed variants'
  assertions.
- Why: PRD FR-002 permits "comandos/mensagens IPC correspondentes **removidos**
  ou marcados `deprecated` de forma aditiva", and SDD §5 gives the PRD precedence
  on the *what*. The additive alternative would keep ~45 zombie request variants
  alive, each needing an `unsupported` arm in the daemon, plus `PhpPoolStatus`,
  `ServiceStatus` and `DatabaseSummary` alive so `StatusReport` still
  deserializes — permanent complexity for zero consumers: the workspace is
  `version = "0.0.0"`, `publish = false`, no release exists, and SPEC-0001
  renamed the IPC endpoint (`io.yerd.Yerd` -> `io.orcker.Orcker`), so no daemon
  in the field speaks this protocol.
- Impact: the exception is narrow and comes with two guardrails.
  (1) `tests/wire_stability.rs` is edited by deletion only — surviving literals
  stay byte-identical and the file is never regenerated, because a regenerated
  baseline can bury a typo in a kept variant and ship a silent protocol bug.
  One named exception, added at S7 after the supervisor caught the guardrail
  overstating what the diff does: a test pinning a **surviving** response whose
  payload is a **removed** enum variant cannot stay byte-identical, since the
  variant is gone. It is retargeted rather than deleted, because deleting it
  would drop coverage of a surviving response. Exactly one test qualifies —
  `response_doctor_fix_byte_shape`, whose `FixResult.code` moves from
  `fpm_pool_failed` to `resolver_not_installed`. Final diff: 8 added lines,
  1403 removed; five of the eight are within-line deletions (four import-list reflows and the status literal); the other three are the one authorized `response_doctor_fix_byte_shape` retarget.
  (2) `PROTOCOL_VERSION` does not travel on the wire and has no handshake, so
  the bump is informational; its doc says so until
  `specs/SPEC-0034-ipc-version-skew-handshake.md` closes the gap. A future spec
  wanting to touch the wire needs its own decision here, not this precedent.

## D14 — SPEC-0002 split at `attempts = 3`: runtime removal lands, coverage becomes SPEC-0037

- Date: 2026-08-22 · Decided by: the human, at the ESCALATE the SDD forces at
  `attempts = 3` (SDD §117, §245). Options offered were re-derive mechanically,
  split, or abort; the human chose **split**.
- Context: three consecutive supervisor passes returned REWORK, all on DT7, each
  naming tests the previous pass had not caught - and the second naming two the
  first had explicitly cleared. Every finding had the same shape: a test deleted
  during the removal that still covered surviving behaviour.
- Measurement that settled it (reproducible; method in `specs/logs/SPEC-0002.md`
  S7 attempt 3):
  - 671 test fns deleted in total; 445 died with their file (R1/R2 deleted the
    crate or module - authorized), 90 were `wire_stability.rs` pins (R4
    reset, verified byte-identical for every surviving literal).
  - **136 were deleted from files that still exist.** That is the whole DT7
    surface. 26 have been dispositioned across the three attempts; 110 had never
    been individually read by anyone.
  - A compiler-based classifier ("restore everything, delete only what fails to
    compile") was built and validated against the supervisor's proven-alive set:
    it discards **4 live tests out of 5**. A symbol-based refinement fails
    identically. Root cause: the unit of judgement is the **assertion**, not the
    test - a 12-row table with 5 dead rows fails to compile as a whole while 7
    rows still cover surviving tools. The single most common missing symbol in
    those failures is `PhpVersion`, which R6 explicitly **keeps**; it is absent
    only because an automated import-pruner removed the `use` line.
- Decision: SPEC-0002 is accepted on runtime removal alone. Its Test plan clause
  requiring a per-case written justification for each deletion outside the removed
  crates is **withdrawn** and replaced by a delegation to SPEC-0037, which makes
  the disposition of all 136 its acceptance criterion. The enumerated list is
  `specs/logs/SPEC-0002-deleted-tests.md`.
- Why not the alternatives: mechanical re-derivation was refuted by the
  measurement above. Splitting by surface (IPC / daemon / CLI / GUI) does not
  reduce the review - the 136 span 12 files across every surface - it only
  spreads it across more cycles.
- Impact and guardrails: SPEC-0037 is queued immediately after SPEC-0002 and
  **blocks SPEC-0003**, so no Docker work starts on a suite whose coverage is
  unaudited. SPEC-0037's AC1 is mechanically checkable (no row left unmarked),
  which the withdrawn clause never was - that is the actual defect this decision
  repairs. The `attempts = 3` on SPEC-0002 stands in the record; the history is
  not rewritten.
- What this precedent does **not** authorize: it is not a licence to defer test
  coverage out of any spec that finds it inconvenient. It applies where coverage
  is a bounded, enumerated set that the spec's own acceptance criteria cannot
  express, and it requires the follow-up spec to exist and block before the work
  it protects.

## SPEC-0033 — the R2 wording grep stays in the cycle log, not in the gate

- Decision: the `grep -rniE "native child process|PHP-FPM|FPM pool|native
  (db|database|engine|runtime)"` criterion added at S3b is **cycle-local
  evidence only**. It is not promoted into `scripts/`, into `scripts/gate.sh`,
  or into any future acceptance criterion.
- Why: two defects, both found by the supervisor at attempt 2. It is line-based,
  so it cannot see a literal split across a wrap — a correct negation at
  `.github/instructions/orckerd.instructions.md:36-38` survived it unseen, which
  means its GREEN proves "absent from every single line", not "absent". And it
  forbids `PHP-FPM`, which `docs/PRD.md:14` makes a *current* term: the app
  container runs PHP-FPM + Supervisor. As a standing guard it would block
  accurate documentation of the containerised app.
- What was done instead: at attempt 2 two correct negations were reworded to
  drop the banned literals rather than the pattern being weakened to admit them.
  The rewordings are semantically identical to what they replaced. R2's real
  verification was the supervisor's independent whole-file,
  whitespace-normalised sweep of `.github/`.
- If a durable guard is wanted it needs its own spec, and it must match
  *assertions* about host-side runtimes rather than substrings. Related gap,
  also unguarded: nothing detects an orphaned
  `.github/instructions/*.instructions.md` after a crate deletion, so a future
  SPEC-0002-style removal can re-orphan a file silently — which is exactly how
  SPEC-0033 came to exist.

## 2026-08-30 · SPEC-0003 — `Ports` replaces R2's two separate port fields

- Decision: R2 lists `http_loopback_port: u16` and `vite_port: u16` as two
  `StackConfig` fields. They were implemented as one validated `Ports` pair type
  (`crates/orcker-stack/src/config.rs`) exposing `http_loopback()` / `vite()`.
- Why: R5 requires the ports to be "non-zero and distinct". Distinctness is a
  property of the pair, not of either value, so with two independent fields the
  check has nowhere honest to live and would have to be re-run by every future
  constructor. Bundling them also holds `StackConfig::new` at seven arguments,
  under clippy pedantic's `too_many_arguments` threshold, without an `#[allow]`.
- Impact: the rendered output is unchanged, so no acceptance criterion moves.
  Consumer specs (SPEC-0012/0013) construct `Ports::new(http, vite)?` before
  `StackConfig::new`. Non-blocking wart the supervisor flagged for those specs:
  `StackConfig::new` still returns `Result` with no failure path now that the
  validation moved into `Ports`; collapse it when a consumer wires the crate.

## 2026-08-30 · SPEC-0003 — `orcker-stack` deliberately does not depend on `orcker-core`

- Decision: `orcker-stack` defines its own `SiteName` and `PhpVersion` instead of
  reusing `orcker-core`'s, and depends only on `thiserror`.
- Why: `orcker_core::PhpVersion` is a `(major, minor)` struct accepting
  `major in 5..=9`, which cannot express R5's `8.1..=8.5` closed set, and core's
  site-name validator is private (only `normalize_site_name` / `slugify_site_name`
  are public). Reusing either would have meant widening `orcker-core`'s public API,
  and `crates/orcker-core/` is not in SPEC-0003's `surface`.
- Impact: DNS-label validation now exists in two pure crates. Accepted for now
  because the two rule sets are not the same one (core's allows dots for hosts,
  stack's is a single Docker/DNS label) and the duplication is ~15 lines. If a
  third copy appears, consolidate via a spec whose surface covers both crates.
  `orcker-stack` is a workspace member only and is in no binary's runtime graph,
  so `no_runtime_deps` guards elsewhere are untouched.


## 2026-08-31 · SPEC-0045 — five files changed where *Design & contracts* named three

- Decision: beyond `docs/SDD.md` (sections 5, 8.1, 8.3), `.claude/commands/spec-next.md`
  and the spec itself, the cycle also edited `docs/SDD.md` sections 8.4, 9.2 and 9.3,
  `.claude/agents/supervisor.md` and `specs/ROADMAP.md`'s header sentence.
- Why: R2 and R3 are rules about *where the loop reads a status* and *what the
  supervisor checks*, and each of those places holds a second copy of the text the
  spec names. `.claude/agents/supervisor.md` is the file that executes the 8.1 layer,
  so a `DT9` living only in `docs/SDD.md` would never run and R3 would be inert; the
  spec's *Out of scope* anticipates exactly this by permitting the supervisor
  definition to change "beyond the DT9 row". Sections 9.2 and 9.3 embed copies of the
  command and the agent, and a copy left unedited would have contradicted section 8.1
  two screens above it — the stale-citation defect SPEC-0042 existed to remove. The
  `ROADMAP.md` header sentence restates the selection rule R2 replaces.
- Impact: the diff is larger than the spec's file list, and every added file is inside
  the declared `surface` (`docs/`, `specs/`, `.claude/`), so DT2 holds. The supervisor
  upheld the reasoning in round 1 and rejected the round-1 diff for applying it to
  section 9.2 and not to 9.3; the fix carried it to every copy. Ongoing rule for this
  repository: an embedded copy in section 9 is part of the change to the section it
  copies, not a separate spec.


## 2026-08-31 · SPEC-0005 — the spike needed no code, and SPEC-0006 may need less than queued

- Decision: R2's conditional code delta was not taken. `orcker proxy add spike
  http://127.0.0.1:18080` plus `orcker secure spike` served a containerized Laravel app
  at `https://spike.test` with a CA-issued leaf, so the declared code surface
  (`crates/orcker-core`, `crates/orcker-config`, `bin/orckerd`, `bin/orcker`) stayed
  untouched and the diff is docs + specs only.
- Why: R2 asks for a delta *only if* the inherited custom-proxy mechanism cannot express
  a loopback upstream. `bin/orcker/src/cli.rs` `ProxyAction::Add` already creates a
  whole-host proxy against any `http://127.0.0.1:<port>`, and the inherited
  `forward/http.rs` / `forward/upgrade.rs` carry both the HTTP and the websocket path.
  Measured: `101 Switching Protocols` through the proxy with the `vite-hmr` subprotocol
  intact, and, warm, medians of ~25.7-31.3 ms direct against nginx versus 113-125 ms
  through the proxy on a **debug build**. The gap is the per-request TLS handshake
  (`time_appconnect` 40-76 ms against `time_connect` ~3 ms), not forwarding. A first
  measurement suggesting no overhead ran against cold opcache and is retracted; the
  release-build cost is unmeasured and is an NFR-02 / Phase-1 question.
- Impact on SPEC-0006: it is queued as "link/loopback port", but routing to a loopback
  port is already expressed. What is actually missing is port allocation and a project
  registry. SPEC-0006 must be re-read before it is drafted further - see finding F7 in
  `docs/spike/PHASE0-SPIKE.md`.
- Impact on SPEC-0007 (preset): the generated stack must emit Vite's
  `server.allowedHosts` for the project's `.test` domain and an `server.hmr.clientPort`
  matching the port the browser reaches; without it Vite answers 403 to any proxied
  request (F5). It must also mount `postgres:18` at `/var/lib/postgresql` (F4) and carry
  `libonig-dev` (F2), both bugs inherited from `docs/referencia-docker-laravel.md`.
