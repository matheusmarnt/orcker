---
id: SPEC-0055
title: The ValidateErrorReason Display smoke test enumerates its variants by hand and has fallen behind
phase: 0
covers: [FR-001]
depends_on: [SPEC-0053]
surface:
  - crates/orcker-config/
status: draft
attempts: 0
---

## Context

`error::tests::display_validate_each_variant_non_empty` asserts every
`ValidateErrorReason` renders a non-empty message, from a hand-written array. It
is missing `RouteRuleDuplicatePrefix` and `RouteRuleUnknownSite`, so two reasons
have never been covered. Found while adding SPEC-0053's three variants to the
same list; reverting the unrelated additions kept that diff inside R4.

The list cannot go stale silently: an exhaustive `match` in a helper that returns
every variant would make the compiler flag the next omission, which is the point
of the test in the first place.

## Requirements

- R1. Every `ValidateErrorReason` is covered without a hand-maintained list, so
      adding a variant fails to compile until it is included.

## Acceptance checklist

- [ ] AC1 (R1) removing a variant from the enumeration is a compile error, not a
      silently narrower test
- [ ] AC2 `scripts/gate.sh specs/SPEC-0055-*.md` passes
