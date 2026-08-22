# Upstream freeze

Orcker is a fork of [Yerd](https://github.com/forjedio/yerd). This file records
the exact point the fork was taken from and the policy for taking anything else.

## Freeze point

| Field | Value |
| --- | --- |
| Upstream repository | `https://github.com/forjedio/yerd` (git remote `upstream`) |
| Tag | `v2.1.0-rc.1` |
| Commit | `896c44938c555d75144ada6da1a72c7d95918a2b` |
| Upstream commit date | 2026-08-15 |
| Fork freeze date | 2026-08-20 |
| Upstream licence | MIT — `Copyright (c) 2026 Forjed` |

Everything in this repository up to and including that commit is Yerd's work.
Orcker's own history starts after it.

## Policy

Changes concentrate in new crates; upstream merges are deliberate,
cherry-picked events.

In practice:

- No routine `git merge upstream/main`, no tracking branch, no scheduled sync.
  The fork diverges on purpose: Orcker replaces Yerd's native runtime with
  Docker, so most upstream commits land on code Orcker is removing.
- New behaviour goes into new `orcker-*` crates rather than into edits of the
  inherited ones, so the two histories stay easy to tell apart.
- Taking an upstream fix is a spec of its own. It names the upstream commit,
  cherry-picks it, and carries the rename through by hand.
  `specs/SPEC-0028-cherry-pick-networkmanager-reload-fix.md` is the worked
  example.
- The `upstream` remote stays configured for reading — `git log upstream/main`,
  `git show <sha>` — never for merging.

## Renames applied at the fork

The freeze commit is Yerd-named throughout. SPEC-0001 renamed it wholesale, so
any upstream commit picked up later needs the same substitution applied:

| Upstream | Orcker |
| --- | --- |
| `yerd-*` crates | `orcker-*` |
| `yerdd`, `yerd`, `yerd-helper` binaries | `orckerd`, `orcker`, `orcker-helper` |
| `yerd_*` Rust identifiers | `orcker_*` |
| `apps/yerd-gui` | `apps/orcker-gui` |
| `io.yerd.Yerd` bundle identifier | `io.orcker.Orcker` |

The `.test` TLD and the IPC `PROTOCOL_VERSION` were deliberately left
unchanged.
