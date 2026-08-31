---
id: SPEC-0037
title: Dispose of every test SPEC-0002 deleted from a surviving file
phase: 0
covers: [FR-002]
depends_on: [SPEC-0002]
surface:
  - crates/
  - bin/
  - apps/orcker-gui/
status: accepted
attempts: 0
---

## Context

SPEC-0002 removed the native runtime: 671 `#[test]` functions disappeared with
it. Most of that is uncontroversial - 445 died with their file when R1/R2 deleted
a crate or module, and 90 more were the `wire_stability.rs` pins of removed IPC
variants, an R4-authorized reset the supervisor verified as byte-identical for
every literal that survives.

The remaining **136 were deleted from files that still exist**. Some genuinely
tested a removed feature. Others did not: three supervisor passes over SPEC-0002
found 26 that covered surviving behaviour, including the only check that the LAN
setup endpoint serves the bytes it advertised a hash for (a script a LAN device
runs as root), the MCP catalog's name and schema contract, and the renderer
behind `orcker status` - which is SPEC-0002's own AC4 evidence. Each pass found a
different subset, because none enumerated the set.

At `attempts = 3` the SDD forced ESCALATE. The human chose to split (see
`specs/DECISIONS.md`): SPEC-0002 lands the runtime removal, and this spec owns the
coverage question as a first-class deliverable with an acceptance criterion that
can actually be checked.

**No mechanical filter can do this work.** It was measured, not assumed: restore
all 136 into a scratch worktree, compile, and classify by "still compiles" or by
"fails only on a symbol that still exists". Both classifiers were validated
against the supervisor's proven-alive set and both discard 4 live tests out of 5.
The reason is structural - the unit of judgement is the **assertion**, not the
test. A 12-row table test whose 5 rows name removed tools fails to compile as a
whole while 7 rows still cover surviving tools. The dominant missing symbol in
those failures is `PhpVersion`, which R6 explicitly **keeps**; it is absent only
because an automated import-pruner removed the `use` line.

So this is a human-judgement review of 136 items, done one at a time.

## Requirements

- R1. For each of the 136 rows in `specs/logs/SPEC-0002-deleted-tests.md`, choose
  exactly one disposition and record it in that file:
  - **restore** - the test covers behaviour this fork keeps; bring it back,
    rewiring moved symbols (`orcker_php::Downloader` ->
    `crate::download::Downloader`, `ext_install::sha256_hex` ->
    `download::sha256_hex`, `current_os_arch` -> `orcker_platform::current_target`)
    and re-adding `use` lines an import-pruner removed;
  - **scrub** - the test is a table or a sequence of assertions, some naming
    removed features and some not; restore it with only the dead rows or
    assertions removed;
  - **retarget** - the test pins generic behaviour but uses a removed symbol as
    its vehicle; restore it against a surviving symbol, naming the substitution;
  - **drop** - the test's subject is genuinely gone. Requires one written line
    saying which deleted item was its subject. "It named a PHP symbol" is not a
    reason; "it asserts on `PhpManager::ensure`, which R1 deleted" is.
- R2. Judge per **assertion**, not per test. A test survives if any of its
  assertions covers surviving behaviour.
- R3. Do not weaken what is already green. Restorations must not delete or
  loosen an assertion that passes today.
- R4. Where a restored test reveals a real gap in the post-SPEC-0002 code (as the
  PHP-CA-bundle-on-every-boot bug was revealed by running the daemon, not by a
  test), raise a `draft` spec rather than fixing it here.

## Design & contracts

The row list is mechanically derived and reproducible: every `#[test]` /
`#[tokio::test]` fn present at the SPEC-0002 base commit and absent from its diff,
restricted to files that still exist. Regenerate it to confirm the set has not
drifted before starting.

Restoration is `git show <base>:<file>` for the fn body plus its doc comment,
then the minimum edit that compiles. Prefer restoring verbatim: an edit is a
judgement that needs justifying, an untouched restore is not.

## Test plan

- The gate must stay green throughout; a restoration that fails is a finding
  about the code, not a licence to delete the test again.
- Manual evidence: the completed row list, every row with a disposition.

## Acceptance checklist

- [ ] AC1 `specs/logs/SPEC-0002-deleted-tests.md` has 136 rows, every one marked
      `restore` / `scrub` / `retarget` / `drop`, with no row left `[ ]`
- [ ] AC2 Every `drop` row carries a one-line written subject, naming the deleted
      item it tested
- [ ] AC3 `cargo test --workspace` green, with a test count strictly greater than
      SPEC-0002's final count -> evidence: gate output before and after
- [ ] AC4 No test passing before this spec is deleted or has an assertion
      loosened -> evidence: the diff is additions plus the dead rows named in AC1
- [ ] AC5 `scripts/gate.sh specs/SPEC-0037-*.md` passes

## Out of scope

Changing behaviour. This spec restores and adapts tests; if a restored test fails
because the code is wrong, that is a `draft` spec (R4), not a fix here. Also out
of scope: the four follow-ups SPEC-0002 already drafted (0033 stale instruction
file, 0034 IPC skew, 0035 config sections, 0036 dead dumps UI).

## Agent notes

Read `specs/logs/SPEC-0002.md` first, in particular the three S7 sections. They
record every classification mistake made during SPEC-0002 and why each was wrong;
the same mistakes are the failure mode here. The one rule that held across all
three supervisor passes: **a test that still compiles against the post-diff API,
or needs only one identifier changed, was not testing the deleted feature.**

Work file by file, smallest first, and run the file's own test target after each.
Do not batch the 136 through a script - scripting this set is what produced the
defect in the first place.
