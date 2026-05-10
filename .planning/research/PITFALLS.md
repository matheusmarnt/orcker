# Domain Pitfalls — Tauri 2 + Rust + Docker Desktop App

**Domain:** Cross-platform desktop app for Docker infra management (Orcker)
**Researched:** 2026-05-10
**Confidence:** HIGH for well-documented Tauri/bollard pitfalls; MEDIUM for cross-platform signing/build edge cases

---

## Critical Pitfalls

### P-C1: Blocking the Tauri main thread with sync Rust code
**What goes wrong:** A Tauri command marked `fn` (not `async fn`) or one that calls blocking I/O (`std::process::Command`, `std::fs` on large files, sync HTTP) inside the async runtime freezes the WebView event loop. UI becomes unresponsive — buttons don't react, animations stutter — until the call returns.
**Why it happens:** Tauri 2 uses Tokio under the hood. Sync handlers run on the main thread; blocking them blocks IPC dispatch.
**Consequences:** UI freezes during `docker ps`, log tailing, large compose-up. Looks like a hard crash to the user.
**Warning signs:** UI lag during Docker calls; window controls (drag/resize) become sluggish; profiler shows main thread held.
**Prevention:**
- All commands that touch I/O **must be `async fn`** and use `bollard` (already async via Tokio).
- For unavoidable sync code (e.g., `git2`, some FS ops), wrap in `tokio::task::spawn_blocking(...)`.
- Long-running ops (compose up, log tail, db dump): use `tokio::spawn` + `app.emit("event:name", payload)` for streaming progress instead of returning a single `Result`.
- Set explicit `Builder::new().invoke_handler(generate_handler![...])` patterns; avoid `.block_on()` inside commands.
**Detection:** Add a 100ms heartbeat event from JS; if it stops arriving during a backend call, the main thread is blocked.
**Phase to address:** Phase A (foundational adapter layer) — establishing async patterns up front avoids retrofits.
**Maps to:** R01 (Rust learning curve).

---

### P-C2: bollard Docker socket not detected cross-platform
**What goes wrong:** Hardcoding `/var/run/docker.sock` breaks on macOS (Docker Desktop uses `~/.docker/run/docker.sock` or `~/.docker/desktop/docker.sock`), OrbStack (`~/.orbstack/run/docker.sock`), Colima (`~/.colima/default/docker.sock`), and Windows (named pipe `npipe:////./pipe/docker_engine`). Rootless Docker on Linux uses `$XDG_RUNTIME_DIR/docker.sock`.
**Why it happens:** Devs test only one runtime; bollard's `Docker::connect_with_local_defaults()` works on most cases but fails on OrbStack/Colima/rootless without `DOCKER_HOST` set.
**Consequences:** "Docker not running" error for users who clearly have Docker running. Bug reports from macOS/OrbStack users dominate first month.
**Warning signs:** Windows user reports "no such file"; macOS user reports "permission denied" when Docker Desktop is clearly up.
**Prevention:**
- Implement a **socket discovery cascade**: read `DOCKER_HOST` env first → probe known paths in order (Docker Desktop, OrbStack, Colima, rootless, system) → fallback to bollard default → settings override.
- Surface detected socket path in M7 Settings with manual override.
- Test against at least 3 runtimes in CI: Docker Engine (Linux native), Docker Desktop (mac CI), Colima (alternative mac).
- For Windows: use `bollard::Docker::connect_with_named_pipe_defaults()` explicitly on `cfg(windows)`.
**Detection:** Pre-flight check command on app start; show "Docker socket: <path> (auto-detected)" banner.
**Phase to address:** Phase A (M1 Global Stack blocker) — cannot list services without socket.
**Maps to:** R02 (Docker runtime divergence).

---

### P-C3: Docker Engine API version mismatch breaks bollard calls
**What goes wrong:** bollard pins to a specific Docker API version. If user's Docker daemon is older (e.g., Synology NAS, old Ubuntu LTS) or much newer, calls fail with `client version 1.43 is too new` or schema deserialization errors.
**Why it happens:** Docker Engine API evolves; bollard's struct definitions assume a specific version.
**Consequences:** App works for dev's machine, breaks for users with Ubuntu 20.04 LTS or rolling-release distros.
**Warning signs:** Random `serde::de::Error` on container list; "API version negotiation failed"; works locally, fails on user machines.
**Prevention:**
- On first connect, call `Docker::version()` and **negotiate down** with `Docker::client_version()` to a compatible API level.
- Document **minimum supported Docker version** in README (recommend ≥ 20.10, target current = 24+).
- Wrap bollard calls in adapter trait so swapping serialization/version logic is local.
- Pin bollard version in `Cargo.toml`; bump deliberately with regression test against 2-3 Docker versions.
**Detection:** Health-check command on startup logs Docker version + API version; flag mismatches.
**Phase to address:** Phase A (adapter layer).
**Maps to:** R03 (Docker API churn).

---

### P-C4: Tauri CSP misconfigured — XSS or app refuses to load
**What goes wrong:** Two failure modes:
1. CSP too loose (`'unsafe-inline'`, `'unsafe-eval'`): allows XSS via injected log content, crafted `.env` values, or Monaco editor content rendered as HTML.
2. CSP too strict: Tailwind JIT, Vite HMR, Monaco workers, or shadcn/vue inline styles are blocked → blank white window with console errors only visible in devtools.
**Why it happens:** CSP defaults differ between dev and prod; Vite injects inline scripts in dev mode; some libs (Monaco) need `worker-src blob:`.
**Consequences:** Either security vuln (R11-adjacent — sandbox sharing logs externally) or "app won't open" bug reports.
**Warning signs:** Blank window after `tauri build`; console: "Refused to load... violates CSP"; production build differs from dev.
**Prevention:**
- Define separate CSP for dev (loose, needed for HMR) and prod (strict).
- For Monaco editor (M5): allow `worker-src blob: 'self'`, `script-src 'self'`.
- For Tailwind: prefer compiled CSS over JIT runtime; use `style-src 'self'` once inline styles are extracted (CONCERNS.md flagged this).
- Sanitize all user-controlled HTML rendering: log lines, container names, env values. Use `v-text` not `v-html`.
- Add CSP to `tauri.conf.json` `security.csp` field — don't leave empty.
**Detection:** E2E smoke test on packaged build (not dev) per platform.
**Phase to address:** Phase A baseline + revisit before each release; M5 (Monaco) and M4 (logs) need explicit review.

---

### P-C5: Docker socket permission denied on Linux (group membership)
**What goes wrong:** User installs Orcker but is not in the `docker` group → bollard returns `permission denied`. Even with rootless Docker, `$DOCKER_HOST` may not be exported in the GUI session (only in shell).
**Why it happens:** Linux Docker socket is owned by `root:docker`; GUI apps inherit DBus session env, not shell env. Adding to group requires logout.
**Consequences:** "Doesn't work on Linux" reviews. Most common Linux Docker app friction point.
**Warning signs:** Linux user reports `EACCES`/`permission denied`; works after `sudo orcker` but that's wrong.
**Prevention:**
- On startup, if connection fails with `EACCES`, surface a **clear actionable dialog**: "User not in docker group. Run: `sudo usermod -aG docker $USER` then log out and back in."
- Detect rootless Docker via `$XDG_RUNTIME_DIR/docker.sock` and prefer it (no group needed).
- Document in `docs/setup.md`: rootless Docker recommendation + group setup.
- Never run as root; never suggest `chmod 666` on socket (security disaster).
**Detection:** Specific error matcher in adapter — translate `EACCES` to "permission" error variant.
**Phase to address:** Phase A (M1) + onboarding/first-run UX in M7 Settings.
**Maps to:** R02.

---

### P-C6: Tokio runtime conflict — "there is no reactor running"
**What goes wrong:** Calling `tokio::spawn` or async bollard in a test/binary that doesn't have a Tokio runtime, or nesting `Runtime::new().block_on(...)` inside an existing async context. Symptoms: panic at runtime, or deadlock.
**Why it happens:** Tauri 2 provides a runtime via `tokio::main` or `Builder::run`; tests, integration tests, and `tauri::async_runtime::spawn` vs `tokio::spawn` are subtly different.
**Consequences:** Tests pass locally, fail in CI; sporadic deadlocks; "future cannot be sent between threads" compile errors.
**Warning signs:** `RuntimeError: there is no reactor running`; tests hang; `Send`/`Sync` errors that don't make sense.
**Prevention:**
- Always use `tauri::async_runtime::spawn` (not raw `tokio::spawn`) inside Tauri commands — it ties to Tauri's runtime.
- For unit tests of async adapters, use `#[tokio::test]` not `#[test]`.
- Never `block_on` inside an `async fn`. If you have to bridge sync→async, use `spawn_blocking`.
- Centralize runtime access through `AppHandle::async_runtime()` if needed.
**Detection:** Run tests in CI with `--test-threads=1` first; investigate any thread/runtime panic.
**Phase to address:** Phase A (foundation).
**Maps to:** R01.

---

### P-C7: Streaming docker logs leaks tasks / clogs IPC
**What goes wrong:** Naive log streaming spawns a `tokio::task` per container that writes every line via `app.emit()`. Issues: (a) task is not cancelled when user navigates away → memory leak + zombie API connections; (b) emitting 10k lines/sec floods IPC bridge → UI render thread saturated.
**Why it happens:** Cancellation tokens not threaded through; no backpressure between Rust producer and JS consumer.
**Consequences:** Memory grows over time; UI stalls when verbose logs spike; M4 module becomes unusable for chatty containers.
**Warning signs:** Memory usage climbs after each log session; closing log view doesn't free memory; UI freezes when log spikes.
**Prevention:**
- Use `tokio_util::sync::CancellationToken` per stream; cancel on Vue `onUnmounted`.
- **Throttle/batch emits**: buffer lines for 50–100ms, emit array of lines, not 1 line per event.
- Cap UI buffer at 5000 lines (PRD §11 already says this) with virtualised list (`vue-virtual-scroller`).
- Drop oldest lines on overflow rather than queueing forever.
- Add a "pause stream" button — stop emitting but keep connection (or drop and reconnect on resume).
**Detection:** Stress test: tail a `yes` container output; memory should plateau, UI should remain interactive.
**Phase to address:** M4 Logs.
**Maps to:** R01, performance considerations from CONCERNS.md.

---

### P-C8: Tauri IPC serialization edge cases — silent data corruption
**What goes wrong:** Rust `chrono::DateTime` serializes as RFC3339 string but JS expects Date object → string compared as date silently fails. Rust `u64` larger than `Number.MAX_SAFE_INTEGER` (2^53) → precision lost. `Result<T, E>` where E doesn't impl `Serialize` → command appears to succeed with wrong payload. Binary data (Vec<u8>) defaults to JSON array of numbers (slow + bloated).
**Why it happens:** Tauri uses `serde_json` for IPC; some Rust types don't map cleanly to JS.
**Consequences:** "Works on my machine" bugs; large logs send MB of `[123, 45, 67, ...]` arrays; date filters fail randomly.
**Warning signs:** Numbers off by tiny amounts; dates compared as strings; payloads unexpectedly huge for binary.
**Prevention:**
- Use `String` for IDs, never raw `u64` if value can exceed 2^53.
- Define explicit DTOs (`#[derive(Serialize)] struct ContainerDTO {...}`); never expose bollard's internal types directly.
- Errors: implement `serde::Serialize` on `OrckerError` enum (CONCERNS.md mentions this) with stable string variants.
- For binary streams, use Tauri's [raw IPC](https://v2.tauri.app/concept/inter-process-communication/) or base64 explicitly.
- Add a Zod (or Valibot) schema on the JS side that validates every command response — catches drift fast.
**Detection:** TypeScript types generated from Rust via `ts-rs` or `specta`; compile-time mismatch breaks build.
**Phase to address:** Phase A (foundation) — establish DTO + error pattern before module work.

---

## Moderate Pitfalls

### P-M1: Cross-platform build matrix — universal binary, signing, AppImage decisions
**What goes wrong:**
- macOS: shipping x64-only binary on Apple Silicon → 2x slower via Rosetta + larger memory footprint. Conversely, building universal binary doubles size and CI time.
- macOS notarization: unsigned `.app` triggers Gatekeeper "damaged, can't be opened" — users panic and delete.
- Windows: unsigned `.msi`/`.exe` triggers SmartScreen warning. EV cert is $300+/year; standard cert still requires reputation building.
- Linux: `.AppImage` doesn't auto-update natively, no system integration; `.deb` works on Ubuntu/Debian only; Flatpak/Snap are different ecosystems.
**Prevention:**
- **MVP:** Linux x64 `.deb` + `.AppImage`, macOS `.dmg` (universal via `cargo-tauri`'s `--target universal-apple-darwin`), Windows `.msi` (unsigned with documented "More info → Run anyway" workaround).
- Use `tauri-action` GitHub Action — it handles platform matrix.
- Sign macOS via Apple Developer ID ($99/yr) + notarize via `notarytool` (automated). Add as CI secret early.
- Windows code signing: defer until v0.x → 1.0; document the SmartScreen unblock in README.
- AppImage updater: use `tauri-plugin-updater` with custom endpoint; or document manual update.
**Phase to address:** Phase 0 (CI matrix already partially set up per PROJECT.md) + signing in Phase pre-1.0.

---

### P-M2: Release Please + Tauri version bump out of sync
**What goes wrong:** Release Please bumps `package.json` version on merge but `src-tauri/tauri.conf.json` `version` field and `Cargo.toml` stay at old value. App ships labeled v0.1.0 but window title shows v0.0.0; updater checks fail.
**Why it happens:** Release Please's `extra-files` config needs explicit JSONPath/TOML path entries for each non-default file.
**Consequences:** Version drift; updater can't determine current version; user-visible inconsistency.
**Prevention:**
- Configure `release-please-config.json` `extra-files`:
  ```json
  {
    "extra-files": [
      {"type": "json", "path": "src-tauri/tauri.conf.json", "jsonpath": "$.version"},
      {"type": "toml", "path": "src-tauri/Cargo.toml", "jsonpath": "$.package.version"}
    ]
  }
  ```
- Verify `Cargo.lock` is regenerated after Cargo.toml bump (CI step: `cargo build` before commit).
- Smoke test: `pnpm tauri build` after release-please PR merge; check About dialog shows new version.
**Phase to address:** Phase 0 (already partially set up; verify before first non-zero release).

---

### P-M3: Tauri auto-updater signing keys lost or leaked
**What goes wrong:** `tauri-plugin-updater` requires Ed25519 signing key. Lose the private key → cannot ship updates ever again (users stuck on old version forever, must re-install). Leak the key → anyone can ship malicious updates to all users.
**Prevention:**
- Generate keypair with `tauri signer generate`; store private key in **password manager AND offline backup** (e.g., encrypted USB).
- Public key embedded in `tauri.conf.json` (safe to commit).
- Private key in CI: GitHub Actions secret `TAURI_SIGNING_PRIVATE_KEY` + password secret.
- Rotate plan: documented procedure for key rotation involving forced re-install (acceptable cost is much greater than losing key entirely).
- Never commit `.tauri/` private key files.
**Phase to address:** Phase pre-1.0 when updater goes live (M7 Settings auto-updater).

---

### P-M4: Vue 3 reactivity pitfalls with Tauri events
**What goes wrong:** Subscribing to `listen()` event in `setup()` without unlisten on unmount → memory leak + duplicate handlers after HMR. Mutating reactive state from outside `setup` context → "writable computed must have setter" or silent no-update.
**Prevention:**
- Always store `unlisten` and call in `onUnmounted`:
  ```ts
  const unlisten = await listen('docker:event', handler);
  onUnmounted(() => unlisten());
  ```
- Use Pinia stores for global state shared across components (CONCERNS.md flagged missing Pinia).
- Wrap event handlers that mutate refs with care: ensure they don't fire during teardown.
**Phase to address:** M1 onwards (every module that subscribes to streaming events).

---

### P-M5: docker-compose.yml schema drift (legacy vs v2)
**What goes wrong:** Detection logic checks for `docker-compose.yml` (legacy) but misses `compose.yaml` (modern). Or parses `version: '3'` but breaks on Compose Spec files without version key.
**Prevention:**
- Detection cascade: `compose.yaml` → `compose.yml` → `docker-compose.yaml` → `docker-compose.yml`.
- Use Compose Spec parser (no `version:` key required for modern spec).
- Don't fail on missing `version:`; default to latest.
- For `serde_yaml`, use `serde(default)` heavily; treat as best-effort parse.
**Phase to address:** M2 (Projects) project import.

---

### P-M6: GeoIP database licensing / bundling
**What goes wrong:** M8 Sandbox PRD mentions GeoIP access log. MaxMind's free GeoLite2 requires registration + license key + redistribution rules. Bundling the .mmdb file may violate license; distributing without it means runtime download fails offline.
**Prevention:**
- Use IP2Location LITE (free, redistributable) or DB-IP (CC BY 4.0) instead of MaxMind for bundled DB.
- Or fetch on first use, cache, document offline limitation.
- If MaxMind required: user provides own license key in M7 Settings.
**Phase to address:** M8 — research license before bundling decision.

---

### P-M7: Bundled cloudflared binary stale or compromised
**What goes wrong:** Bundling `cloudflared` binary for sandbox tunnel (PRD R12). Bundled version goes stale (security CVE) or download mirror compromised (supply-chain).
**Prevention:**
- Pin specific cloudflared version + SHA-256 hash in build script.
- Verify hash on every build.
- Set up Dependabot or scheduled CI to alert on new cloudflared releases.
- Auto-update mechanism: app checks for new cloudflared on launch, downloads + verifies hash.
- Document as known limitation: "If bundled cloudflared CVE, ship app update within 7 days."
**Phase to address:** M8 Sandbox.
**Maps to:** R12.

---

### P-M8: Port conflict detection (R06)
**What goes wrong:** User starts Project A on :8080, registers Project B also on :8080. Compose-up succeeds for one, second silently uses different port (Docker assigns random) or fails cryptically.
**Prevention:**
- Pre-flight check: parse compose ports, scan listening sockets (`netstat`/`ss` or Tokio `TcpListener::bind` test), warn before starting.
- Suggest auto-remap to next free port.
- Track port allocations in `~/.orcker/state.json` to detect cross-project conflicts.
**Phase to address:** M2 (project start) + M5 (compose editor).

---

## Minor Pitfalls

### P-m1: pnpm lockfile not committed (CONCERNS.md)
**Fix:** Commit `pnpm-lock.yaml`; remove from `.gitignore`. Required for reproducible builds.
**Phase:** Phase 0 cleanup.

### P-m2: No `.nvmrc` for Node version pinning
**Fix:** Add `.nvmrc` with `20`; CI uses `actions/setup-node@v4` with `node-version-file: .nvmrc`.
**Phase:** Phase 0.

### P-m3: Vue Router not present — bundle bloat as modules grow
**Fix:** Add `vue-router` early; lazy-load module routes via `defineAsyncComponent`.
**Phase:** Phase A before M2.

### P-m4: `tracing` crate not set up — debug logs scattered
**Fix:** Add `tracing` + `tracing-subscriber` in Phase A; structured JSON logs to `~/.orcker/logs/orcker.log`.
**Phase:** Phase A foundation.

### P-m5: No DTO codegen between Rust and TypeScript
**Fix:** Use `specta` or `ts-rs` to auto-generate `.d.ts` from Rust structs; commit generated types.
**Phase:** Phase A foundation.

### P-m6: Tauri capabilities (allowlist) not pre-planned
**Fix:** Define `src-tauri/capabilities/default.json` with minimal scopes BEFORE writing commands. Restrict `fs:scope` to `~/.orcker/`, `shell:scope` to docker CLI only.
**Phase:** Phase A foundation.

### P-m7: Config corruption on update (R08)
**Fix:** Version `~/.orcker/config.json` with `schemaVersion`; on load, run migrations; backup `config.json.bak` before write.
**Phase:** M5 / M7.

### P-m8: External edits to compose files break Orcker state (R09)
**Fix:** File watcher (`notify` crate) on tracked compose files; reconcile state on change; show "modified externally" badge.
**Phase:** M5 Infra.

### P-m9: Healthcheck false negatives (R07)
**Fix:** Don't trust `docker inspect`'s `Health.Status` alone; for critical services, do app-level probe (TCP connect, HTTP /health). Configurable timeouts.
**Phase:** M1 Global Stack + M2 Projects.

### P-m10: Sandbox accidentally exposes prod-like data (R11)
**Fix:** Big red banner "SANDBOX ACTIVE", required password, CIDR allowlist mandatory, hard expiry (no "unlimited"), confirmation modal listing exposed hostname before activation. Already in PRD scope.
**Phase:** M8 Sandbox.

### P-m11: Onboarding for non-Rust contributors (R10)
**Fix:** `docs/development.md` with platform-specific setup; `CONTRIBUTING.md` with module structure walkthrough; `good-first-issue` labels.
**Phase:** Pre-1.0 launch / Phase 2.

### P-m12: Scope creep — solo dev death spiral (R05)
**Fix:** Strict YAGNI; no PRs without GitHub issue; PRD §3 features locked for v1.0; every "wouldn't it be cool" goes to backlog labeled `phase: 3+`. Maintainer (you) commits to refusing.
**Phase:** Continuous.

---

## Phase-Specific Warnings

| Phase / Module | Likely Pitfall | Mitigation Phase |
|----------------|---------------|------------------|
| Phase A foundation | P-C1 (blocking main thread), P-C6 (Tokio runtime), P-C8 (IPC serialization), P-m4/5/6 (tracing, DTOs, capabilities) | Set patterns NOW; retrofit cost is high |
| M1 Global Stack | P-C2 (socket detection), P-C3 (API version), P-C5 (Linux permissions), P-m9 (healthchecks) | Address before any UI polish |
| M2 Projects | P-M5 (compose schema), P-M8 (port conflict), P-m8 (external edits) | Validate import flow against 5 real-world Laravel projects |
| M3 Quick Actions | P-C1 (long-running exec), P-C7 (output streaming) | Reuse log streaming pattern from M4 |
| M4 Logs | P-C7 (stream cleanup + throttling) — **highest risk module for performance** | Stress test mandatory; virtualization required |
| M5 Infra | P-C4 (Monaco CSP), P-m7 (config corruption), P-m8 (file watcher) | Monaco is the trickiest UI piece |
| M6 Database | P-C1 (long dump/restore), P-C7 (progress events) | Same streaming pattern |
| M7 Settings | P-M3 (updater key mgmt), P-M2 (version sync) | Touches every release pipeline |
| M8 Sandbox | P-M6 (GeoIP license), P-M7 (cloudflared bundle), P-m10 (data exposure) | Highest security surface — security review required |
| Release pipeline | P-M1 (signing/notarization), P-M2 (version drift), P-M3 (signing keys) | Lock keys before first signed release |

---

## Risk Matrix Cross-Reference (PRD §12)

| PRD Risk | Pitfalls Addressed | Phase to Mitigate |
|----------|-------------------|-------------------|
| R01 Rust learning curve | P-C1, P-C6, P-C7 | Phase A (foundation patterns) |
| R02 Docker runtime divergence | P-C2, P-C5 | Phase A (M1) |
| R03 Docker API churn | P-C3 | Phase A (adapter) + ongoing |
| R04 (compose performance) | P-C7, P-M5 | M4, M5 |
| R05 Scope creep | P-m12 | Continuous |
| R06 Port conflicts | P-M8 | M2 |
| R07 Healthcheck false negatives | P-m9 | M1, M2 |
| R08 Config corruption | P-m7 | M5, M7 |
| R09 External infra edits | P-m8 | M5 |
| R10 Open-source onboarding | P-m11 | Pre-1.0 |
| R11 Sandbox data leak | P-m10, P-C4 | M8 |
| R12 cloudflared bundle | P-M7 | M8 |

---

## Sources

- **Tauri 2 docs** (HIGH) — https://v2.tauri.app/concept/inter-process-communication/, https://v2.tauri.app/security/csp/, https://v2.tauri.app/reference/config/ — async patterns, CSP, capabilities model.
- **bollard crate docs** (HIGH) — https://docs.rs/bollard/ — `connect_with_local_defaults`, `client_version` negotiation, version pin guidance.
- **Tokio docs** (HIGH) — https://tokio.rs/tokio/topics/bridging — sync↔async bridging, `spawn_blocking`, runtime nesting hazards.
- **Docker Engine API versioning** (HIGH) — https://docs.docker.com/engine/api/v1.43/ — version negotiation guidance.
- **tauri-plugin-updater docs** (HIGH) — Ed25519 signing key lifecycle.
- **release-please docs** (MEDIUM) — `extra-files` JSONPath/TOML configuration for multi-file version bumps.
- **MaxMind GeoLite2 license** (MEDIUM) — distribution restrictions noted in EULA.
- **Compose Spec** (HIGH) — https://compose-spec.io/ — modern compose file detection patterns.
- **PRD §12 + CONCERNS.md** — project-specific risks and audit findings.
- **Training data + ecosystem experience** (MEDIUM) — common Linux Docker socket / group-membership friction; Apple notarization pitfalls; Windows SmartScreen.

**Confidence note:** Critical pitfalls (P-C1 through P-C8) are well-documented Tauri/Docker community knowledge; Moderate (P-M1 through P-M8) are MEDIUM confidence — recommend re-verifying signing/notarization workflow against current Apple + Microsoft policies in 2026 before first signed release.
