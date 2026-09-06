---
applyTo: "crates/orcker-ipc/**/*.rs"
---

# orcker-ipc — protocol & framing

The request/response protocol between clients (CLI, GUI) and the daemon, plus
the length-prefixed framing that carries it over a socket/pipe.

**Layer:** the codec is pure and runtime-free. Transport is **feature-gated**
behind `transport` (the only part allowed to touch `tokio`).

## Owns

- `Request`, `Response`, `ErrorCode`/status — internally tagged on `type`,
  `snake_case`.
- The pure frame codec (`frame.rs`): `encode_frame`, `FrameDecoder` (partial
  reads, multiple frames per buffer, oversized-length rejection), and
  `encode_message` / `decode_message`.
- Optionally, an async connect/read/write transport helper, shared by `orckerd`
  and the CLI — **only** under the `transport` feature.

## Must not

- Bind or own a socket/pipe in the default build. No async in the default
  feature set — keep the codec runtime-free so pure clients can use it.
- Reimplement domain logic; depend on `orcker-core` for domain types only.

## Contract rules (this is a published contract)

- Add variants and fields **additively**. Never rename or reorder a variant or
  field silently — `tests/wire_stability.rs` will (and should) fail.
- Expand `ErrorCode` rather than overloading a generic variant.
- Introduce/raise the protocol version constant on any breaking change, with a
  handshake before the first incompatible change.

## Precheck before adding a field to an existing variant

Before adding a field to an existing `Request`/`Response` variant, or to a
struct one carries, run `git grep -nE '(Request|Response)::<Variant> \{' --
'*.rs'` (substitute the real variant name) and confirm every hit is inside the
spec's declared `surface:`. A `{ .. }` pattern is safe; any hit that names
fields is not, since it breaks the same way a literal does. A hit outside the
surface means the field is not available there yet: either declare those
files in `surface:` up front, or use a new variant instead.

Escape hatch: a separate request merged at render time, as `orcker status`
does with `Request::Status` + `Request::EngineStatus`
(`bin/orcker/src/lib.rs`, `bin/orcker/src/map.rs`) and `orcker sites` does
with `Request::ListSites` + `Request::ListProjects` (`bin/orcker/src/lib.rs`).

Two cycles paid for skipping this: SPEC-0004 wanted a field on `StatusReport`
(see the trap below) and SPEC-0006 wanted a `projects` field on
`Response::Sites`, blocked by a literal in `crates/orcker-mcp/tests/render.rs`
and resolved only after the code was written, by splitting into a separate
`Response::Projects`.

## The `StatusReport` trap

`StatusReport` is **not** `#[non_exhaustive]` and is built with a full struct
literal at nine sites across five crates: the one production site is
`bin/orckerd/src/ipc_server.rs`; the other eight are test fixtures —
`bin/orcker/src/map.rs`, `bin/orcker/src/mcp_cmd.rs`,
`crates/orcker-doctor/src/lib.rs`, `crates/orcker-ipc/src/response.rs`,
`crates/orcker-ipc/tests/roundtrip.rs`, `crates/orcker-ipc/tests/wire_stability.rs`
(×2), and `crates/orcker-mcp/tests/render.rs`. Adding a field to it breaks
every one of them at once. A spec that intends to extend `StatusReport` must
declare all nine sites in its `surface:` up front — run
`git grep -nE 'StatusReport \{' -- '*.rs'` before writing the design.

Two apparent fixes do not work, so do not re-propose them:
`#[non_exhaustive]` would forbid the daemon's own literal in
`ipc_server.rs`, and `derive(Default)` does not compile because
`dns_addr: SocketAddr` has no `Default`. SPEC-0020 (doctor Docker checks) is
the first queued spec that will hit this, because `orcker_doctor::diagnose`
takes a `&StatusReport`.

## Tests / invariants

- `tests/frame_codec.rs` — partial, pipelined, oversized, and empty frames.
- `tests/roundtrip.rs` — message encode/decode.
- `tests/wire_stability.rs` — exact JSON tags/shapes. Treat a failure as a
  contract alarm, not a test to "fix".

## Review checklist

- [ ] Protocol change is additive; no silent rename/reorder.
- [ ] `transport`/`tokio` stays behind the feature gate; default build is pure.
- [ ] Protocol version bumped if the change is breaking.
- [ ] Wire-stability + codec tests pass and were updated intentionally.
