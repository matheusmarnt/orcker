# Technology Stack — Orcker

**Project:** Orcker (Tauri 2 desktop Docker manager for Laravel devs)
**Researched:** 2026-05-10
**Mode:** Ecosystem
**Overall confidence:** MEDIUM-HIGH (verified via training data through Jan 2026; Context7/WebFetch unavailable in this run — flagged versions should be verified at install time via `cargo search` / `pnpm view`)

> **Note on verification:** Web tools (WebSearch, WebFetch, Context7) were unavailable during this research session. Recommendations below are based on the ecosystem state as of late 2025 / early 2026 plus the existing scaffold (Phase 0 complete: Tauri 2.11, Vue 3.5.13, Vite 6.0.3, pnpm). All version numbers should be re-validated at install with `cargo search <crate>` or `pnpm view <pkg> version`. Confidence levels are assigned per item.

---

## Recommended Stack

### Core Framework

| Technology | Version | Purpose | Why | Confidence |
|------------|---------|---------|-----|------------|
| **Tauri** | 2.11.x | Desktop runtime (Rust + WebView) | Smaller bundles (~3-10 MB) and lower memory than Electron; native Rust backend matches `bollard`'s async model; CSP-first security model; v2 stabilized capabilities/permissions system | HIGH (already installed) |
| **Vue** | 3.5.x | UI framework | Reactivity v2 in 3.5 reduces re-renders ~40%; Composition API + `<script setup>` is idiomatic for Tauri apps; smaller runtime than React for desktop apps | HIGH (already installed) |
| **TypeScript** | 5.6+ | Type safety | Strict mode + generated types from `tauri-specta` (optional) eliminates the `invoke<T>()` cast burden | HIGH |
| **Rust** | 2021 edition, 1.77+ | Backend logic | Tauri 2 requires 1.77+ MSRV; async/await + tokio is mature | HIGH |
| **Vite** | 6.0.x | Frontend build/dev server | Tauri 2 default; HMR works through Tauri dev; rolldown migration not yet stable for production | HIGH |

### Docker Integration (Critical Path)

| Technology | Version | Purpose | Why | Confidence |
|------------|---------|---------|-----|------------|
| **bollard** | 0.17.x or 0.18.x | Rust Docker daemon client | Async-native (tokio); used by Podman Desktop and `docker-compose` Rust ports; supports streaming logs/exec/stats via `futures::Stream`; covers Engine API ≥ 1.43 | HIGH (verify minor at install) |
| **bollard-stubs** | (transitive) | Generated API types | Pulled in automatically; do NOT depend directly | HIGH |
| **tokio** | 1.40+ | Async runtime | Required by bollard; use `rt-multi-thread` + `macros` features; same runtime Tauri 2 uses internally | HIGH |
| **futures-util** | 0.3 | Stream combinators | `StreamExt::next()`, `try_collect()`, `forward()` for piping bollard streams into Tauri channels | HIGH |
| **bytes** | 1.x | Byte buffers | bollard log/exec streams emit `Bytes` chunks; needed in adapter signatures | HIGH |

**bollard usage notes:**
- Use `Docker::connect_with_socket_defaults()` for Linux/macOS; falls back to `unix:///var/run/docker.sock`. For Windows, use `connect_with_named_pipe_defaults()`. Wrap detection behind a single `DockerAdapter::new()` constructor.
- Container exec output: `start_exec()` returns `StartExecResults::Attached { output, .. }` — a `Stream<Item = Result<LogOutput, Error>>` where `LogOutput` distinguishes stdout/stderr. Pipe directly into a Tauri `Channel<T>` (Tauri 2 feature) — do NOT collect to `String` for M3 Quick Actions / M4 Logs.
- Logs: `logs()` with `LogsOptions { follow: true, stdout: true, stderr: true, tail: "5000", .. }` returns the same stream type.
- Compose support: bollard does NOT implement Docker Compose. Either (a) shell out to `docker compose` (PRD M5), or (b) use `docker-compose-types` crate to parse YAML and orchestrate via bollard primitives. **Recommendation: shell out for MVP** (matches PRD scope, less surface area).

### State Management & UI

| Technology | Version | Purpose | Why | Confidence |
|------------|---------|---------|-----|------------|
| **Pinia** | 2.2.x | Frontend state | Official Vue store; tree-shakable; DevTools integration; idiomatic Composition API stores. Use one store per module (M1..M8) | HIGH |
| **Vue Router** | 4.4.x | SPA routing | Required for module navigation; use `createWebHashHistory()` (Tauri WebView quirk with HTML5 history mode on `tauri://` scheme) | HIGH |
| **VueUse** | 11.x | Composables library | `useDark`, `useStorage`, `useEventListener`, `useDebounceFn` — saves writing primitives. Pull individual functions, not the whole package | HIGH |
| **shadcn-vue** | latest (CLI-driven) | Copy-in component library | Components copied into repo (no runtime dep); built on **reka-ui** (radix-vue successor as of mid-2025); full TypeScript; Tailwind-styled but unopinionated | MEDIUM-HIGH |
| **reka-ui** | 1.x | Headless primitives (peer of shadcn-vue) | Successor to radix-vue (renamed/forked mid-2025 — verify); accessible primitives for Dialog, Combobox, Toast, Tabs | MEDIUM (verify name; if unmaintained, fall back to **radix-vue 1.9.x**) |
| **Tailwind CSS** | 3.4.x | Utility CSS | shadcn-vue requires v3 (NOT v4) at time of writing; v4 alpha not stable for shadcn-vue ecosystem yet | HIGH |
| **lucide-vue-next** | 0.460+ | Icon set | shadcn-vue default; tree-shakable per-icon imports | HIGH |
| **class-variance-authority (cva)** | 0.7.x | Variant prop API | Required by shadcn-vue components | HIGH |
| **tailwind-merge** + **clsx** | latest | className utilities | Required by shadcn-vue `cn()` helper | HIGH |

### Terminal & Logs

| Technology | Version | Purpose | Why | Confidence |
|------------|---------|---------|-----|------------|
| **xterm.js (@xterm/xterm)** | 5.5.x | Terminal renderer | Industry standard (VS Code, Hyper); package was renamed from `xterm` → `@xterm/xterm` in 2024 — use new package name | HIGH |
| **@xterm/addon-fit** | 0.10.x | Resize handling | Required for responsive panels | HIGH |
| **@xterm/addon-web-links** | 0.11.x | Clickable URLs in output | UX for logs/artisan output | HIGH |
| **@xterm/addon-search** | 0.15.x | Search-in-terminal | M4 Logs requirement | HIGH |
| **@xterm/addon-serialize** | 0.13.x | Export to text | M4 export feature | MEDIUM |
| (NOT used) `@xterm/addon-attach` | — | WebSocket attach | We bridge via Tauri channels, not WebSocket — skip this addon | HIGH |

**Streaming pattern (M3/M4):** Tauri `ipc::Channel<T>` (added in 2.0) sends typed messages from Rust to a single frontend listener with backpressure. Use it for log/exec streaming — do NOT use `app.emit()` (broadcast, no backpressure). On the frontend, write each chunk into `xterm.write()`. For 5000-line virtual buffer (PRD M4), let xterm.js handle the ring buffer (`scrollback: 5000` option).

### Code Editor (M5)

| Technology | Version | Purpose | Why | Confidence |
|------------|---------|---------|-----|------------|
| **Monaco Editor** | 0.52.x | docker-compose.yml + .env editor | VS Code engine; YAML/dotenv/INI syntax; Tauri 2 + Vite integration via `vite-plugin-monaco-editor` | HIGH |
| **vite-plugin-monaco-editor-esm** | 2.x | Vite worker plugin | Monaco's web workers need plugin help in Vite | MEDIUM-HIGH |
| (Alternative) **CodeMirror 6** | 6.x | Lighter editor | If Monaco bundle (~2 MB) is unacceptable, CM6 is ~300 KB — but worse YAML schema validation. **Default to Monaco**, switch only if bundle complaint emerges | MEDIUM |

### Rust Backend Libraries

| Crate | Version | Purpose | Why | Confidence |
|-------|---------|---------|-----|------------|
| **serde** + **serde_json** | 1.x | Serialization | Tauri command args/returns; `.orcker.json` config | HIGH (installed) |
| **thiserror** | 1.0.x | Library error types | Use in `adapters/` and `core/` for typed error enums (`#[derive(Error)]`); cleaner than anyhow for crossing the Tauri boundary | HIGH |
| **anyhow** | 1.0.x | Application error context | Use in `commands/` (Tauri command bodies) where errors get serialized to frontend; `anyhow::Result<T>` + `.context("...")` | HIGH |
| **tracing** + **tracing-subscriber** | 0.1 / 0.3 | Structured logging | Tauri 2 plays well with tracing; use `tracing::instrument` on adapter methods | HIGH |
| **tauri-plugin-log** | 2.x | Frontend log forwarding | Pipes Vue `console.log` to Rust tracing for unified logs | HIGH |
| **tauri-plugin-store** | 2.x | Persistent KV store | M7 settings (theme, locale, socket path); JSON file backed | HIGH |
| **tauri-plugin-dialog** | 2.x | Native file pickers | Project import "Select folder" | HIGH |
| **tauri-plugin-fs** | 2.x | Sandboxed FS access | `.env` editor, scoped to project paths via capabilities | HIGH |
| **tauri-plugin-shell** | 2.x | Shell-out (locked-down) | `docker compose up`, `git`, `composer` if not exec-in-container; ALLOWLIST commands in `capabilities/` | HIGH |
| **tauri-plugin-updater** | 2.x | Auto-update (M7) | GitHub Releases artifact + Ed25519 signature | HIGH |
| **tauri-plugin-single-instance** | 2.x | Prevent dup launches | Standard for tray apps | HIGH |
| **tauri-plugin-window-state** | 2.x | Persist window geometry | UX nicety | HIGH |
| **keyring** | 3.x | OS keychain integration | M8 sandbox secrets, future API tokens; uses Secret Service / Keychain / Credential Manager | HIGH |
| **git2** | 0.19.x | M5 config versioning | libgit2 bindings; mature | HIGH |
| **tokio-stream** | 0.1.x | Stream utilities | `wrappers::ReceiverStream` for bridging mpsc to Stream | HIGH |
| **uuid** | 1.x | Project IDs | `Uuid::now_v7()` for sortable IDs in `.orcker.json` | HIGH |
| **directories** | 5.x | XDG/AppData paths | Cross-platform config/cache dirs; better than rolling our own | HIGH |
| **which** | 6.x | Detect CLI tools | Find `docker`, `git`, `cloudflared` on PATH | HIGH |
| **sha2** | 0.10.x | M8 cloudflared SHA-256 verify | Bundle integrity check (PRD security requirement) | HIGH |
| **bcrypt** | 0.15.x | M8 sandbox password hash | PRD-mandated | HIGH |
| **portpicker** | 0.1.x | Find free local port | Sandbox tunnel + service ports | MEDIUM-HIGH |

### Frontend Libraries

| Package | Version | Purpose | Why | Confidence |
|---------|---------|---------|-----|------------|
| **@tauri-apps/api** | 2.11+ | JS bridge | `invoke`, `Channel`, `listen`, `getCurrent*` | HIGH (installed) |
| **@tauri-apps/plugin-store** | 2.x | Settings persistence | Pairs with Rust plugin | HIGH |
| **@tauri-apps/plugin-dialog** | 2.x | File pickers | — | HIGH |
| **@tauri-apps/plugin-fs** | 2.x | Sandboxed FS | — | HIGH |
| **@tauri-apps/plugin-shell** | 2.x | Allowlisted shell | — | HIGH |
| **@tauri-apps/plugin-updater** | 2.x | Auto-update | — | HIGH |
| **vue-i18n** | 10.x or 11.x | i18n (PT-BR + EN, M7) | De-facto Vue i18n; Composition API mode | HIGH |
| **@vueuse/core** | 11.x | Composables | — | HIGH |
| **zod** | 3.23+ | Runtime validation | Validate parsed `.orcker.json`, form schemas; pairs with TS types | HIGH |
| **vue-virtual-scroller** OR **@tanstack/vue-virtual** | 2.x / 3.x | Log list virtualization | M4 5000-line buffer — xterm covers terminal view, but a log-record list (filterable) needs virtualization | MEDIUM |
| (Optional) **tauri-specta** | 2.x | Auto-generate TS types from Rust | Eliminates manual type duplication on the IPC boundary; small ergonomic win, not required for MVP | MEDIUM |

### Testing

| Tool | Version | Purpose | Why | Confidence |
|------|---------|---------|-----|------------|
| **Vitest** | 2.1.x | Unit tests (frontend) | Already installed | HIGH |
| **@vue/test-utils** | 2.4.x | Component tests | Already installed | HIGH |
| **@testing-library/vue** | 8.x | User-centric component tests | Better than shallow-mount for shadcn-vue components | MEDIUM-HIGH |
| **happy-dom** OR **jsdom** | latest | DOM env for Vitest | happy-dom is faster; jsdom more compatible with xterm.js (xterm needs Canvas — may need `canvas` mock either way) | MEDIUM |
| **cargo test** + **tokio-test** | builtin / 0.4 | Rust unit tests | Standard | HIGH |
| **mockall** | 0.13.x | Rust trait mocking | Mock `DockerAdapter` trait for `core/` tests without hitting daemon | HIGH |
| **insta** | 1.x | Snapshot tests (Rust) | Compose YAML rendering, error messages | MEDIUM |
| (Defer) **Playwright** / **WebdriverIO** | — | E2E | Tauri 2 + WebdriverIO via `tauri-driver` works on Linux/Windows; macOS unsupported. Not MVP — Phase 2+ | LOW priority |

### Tooling / DX

| Tool | Version | Purpose | Why | Confidence |
|------|---------|---------|-----|------------|
| **pnpm** | 9.x | Package manager | Already installed; fast, strict | HIGH |
| **ESLint** (flat config) | 9.x | Linting | Already installed | HIGH |
| **eslint-plugin-vue** | 9.x | Vue rules | Pairs with flat config | HIGH |
| **Prettier** | 3.x | Formatter | Pair with ESLint via `eslint-config-prettier` | HIGH |
| **vue-tsc** | 2.1.x | Type-check Vue | Already installed | HIGH |
| **clippy** | rustup | Rust linting | `clippy::pedantic` selectively; CI gate | HIGH |
| **rustfmt** | rustup | Rust formatting | CI gate | HIGH |
| **cargo-deny** | 0.16.x | License/CVE audit | Supply-chain hygiene; CI gate | HIGH |
| **cargo-machete** | 0.7.x | Detect unused deps | Periodic cleanup | MEDIUM |
| **release-please-action** | v4 | Versioning | Already configured | HIGH |
| **tauri-action** (GH Actions) | v0 (latest) | Build matrix | Linux/macOS/Windows artifacts | HIGH |

---

## Tauri 2 IPC — Patterns to Standardize

| Pattern | When | Notes |
|---------|------|-------|
| **`#[tauri::command] async fn ... -> Result<T, AppError>`** | Default for all commands | `AppError` is a `thiserror` enum that derives `serde::Serialize` so it crosses the boundary cleanly. Wrap with `.map_err(AppError::from)` from adapter results. |
| **`tauri::State<'_, AppState>`** | Sharing adapters/clients | `AppState { docker: DockerAdapter, projects: Mutex<ProjectStore>, .. }`. Register with `.manage(state)` in `setup`. Prefer `tokio::sync::Mutex` (NOT `std::sync::Mutex`) to keep `await` points safe. |
| **`ipc::Channel<T>`** (Tauri 2 native) | Streaming logs, exec output, build progress | Single-listener, typed, with backpressure. Pass channel as command arg from frontend. **Use this, not `emit`.** |
| **`app.emit()` / `WebviewWindow::emit()`** | Broadcast events (multiple listeners) | Service status changes (M1), project updates. Avoid for high-frequency streams. |
| **`AppHandle`** clone-into-task | Long-running background tasks | `let handle = app.app_handle().clone(); tokio::spawn(async move { ... })`. Always `.clone()` — handles are cheap. |
| **Capabilities (`src-tauri/capabilities/*.json`)** | Permission scoping | One capability file per module; allowlist plugin commands; scope `fs` paths to project roots only. |

**Anti-patterns:**
- ❌ Blocking I/O in `#[tauri::command]` (use `async fn` + tokio everywhere)
- ❌ `std::sync::Mutex` across `.await` (causes Send issues, livelock risk)
- ❌ Returning `String` errors instead of typed enums (frontend can't pattern-match)
- ❌ `emit` for log streams (no backpressure, drops messages under load)
- ❌ Holding a Mutex guard across an IPC boundary (deadlock potential — clone the data out)

---

## Rust Error Handling — Layered Approach

```
adapters/   →  thiserror enums (DockerError, FsError, GitError)
core/       →  thiserror enums per domain (ProjectError, StackError, etc.)
                  with #[from] conversions for adapter errors
commands/   →  AppError (one big enum) + Result<T, AppError>
                  derives Serialize so frontend gets {kind, message, details}
```

**Why this split:**
- `thiserror` gives you typed, exhaustive errors for matching in tests and `core/` logic
- `anyhow` is fine inside command bodies for *intermediate* errors, but the **return type must be a serializable enum** — `anyhow::Error` does not implement `Serialize`
- Frontend gets discriminated unions it can handle: `if (err.kind === 'DockerNotRunning') showInstallPrompt()`

**Recommended `AppError` shape:**

```rust
#[derive(Debug, thiserror::Error, serde::Serialize)]
#[serde(tag = "kind", content = "details")]
pub enum AppError {
    #[error("Docker daemon not reachable: {0}")]
    DockerUnavailable(String),
    #[error("Project not found: {0}")]
    ProjectNotFound(String),
    #[error("Filesystem error: {0}")]
    Fs(String),
    #[error("Internal error: {0}")]
    Internal(String),
}
```

---

## Alternatives Considered

| Category | Recommended | Alternative | Why Not |
|----------|-------------|-------------|---------|
| Desktop framework | Tauri 2 | Electron | 5-10× larger bundles, higher RAM, no native Rust integration |
| Desktop framework | Tauri 2 | Wails (Go) | No Laravel/Docker library advantage over Rust + bollard; smaller community |
| Docker client | bollard | `shiplift` | Unmaintained since 2021 |
| Docker client | bollard | shell-out to `docker` CLI | Latency, parsing fragility, no streaming primitives, hard to test |
| UI primitives | reka-ui (via shadcn-vue) | PrimeVue | Heavier, opinionated styling, less composable |
| UI primitives | reka-ui | Vuetify | Material Design lock-in, hard to theme to match Laravel-dev aesthetic |
| UI primitives | reka-ui | Element Plus | Chinese-first community; English docs lag; heavier |
| State | Pinia | Vuex 4 | Vuex is maintenance-mode; Pinia is the official successor |
| Terminal | xterm.js | hterm | Less maintained, smaller ecosystem |
| Terminal | xterm.js | custom `<pre>` renderer | No ANSI support, no resize, no search — re-implementing xterm |
| Editor | Monaco | CodeMirror 6 | Worse YAML IntelliSense; only choose if Monaco bundle becomes a complaint |
| Editor | Monaco | Ace | Stagnant; Monaco is VS Code's engine — better DX expectations match |
| Streaming IPC | `Channel<T>` | `emit` + `listen` | `emit` is broadcast with no backpressure; loses messages on bursty log output |
| Streaming IPC | `Channel<T>` | Custom WebSocket | Reinvents Tauri's IPC; requires port management |
| Logging | tracing | log + env_logger | tracing has structured fields, spans, async-aware — strictly better for adapters |
| Error (lib) | thiserror | manual `impl Error` | Boilerplate without benefit |
| Error (app) | thiserror enum that derives Serialize | anyhow at the boundary | `anyhow::Error` doesn't `Serialize` — won't cross IPC |
| FS access | tauri-plugin-fs (scoped) | std::fs in commands | Bypasses capabilities — security regression |
| Compose orchestration (MVP) | shell out to `docker compose` | reimplement compose in bollard | Months of work, low value, compose CLI is the spec |

---

## Installation Commands

```bash
# Frontend (run from project root with pnpm)
pnpm add vue-router pinia @vueuse/core vue-i18n@^10 zod
pnpm add @tauri-apps/plugin-store @tauri-apps/plugin-dialog \
        @tauri-apps/plugin-fs @tauri-apps/plugin-shell \
        @tauri-apps/plugin-updater
pnpm add @xterm/xterm @xterm/addon-fit @xterm/addon-web-links \
        @xterm/addon-search @xterm/addon-serialize
pnpm add monaco-editor
pnpm add lucide-vue-next class-variance-authority clsx tailwind-merge

# shadcn-vue init (CLI-driven; copies components into src/components/ui/)
pnpm dlx shadcn-vue@latest init
# Then per-component:
pnpm dlx shadcn-vue@latest add button dialog dropdown-menu input \
        label tabs toast tooltip command select switch

# Dev deps
pnpm add -D tailwindcss@^3 postcss autoprefixer \
        prettier eslint-config-prettier \
        @testing-library/vue happy-dom \
        vite-plugin-monaco-editor-esm
pnpm dlx tailwindcss init -p

# Rust (in src-tauri/, edit Cargo.toml — versions to verify with `cargo search`)
# Add under [dependencies]:
#   bollard = "0.17"
#   tokio = { version = "1", features = ["rt-multi-thread", "macros", "sync", "fs", "process"] }
#   futures-util = "0.3"
#   tokio-stream = "0.1"
#   bytes = "1"
#   thiserror = "1"
#   anyhow = "1"
#   tracing = "0.1"
#   tracing-subscriber = { version = "0.3", features = ["env-filter"] }
#   tauri-plugin-log = "2"
#   tauri-plugin-store = "2"
#   tauri-plugin-dialog = "2"
#   tauri-plugin-fs = "2"
#   tauri-plugin-shell = "2"
#   tauri-plugin-updater = "2"
#   tauri-plugin-single-instance = "2"
#   tauri-plugin-window-state = "2"
#   keyring = "3"
#   git2 = "0.19"
#   uuid = { version = "1", features = ["v7", "serde"] }
#   directories = "5"
#   which = "6"
#   sha2 = "0.10"
#   bcrypt = "0.15"
#   portpicker = "0.1"
# [dev-dependencies]:
#   mockall = "0.13"
#   tokio-test = "0.4"
#   insta = "1"
```

---

## Things to NOT Use (and Why)

| Avoid | Reason |
|-------|--------|
| **Electron** | Stack constraint (PRD §10) — and rightly so for a Docker-heavy app where every MB of RAM matters |
| **Tauri 1.x** | EOL; no migration path back |
| **shiplift** | Abandoned (last release 2021) |
| **xterm** (old npm name) | Use `@xterm/xterm` — old package is unmaintained |
| **radix-vue** (if reka-ui supersedes) | Verify at install time — if reka-ui is the maintained fork, switch; if community split, stay with whichever shadcn-vue's CLI installs |
| **Tailwind v4** | shadcn-vue ecosystem still on v3; v4 alpha breaks utility names — wait for shadcn-vue official v4 support |
| **Vuex** | Pinia is the official successor; no reason to start new code on Vuex |
| **Vue Router HTML5 history mode** | Use hash mode in Tauri — `tauri://` and `asset://` schemes confuse history API |
| **`std::sync::Mutex` in async commands** | Use `tokio::sync::Mutex`; std mutex held across `.await` breaks Send |
| **`anyhow::Error` as command return type** | Doesn't implement Serialize — won't cross the IPC boundary |
| **`emit` for log streams** | No backpressure; use `Channel<T>` |
| **`docker` CLI shell-out for container ops** | Use bollard — async, typed, streamable. Reserve shell-out for `docker compose` (which bollard doesn't cover). |
| **Custom CSS-in-JS** | Tailwind + shadcn-vue is the path; mixing styling systems hurts cohesion |
| **Pinia Persisted State plugin** | Tauri's `plugin-store` is the canonical persistence layer — go through Rust so settings live in the right OS dir |
| **localStorage for any non-trivial state** | Same — use `plugin-store` |
| **Sentry / external telemetry at MVP** | Privacy-sensitive (developer machines, paths, env contents); add only with explicit opt-in much later |

---

## Open Questions / Verify-At-Install

1. **reka-ui vs radix-vue naming** — verify at the moment `shadcn-vue init` runs which peer dep it pulls. (Confidence: MEDIUM on the rename.)
2. **bollard 0.17 vs 0.18** — `cargo search bollard` to confirm latest. API has been stable across recent minors but check `Docker::connect_*` signatures.
3. **Tailwind v3 vs v4 for shadcn-vue** — check shadcn-vue docs at install; default to v3 unless explicit v4 install path is documented.
4. **Tauri 2 `Channel<T>` API stability** — was added during 2.x; confirm it's GA (not behind `unstable` feature) for your installed minor.
5. **vue-i18n v10 vs v11** — both are Composition-API friendly; v11 may have breaking changes in plural/format API. Pin to whichever is current stable.

---

## Sources

- Tauri 2 official docs: https://v2.tauri.app/ (commands, channels, capabilities)
- bollard crate: https://docs.rs/bollard/ + https://crates.io/crates/bollard
- shadcn-vue: https://www.shadcn-vue.com/
- reka-ui: https://reka-ui.com/ (verify availability)
- xterm.js: https://xtermjs.org/
- Pinia: https://pinia.vuejs.org/
- VueUse: https://vueuse.org/
- Tauri plugins index: https://v2.tauri.app/plugin/
- Existing scaffold: `src-tauri/Cargo.toml`, `package.json` (already validated Phase 0)

**Confidence summary:** HIGH for installed/scaffold pieces (Tauri 2.11, Vue 3.5, Vite 6); HIGH for well-established crates (tokio, bollard, thiserror, anyhow, serde, tracing); MEDIUM-HIGH for shadcn-vue ecosystem (rename of radix-vue → reka-ui needs install-time verification); MEDIUM for editor/virtualization choices (Monaco vs CM6, vue-virtual-scroller vs tanstack). Re-verify all version pins via `cargo search` / `pnpm view` at install time.
