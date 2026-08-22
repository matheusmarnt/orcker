---
applyTo: "crates/orcker-core/**/*.rs"
---

# orcker-core — pure domain model & routing

The foundation every other crate depends on. It defines the domain values and
the host→site routing that every request flows through.

**Layer:** strictly pure. No async, no I/O, no internal `orcker-*` dependencies.
Crate root carries `#![forbid(unsafe_code)]`.

## Owns

- Domain types: `PhpVersion`, `Site` / `SiteKind`, `Tld`, host parsing, and
  PHP settings (`php_settings`).
- `SiteRouter` / `RouterConfig` and `resolve(host) -> Option<&Site>`: port
  stripping, trailing-dot FQDN handling, case-insensitivity, TLD enforcement,
  exact-match-beats-wildcard, wildcard-subdomain→parent.
- The typed `CoreError` family with specific reason enums.

## Must not

- Perform any I/O: no filesystem, network, process spawning, clock or env reads.
- Depend on `tokio` or any async runtime.
- Depend on any other `orcker-*` crate (it sits at the bottom of the graph).
- Grow knowledge of *where* config lives or *how* sites are discovered — that is
  config/daemon territory.

## Conventions

- New routing/parsing behaviour is added as pure functions with table-driven
  tests. Keep the routing rules pinned by the existing table suites.
- Serialised forms here are a contract for `orcker-config` and `orcker-ipc`. The
  human-editable on-disk form of `PhpVersion` is `"8.3"`; preserve it. Any
  serde-shape change must be reflected in `tests/wire_stability.rs` deliberately.

## Tests / invariants

- `tests/serde_roundtrip.rs` — types round-trip through serde.
- `tests/wire_stability.rs` — exact JSON wire shapes; a diff here means a
  rename/reorder that ripples into IPC and config. Only update it on purpose.

## Review checklist

- [ ] Change is pure — no I/O, clock, env, or async crept in.
- [ ] No new internal dependency.
- [ ] New behaviour is table-tested.
- [ ] Wire-stability test updated only for an intended serde-shape change.
