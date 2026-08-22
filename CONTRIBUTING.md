# Contributing to Orcker

Orcker is a Docker-backed local development orchestrator for PHP and Laravel.
It is a fork (see `docs/UPSTREAM.md`), pre-release, and under active,
spec-driven development.

## Read these first

| Question | File |
| --- | --- |
| What is Orcker supposed to do? | `docs/PRD.md` |
| How does work get done here? | `docs/SDD.md` |
| What is queued next? | `specs/ROADMAP.md` |
| Day-to-day rules for agents and humans | `CLAUDE.md` |
| Per-crate code rules | `.github/instructions/` |
| Where the fork came from | `docs/UPSTREAM.md` |
| How to build and run from source | `docs/developer/building.md` |

## The one process rule

No product code without an `approved` spec in `specs/`. One spec per branch,
one atomic commit per accepted spec, and the diff stays inside the spec's
declared `surface`. `docs/SDD.md` is the full contract; everything else in this
file is a summary of it.

Found something worth fixing that is outside the spec you are on? Add a
three-line `draft` spec and carry on. Do not widen the current diff.

## The one architecture rule

> Pure logic lives in library crates. I/O and OS calls are pushed to the edges
> behind traits.

No `unsafe`, and no `unwrap` / `expect` / `panic!` / `todo!` / `dbg!` in
non-test code — the workspace lints enforce both. Errors are `thiserror` enums
in libraries and `anyhow` only at binary top level. TLS is rustls plus rcgen,
never OpenSSL.

## Before you open a pull request

Run the same gate CI runs:

```sh
scripts/gate.sh specs/SPEC-XXXX-your-spec.md
```

It is fmt, clippy with `-D warnings`, the full test suite, the GUI checks when
the GUI is touched, the clippy-allow ratchet, and the surface check. Never
weaken the gate, the lints, or an existing test to make it pass. Never skip a
git hook.

Commits follow Conventional Commits with a crate scope, and the body references
the SPEC and FR ids.

## Reporting bugs and requesting features

Open an [issue](https://github.com/matheusmarnt/orcker/issues/new). Include your
OS, your Docker version, and the exact commands you ran. Security-sensitive
reports go through `SECURITY.md` instead.

## Licence

By contributing you agree that your contributions are licensed under the MIT
licence, as described in `LICENSE.md`.
