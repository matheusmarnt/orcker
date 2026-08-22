# Security Policy

Orcker binds local ports, manages TLS certificates and a local trust store,
generates container stacks, and elevates privileges once during setup (through
the `orcker-helper` boundary). Reports that help keep that surface safe are
welcome.

## Supported versions

Orcker is pre-release and pinned at `0.0.0` until the MVP gate. There is no
supported release line yet: please reproduce against the `main` branch.

## Reporting a vulnerability

**Please do not open a public GitHub issue for a security-sensitive report.**

Use GitHub's
[private vulnerability reporting](https://github.com/matheusmarnt/orcker/security/advisories/new).
Include:

- a description of the vulnerability and its impact;
- the commit you reproduced against, and the platform (macOS / Linux);
- steps to reproduce, or a proof of concept, if you have one;
- any relevant logs (daemon or GUI), with secrets redacted.

For a non-sensitive hardening suggestion with no exploit, a regular
[issue](https://github.com/matheusmarnt/orcker/issues/new) is fine.

Please allow a reasonable opportunity to release a fix before public
disclosure.

## Upstream

Orcker is a fork and inherits much of its upstream's code; `docs/UPSTREAM.md`
records which project, at which commit. A vulnerability in inherited code
affects that project too — report it here, and it will be escalated upstream.
