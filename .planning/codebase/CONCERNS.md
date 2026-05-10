# Codebase Concerns

**Analysis Date:** 2026-05-10

## Scaffolding Phase — MVP Not Yet Implemented

**Status:** Pre-alpha scaffold only
- Files: `src-tauri/src/lib.rs`, `src-tauri/src/main.rs`, `src/App.vue`, `src/main.ts`
- Impact: All concerns below reflect the current empty-scaffold state. No production logic exists yet.
- Next phase: Architecture implementation per PRD §7.2 (adapters → core → commands → UI layer)

---

## Tech Debt

**Generic Tauri boilerplate starter code:**
- Issue: Current codebase is a vanilla Tauri 2 + Vue 3 starter template with placeholder `greet()` command
- Files: `src-tauri/src/lib.rs` (15 lines), `src/App.vue` (160 lines of demo UI)
- Impact: No actual business logic exists; refactoring will be a full rewrite from scratch during Phase A (Host Setup module)
- Fix approach: Follow PRD §7.2 architecture strictly when implementing M1-M8 modules

**Minimal Rust/Tauri surface:**
- Issue: Only `tauri-plugin-opener` is integrated; no Docker, system inspection, or IPC handlers yet
- Files: `src-tauri/Cargo.toml` (25 lines, 4 dependencies)
- Impact: Full command handler layer (`commands/`) and adapter layer (`adapters/`) must be built from scratch
- Fix approach: When implementing Docker integration, use architecture pattern: `adapters/docker.rs` → `core/docker/service.rs` → `commands/docker_*.rs` (see PRD §7.2)

---

## Missing Critical Features

**No Docker integration:**
- Problem: Orcker's core value proposition — Docker service management — is not implemented
- Files: None (feature gap)
- Blocks: Cannot manage services, detect docker-compose.yml, list containers, or control infrastructure
- Timeline: M1 (Global Stack) is Phase A blocker per PRD §11.1

**No persistence layer:**
- Problem: No configuration storage, no database, no state management across app restarts
- Files: None
- Blocks: Cannot save user preferences, service states, or project metadata
- Timeline: M5 (Infrastructure) requires SQLite or JSON-based config store

**No IPC command routing:**
- Problem: Tauri invoke handlers are not structured; only placeholder `greet` command exists
- Files: `src-tauri/src/lib.rs` (line 11: single handler)
- Blocks: All Rust↔Vue communication must be built
- Fix approach: Implement command factory pattern in `src-tauri/src/commands/mod.rs` with handlers for: `docker_status`, `service_start`, `service_stop`, `project_list`, etc.

**No error handling strategy:**
- Problem: No custom error types, no Result wrapping, no error serialization to frontend
- Files: All Rust code (none yet)
- Impact: Frontend cannot distinguish between network errors, Docker errors, permission errors
- Fix approach: Define `Result<T, OrckerError>` enum in `src-tauri/src/core/error.rs` with serializable variants

---

## Security Considerations

**Unsafe CSP in tauri.conf.json:**
- Risk: `style-src 'unsafe-inline'` allows arbitrary CSS injection; used for inline styles in App.vue
- Files: `src-tauri/tauri.conf.json` (line 23)
- Current mitigation: Hardcoded styles in component, no user input in styles yet
- Recommendations:
  - Extract all inline styles to external CSS files or scoped `<style>` blocks (already done for some, but line 116 uses `box-shadow` inline)
  - Change CSP to `style-src 'self'` once external stylesheets are only source
  - Remove `'unsafe-inline'` before production release

**No input validation in greet command:**
- Risk: `greet(name: &str)` accepts unbounded user input; no length checks or sanitization
- Files: `src-tauri/src/lib.rs` (line 3)
- Current mitigation: Only used in demo, no backend operations
- Recommendations:
  - Add validation layer: `src-tauri/src/core/validation.rs` with max-length, character-class checks
  - Implement for all public commands: validate project paths, service names, port numbers
  - Use newtype pattern: `struct ProjectName(String)` with constructor validation

**Environment variables not documented:**
- Risk: No .env.example or .env.schema; secrets could be committed if added
- Files: None (config gap)
- Current mitigation: Project has no secrets yet
- Recommendations:
  - Create `.env.example` with required vars (Docker socket path, app data dir, dev mode flag)
  - Update `.gitignore` to exclude `.env*` (already done in typical Rust projects)
  - Document in `docs/setup.md` which env vars are required vs. optional

**Tauri allowlist not restricted:**
- Risk: `tauri.conf.json` has empty `security.csp`; no permission model defined for future API calls
- Files: `src-tauri/tauri.conf.json` (no `allowlist` field)
- Impact: When implementing M3 (Terminal) and M5 (Infra), Tauri scope rules must be enforced
- Recommendations:
  - Pre-plan allowlist scope in `src-tauri/capabilities/default.json` before implementing commands
  - Restrict file access to `~/.orcker/` only (projects config directory)
  - Restrict process spawning to Docker CLI and safe shell scripts only

---

## Performance Bottlenecks

**Single-threaded Vue app (no service workers):**
- Problem: All Docker commands will block UI until Rust handler completes (no async wrapping)
- Files: `src/main.ts`, `src/App.vue` (line 10: `await invoke()`)
- Cause: Standard Tauri `invoke()` is single-threaded; long operations (docker-compose up, log tail) will freeze UI
- Improvement path:
  - Implement background command layer: `src-tauri/src/core/background.rs` with `tokio::task::spawn()`
  - Use Tauri event emitter for streaming: `tauri::emit()` for log lines, progress updates
  - Example: instead of `invoke("docker_logs")`, emit `"docker:log:line"` event every N lines
  - Test with large log streams (M4 module)

**No caching of Docker state:**
- Problem: Every UI render may query Docker API (when implemented), causing latency
- Files: Not yet relevant
- Cause: No in-memory cache; no invalidation strategy
- Improvement path:
  - Implement `src-tauri/src/core/cache.rs` with TTL-based memoization
  - Cache `docker ps`, `docker images` for 5-10 seconds; invalidate on user action
  - Use `std::sync::Arc<Mutex<HashMap>>` or `dashmap` crate for thread-safe caching

---

## Test Coverage Gaps

**Zero unit tests for Rust:**
- What's not tested: No test functions in `src-tauri/src/*.rs`
- Files: `src-tauri/src/lib.rs`, `src-tauri/src/main.rs`
- Risk: Docker adapter layer, service state machine, error handling all untested in production
- Priority: **High** — M1, M2, M5 involve critical infrastructure operations; test-before-code required
- Action: Implement test module in each Rust file:
  ```rust
  #[cfg(test)]
  mod tests {
    use super::*;
    #[test]
    fn test_greet_with_empty_name() { ... }
  }
  ```

**Zero integration tests for Tauri IPC:**
- What's not tested: Vue↔Rust command invocation, error propagation, event emission
- Files: `src/App.vue` (invoke calls not validated)
- Risk: Breaking changes in command signatures won't be caught until runtime
- Priority: **High** — use `@tauri-apps/api` test utilities to mock invoke
- Action: Create `src/__tests__/App.spec.ts` with `vi.mock()` for Tauri API

**Zero E2E tests:**
- What's not tested: Desktop app startup, UI interactions, file operations
- Files: None (no test infrastructure)
- Risk: UI regression on macOS/Windows platforms won't be detected
- Priority: **Medium** — defer until M2, then use Webdriver or Tauri CLI test mode
- Recommendation: Set up `cypress` or `playwright` with Tauri WebDriver

**Coverage not tracked:**
- Current: `pnpm test` runs vitest with `--passWithNoTests` (passes with zero tests)
- Files: `package.json` (line 14)
- Impact: No CI safeguard against untested code merging
- Fix: Enable coverage enforcement in CI when tests exist
  ```yaml
  - run: pnpm test:coverage
    env:
      COVERAGE_THRESHOLD: 70
  ```

---

## Fragile Areas

**App.vue is monolithic demo component:**
- Files: `src/App.vue` (160 lines, single component)
- Why fragile: Mixing template, logic, and styles; no separation of concerns; will become unmaintainable when M1-M8 UI is added
- Safe modification: Do NOT add features to this file — it's a temporary scaffold. Create feature components in `src/components/` per feature module (M1/ M2/ M3/ etc.)
- Test coverage: None

**Tauri command handler in lib.rs is inline:**
- Files: `src-tauri/src/lib.rs` (lines 2-11)
- Why fragile: Single file for all command registration; adding M1-M8 will make this unmaintainable (50+ commands)
- Safe modification: Refactor to module system before Phase A completion:
  ```
  src-tauri/src/
    commands/
      mod.rs       (handler registration)
      docker.rs    (docker commands)
      project.rs   (project commands)
      ...
  ```
- Test coverage: None

---

## Scaling Limits

**No lazy loading in Vue:**
- Current capacity: App.vue loads all Tauri plugins upfront; no route-based splitting
- Limit: As M1-M8 modules add 20+ screens, initial bundle will grow to 500KB+
- Scaling path:
  - Add Vue Router: `npm install vue-router`
  - Create `src/router/index.ts` with lazy-loaded routes per module
  - Implement dynamic imports: `const M1 = defineAsyncComponent(() => import('./modules/M1.vue'))`

**Single Rust binary for all logic:**
- Current capacity: All 8 modules' commands in one `main.rs` executable
- Limit: Binary size and startup time scale linearly with feature count; ~50MB+ when all modules implemented
- Scaling path: Consider modular architecture (low priority for MVP):
  - Each module as separate plugin system (Tauri plugins)
  - Load plugins on demand based on user configuration
  - Deferred to Phase 2; MVP uses monolithic approach per PRD §11.1

---

## Dependencies at Risk

**Tauri 2.0 — young framework:**
- Risk: Tauri 2 was released 2024; fewer ecosystem plugins than Electron or native frameworks
- Impact: Custom Docker integration may require FFI or raw shell spawning instead of battle-tested libraries
- Migration plan: If Tauri becomes unmaintainable, fallback is rewrite in PyQt (Python + Qt) or Electron (Node.js), but this is 2-3 month effort

**pnpm lockfile not committed:**
- Risk: `pnpm-lock.yaml` absent; `pnpm install` may resolve different versions on different machines
- Files: `.gitignore` likely excludes lockfile
- Current mitigation: CI uses `pnpm install` which regenerates lockfile
- Recommendation: **Commit `pnpm-lock.yaml` to git** for reproducible builds; add to `.gitignore` exceptions or remove from .gitignore

**No pinned Node.js version in CI:**
- Risk: `.nvmrc` absent; CI uses Node 20, but local dev may use Node 18 or 22, causing version mismatches
- Files: `.github/workflows/ci.yml` (line 23: `node-version: 20`)
- Current mitigation: Nothing
- Fix: Create `.nvmrc` with `20.0.0` and update CI to use `nvm use`

**Minimal Cargo dependencies:**
- Current: 4 dependencies (tauri, tauri-plugin-opener, serde, serde_json)
- Risk: As Docker integration is added, will depend on `tokio` (async), `docker-cli` or `docker-api` crate (Docker), `regex`, `serde_yaml` (docker-compose parsing)
- Mitigation: Vet new deps before adding; prefer standard-library solutions where possible

---

## Known Bugs

**No bugs tracked yet** — project is a clean scaffold with no implementation.

---

## Architecture Risks

**No module separation planned before Phase A:**
- Issue: `src/` and `src-tauri/src/` have no subdirectories for M1-M8; risk of monolithic mess
- Files: All source directories
- Impact: When 10+ developers contribute, merge conflicts and code tangling will slow delivery
- Mitigation: Pre-create module structure in Phase A Task 1:
  ```
  src/modules/
    m1-global/   (Global Stack UI components)
    m2-projects/ (Projects management)
    m3-terminal/ (Terminal emulator)
    ...
  src-tauri/src/
    commands/
      docker.rs
      project.rs
      ...
    core/
      models/   (Docker, Project, Service types)
      adapters/ (Docker adapter interface)
  ```

**Tauri build not optimized for distribution:**
- Issue: `tauri.conf.json` sets `"targets": "all"` in bundle section (line 28), building for all platforms on every release
- Files: `src-tauri/tauri.conf.json` (line 28)
- Impact: Release build takes 30+ minutes; GitHub Actions may timeout on slower hardware
- Fix approach: In `.github/workflows/build.yml`, use matrix strategy (already done, lines 14-20) but ensure local `pnpm tauri build` only builds current platform:
  ```json
  "targets": ["deb", "app"]  // Linux only for local dev
  ```

---

## Missing Infrastructure

**No logging framework:**
- Problem: No centralized logging; `println!` and `console.log` scattered once impl starts
- Files: None
- Blocks: M4 (Logs) cannot work without structured logging
- Recommendation: Add `tracing` crate to Rust, `pino` to Node.js; implement before M4 phase

**No state management for Vue:**
- Problem: No Pinia store; App.vue uses bare `ref()` which doesn't scale
- Files: `src/App.vue` (lines 5-6)
- Blocks: M1 (Global Stack), M2 (Projects) require shared state
- Recommendation: Implement Pinia store in `src/stores/` before Phase A completion

**No build documentation:**
- Problem: No docs on building from source, platform-specific setup, or troubleshooting
- Files: None
- Blocks: Community contributions in Phase 2+
- Recommendation: Create `docs/development.md` with:
  - System requirements per platform (Ubuntu 22.04, macOS 12+, Windows 10+)
  - Rust toolchain setup (rustup)
  - pnpm setup
  - Local build command
  - Troubleshooting common errors

---

*Concerns audit: 2026-05-10*
