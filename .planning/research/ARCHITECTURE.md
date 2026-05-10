# Architecture Patterns — Tauri 2 + Rust + Vue 3 Docker Manager

**Domain:** Cross-platform desktop app for Docker infrastructure management (Laravel/TALL focus)
**Researched:** 2026-05-10
**Confidence:** HIGH (PRD §7 + Tauri 2 official patterns) · MEDIUM (bollard streaming patterns from training data; recommend Context7 verification before Phase A)

---

## 1. Recommended Architecture

The PRD §7.2 already mandates a four-layer architecture. This research validates and expands it into concrete component boundaries.

```
┌─────────────────────────────────────────────────────────────────┐
│  UI LAYER  —  Vue 3 + TypeScript + Pinia                        │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────────┐    │
│  │ Views    │  │ Stores   │  │ Composab.│  │ IPC Wrappers │    │
│  │ (M1..M8) │  │ (Pinia)  │  │ (use*)   │  │ (typed)      │    │
│  └──────────┘  └──────────┘  └──────────┘  └──────┬───────┘    │
│                                                    │            │
└────────────────────────────────────────────────────┼────────────┘
                Tauri IPC: invoke() / listen()       │
┌────────────────────────────────────────────────────▼────────────┐
│  COMMANDS LAYER  —  src-tauri/src/commands/*                    │
│  Thin async fns annotated #[tauri::command]                     │
│  Validate inputs → delegate to core → return Result<T, AppError>│
└────────────────────────────────────────────────────┬────────────┘
                                                     │
┌────────────────────────────────────────────────────▼────────────┐
│  CORE LAYER  —  src-tauri/src/core/*                            │
│  Business logic: project lifecycle, stack orchestration,        │
│  config diffing, sandbox session management                     │
│  Holds: AppState (tauri::State), domain types, error enums      │
└────────────────────────────────────────────────────┬────────────┘
                                                     │
┌────────────────────────────────────────────────────▼────────────┐
│  ADAPTERS LAYER  —  src-tauri/src/adapters/*                    │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌────────┐│
│  │ Docker   │ │ FS       │ │ Git      │ │ Shell    │ │ Keychn ││
│  │ (bollard)│ │(tokio fs)│ │  (git2)  │ │ (tokio)  │ │(keyring││
│  └────┬─────┘ └────┬─────┘ └────┬─────┘ └────┬─────┘ └───┬────┘│
└───────┼────────────┼────────────┼────────────┼───────────┼──────┘
        ▼            ▼            ▼            ▼           ▼
   Docker API   Filesystem    Git repo    docker exec  OS keychain
```

### Component Boundaries (who talks to whom)

| Component | Talks to | Never talks to |
|-----------|----------|----------------|
| Vue components | Pinia stores, IPC wrappers | Rust commands directly (always via wrapper), Tauri APIs raw |
| Pinia stores | IPC wrappers, other stores | Vue components directly (push via reactivity) |
| IPC wrappers (`src/ipc/*.ts`) | `invoke()`, `listen()` | Adapters, Docker socket |
| Tauri commands | Core services, AppState | Adapters directly (forces a service layer) |
| Core services | Adapters, AppState | Tauri AppHandle (except via injected emitter trait) |
| Adapters | External world (Docker, FS, Git, OS) | Other adapters (no cross-adapter calls) |

**Rule of thumb:** Each layer only depends on the layer immediately below. Adapters never reach upward; UI never reaches downward past commands.

---

## 2. Tauri 2 Command/Event Patterns

### 2.1 Async commands as the default

For an I/O-heavy app (Docker socket, filesystem, Git), every command should be `async fn` with `tokio` runtime. Sync commands block the main thread and freeze the webview.

```rust
// src-tauri/src/commands/containers.rs
#[tauri::command]
pub async fn list_containers(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<ContainerSummary>, AppError> {
    state.docker.list_containers().await
}
```

**Pattern:** Commands are <20 lines. They validate, delegate, and translate errors. Business logic lives in `core/`.

### 2.2 Error propagation — single AppError enum

Define one error type using `thiserror`, derive `serde::Serialize`, return as `Result<T, AppError>`. The frontend receives the `Err` payload as a typed JSON object via Promise rejection.

```rust
// src-tauri/src/core/error.rs
#[derive(Debug, thiserror::Error, serde::Serialize)]
#[serde(tag = "kind", content = "message")]
pub enum AppError {
    #[error("docker engine unreachable: {0}")]
    DockerUnreachable(String),
    #[error("project not found: {0}")]
    ProjectNotFound(String),
    #[error("port {0} already in use")]
    PortConflict(u16),
    #[error("io error: {0}")]
    Io(String),
    // ...
}
```

The `#[serde(tag = "kind")]` discriminator gives the TypeScript side a tagged union to switch on.

### 2.3 Events for streaming (logs, stats, sandbox access)

Use `app.emit()` (global) or `app.emit_to()` (window-targeted) for one-way streams from Rust to UI. Vue listens via `listen("event_name", handler)` from `@tauri-apps/api/event`.

**Topic naming convention:**
- `docker://container/{id}/log` — log lines
- `docker://stats` — aggregated stats tick (every 2s per RNF-01)
- `project://{id}/status` — service status changes
- `sandbox://{session_id}/access` — incoming request to sandbox

### 2.4 Channels (Tauri 2 `Channel<T>` API) — preferred for per-invocation streams

Tauri 2 introduced `tauri::ipc::Channel<T>` — superior to global events when a stream is scoped to a single command call (e.g., "tail logs of container X"). The frontend passes a callback channel as part of the invoke arguments; only that caller receives messages.

```rust
#[tauri::command]
pub async fn tail_container_logs(
    container_id: String,
    on_line: tauri::ipc::Channel<LogLine>,
    state: tauri::State<'_, AppState>,
) -> Result<(), AppError> {
    let mut stream = state.docker.logs(&container_id).await?;
    while let Some(line) = stream.next().await {
        on_line.send(line?)?;
    }
    Ok(())
}
```

**Use channels for:** log tailing, build output, sandbox per-session access log, stats per-container.
**Use global events for:** docker daemon connect/disconnect, app-wide notifications.

---

## 3. Rust Module Organization

```
src-tauri/src/
├── main.rs                 # Entry — calls lib::run()
├── lib.rs                  # Tauri builder, plugin setup, command registry
├── commands/               # #[tauri::command] handlers (thin)
│   ├── mod.rs
│   ├── docker.rs           # list/start/stop/inspect containers, images, volumes
│   ├── projects.rs         # CRUD projects, scaffold, import
│   ├── stack.rs            # M1 global stack toggle
│   ├── compose.rs          # M5 docker-compose.yml read/write/validate
│   ├── logs.rs             # M4 log tailing (channel-based)
│   ├── exec.rs             # M3 docker exec, terminal sessions
│   ├── sandbox.rs          # M8 tunnel lifecycle
│   └── settings.rs         # M7 preferences, theme, socket path
├── core/
│   ├── mod.rs
│   ├── state.rs            # AppState struct (held in tauri::State)
│   ├── error.rs            # AppError enum + thiserror
│   ├── domain/             # Pure data types (Project, Service, ...)
│   │   ├── project.rs
│   │   ├── service.rs
│   │   └── sandbox.rs
│   └── services/           # Business logic (use cases)
│       ├── project_service.rs
│       ├── stack_orchestrator.rs
│       ├── config_sync.rs   # diff between disk and managed state (R09)
│       └── sandbox_service.rs
├── adapters/
│   ├── mod.rs
│   ├── docker/             # bollard wrapper
│   │   ├── client.rs       # connection, version negotiation
│   │   ├── containers.rs
│   │   ├── images.rs
│   │   ├── volumes.rs
│   │   ├── networks.rs
│   │   ├── exec.rs
│   │   └── streams.rs      # log/stats/event stream helpers
│   ├── filesystem.rs       # tokio::fs wrappers, atomic writes
│   ├── git.rs              # git2 wrappers (commit, diff, log)
│   ├── shell.rs            # tokio::process::Command + cancellation
│   └── keychain.rs         # keyring-rs wrapper
└── ipc/
    └── events.rs           # Event topic constants + payload types
```

**Why split commands/core/adapters?**
- **Testability:** Core services accept adapter traits → mockable. Commands are too thin to test.
- **Compile times:** Adapters change less often than commands. Splitting reduces incremental rebuild surface.
- **Learn curve:** Solo dev can master one folder at a time (R01 mitigation).

---

## 4. bollard — Docker Client Patterns

### 4.1 Connection management — single Arc<Docker>

`bollard::Docker` is cheap to clone (internally `Arc`-wrapped) and intended to be created once and shared. Hold it in `AppState` and clone the handle into spawned tasks.

```rust
pub struct AppState {
    pub docker: Arc<DockerAdapter>,           // wraps bollard::Docker
    pub projects: Arc<RwLock<ProjectRegistry>>,
    pub sandboxes: Arc<RwLock<SandboxRegistry>>,
    pub config: Arc<RwLock<AppConfig>>,
    pub emitter: tauri::AppHandle,            // for global events
}
```

### 4.2 Connect strategy

Per RNF-04 (graceful degradation when Docker is down), connection is non-fatal:

```rust
pub fn connect() -> Result<Docker, BollardError> {
    // Auto-detect: DOCKER_HOST env, then platform default socket
    // macOS: also try ~/.colima/default/docker.sock, ~/.orbstack/run/docker.sock
    Docker::connect_with_local_defaults()
}
```

UI must subscribe to a `docker://daemon` event for connect/disconnect transitions and render a degraded mode banner (RNF-04).

### 4.3 Stream handling — three flavors

bollard returns `impl Stream<Item = Result<T, Error>>` for everything streaming. Three patterns matter:

**(a) Logs** — tail via `Docker::logs()` with `LogsOptions { follow: true, tail: "100", ... }`. Pipe into a `Channel<LogLine>`. Cancel by dropping the channel (the spawned task observes `channel.send()` failure and exits).

**(b) Stats** — `Docker::stats()` per container yields `Stats` every ~1s. For the M4 dashboard, fan-in: spawn one task per active container into a `tokio::sync::mpsc`, aggregate, emit `docker://stats` every 2s (RNF-01 polling cadence).

**(c) Events** — `Docker::events()` is the global daemon event firehose (`container start`, `container die`, `network create`). Subscribe once at app start; route filtered events into the relevant store via global emit.

### 4.4 Buffering and backpressure (R04 mitigation)

Logs at high volume can saturate IPC. Apply at the Rust side:
- Cap channel buffer (e.g., `tokio::sync::mpsc::channel(512)`)
- Coalesce: batch up to N lines or flush every 100ms, whichever first
- Drop-oldest on overflow with a "[truncated]" marker line
- Frontend: xterm.js render throttling + 5000-line ring buffer (per RF-M4 + R04)

### 4.5 Error recovery

bollard errors map to `AppError` variants. Common transient cases worth retrying once with backoff:
- `BollardError::HyperError` during `events()` stream (daemon restart)
- `BollardError::DockerStreamError` mid-log-tail

Fatal cases (don't retry, surface to UI):
- `BollardError::DockerResponseServerError { status: 404 }` (container deleted)
- Connection refused (degraded mode)

---

## 5. State Management

### 5.1 The hierarchy

| Concern | Where | Why |
|---------|-------|-----|
| Persistent config (projects, services, settings) | `tauri-plugin-store` JSON files + `Arc<RwLock<AppConfig>>` in memory | Survives restart; fast reads from RAM |
| Live container state | `Arc<RwLock<ContainerCache>>` rebuilt from Docker events | Source of truth is Docker; cache for UI fast queries |
| Secrets | `keyring-rs` only — never in AppState memory beyond the call that uses them | RNF-03 |
| Active sandbox sessions | `Arc<RwLock<SandboxRegistry>>` + child process handles | Must terminate on app close (RNF-03 §M8) |
| UI ephemeral state | Pinia (frontend only) | No round-trip needed |

### 5.2 Mutex vs RwLock vs DashMap — decision matrix

| Use case | Type | Why |
|----------|------|-----|
| Read-heavy (project list shown on every nav) | `tokio::sync::RwLock` | Many concurrent readers, infrequent writes |
| Short critical section, mostly write (config update) | `tokio::sync::Mutex` | Simpler, no reader-writer overhead |
| Per-container concurrent updates from event stream | `dashmap::DashMap<ContainerId, ContainerState>` | Lock-free sharded map, ideal for high-frequency updates |
| Single-writer queue (command history append) | `tokio::sync::Mutex<Vec<_>>` | Append-only, no contention |

**Always `tokio::sync::*` not `std::sync::*` inside async commands** — `std::sync::Mutex` held across `.await` deadlocks the runtime.

### 5.3 Arc rules

- Wrap shared adapters in `Arc<T>` once, in `AppState`. Clone the `Arc` (cheap) into spawned tasks.
- Inside `tauri::State<AppState>`, `AppState` itself doesn't need `Arc` (Tauri manages lifetime), but its fields do if tasks outlive the command.

### 5.4 Spawned tasks must be cancellable

Every long-running task (log tail, stats, sandbox tunnel) gets a `tokio_util::sync::CancellationToken` stored alongside the session in the registry. On stop, cancel the token; the task's `select!` arm exits cleanly.

---

## 6. Frontend ↔ Backend Data Contract

### 6.1 Single source of truth: Rust serde types

Domain types are defined once in `src-tauri/src/core/domain/` with `#[derive(Serialize, Deserialize)]`. The frontend mirrors them as TypeScript.

### 6.2 Type generation — pick one

**Option A: `tauri-specta` + `specta` (recommended)**
- Annotate commands with `#[specta::specta]`, run a build-time generator that emits `bindings.ts` with typed `invoke` wrappers and event payload types
- Pros: zero drift, full type-safety end-to-end, idiomatic for Tauri 2
- Cons: extra crate, adds compile step
- Status (training data): Active maintenance through 2025; verify current version on crates.io / Context7 before adopting

**Option B: `ts-rs` (alternative)**
- Annotate types with `#[derive(TS)]`, generate `.ts` files at test-time
- Pros: simpler, type-only (no runtime invoke wrappers)
- Cons: you still write `invoke<T>()` calls by hand → drift surface

**Option C: hand-maintained `src/types/*.ts` mirrors (NOT recommended)**
- Solo dev + 39 RFs = guaranteed drift. Avoid.

**Recommendation:** `tauri-specta`. The boilerplate it eliminates pays back within Phase 1.

### 6.3 IPC wrapper layer (frontend side)

Don't sprinkle `invoke()` calls across components. Create per-domain wrappers:

```typescript
// src/ipc/containers.ts
import { invoke } from '@tauri-apps/api/core';
import type { ContainerSummary, AppError } from './bindings';

export async function listContainers(): Promise<ContainerSummary[]> {
  return invoke('list_containers');
}

export async function startContainer(id: string): Promise<void> {
  return invoke('start_container', { id });
}
```

Components/stores import these wrappers — never `invoke` directly. This gives one place to add error handling, logging, and (later) optimistic updates.

### 6.4 Error handling on the TS side

Tauri serializes `Err(AppError)` as a JSON object that becomes a Promise rejection. Wrap in a result helper:

```typescript
export async function safe<T>(p: Promise<T>): Promise<Result<T, AppError>> {
  try { return { ok: true, value: await p }; }
  catch (e) { return { ok: false, error: e as AppError }; }
}
```

---

## 7. Real-time Docker Event Streaming to Vue

### 7.1 Two streams, two patterns

| Stream | Mechanism | Lifetime |
|--------|-----------|----------|
| Daemon-wide events (start/stop/die) | Single global `docker://event` emit on app start | App lifetime |
| Per-container logs / stats | `Channel<T>` per invoke call | Component mount → unmount |

### 7.2 Pinia store as the integration point

```typescript
// src/stores/containers.ts
export const useContainersStore = defineStore('containers', () => {
  const map = ref<Map<string, ContainerSummary>>(new Map());

  async function init() {
    // 1. Initial snapshot
    for (const c of await listContainers()) map.value.set(c.id, c);
    // 2. Subscribe to event stream
    await listen<DockerEvent>('docker://event', (e) => {
      applyDockerEvent(map.value, e.payload);
    });
  }
  return { map, init };
});
```

The store is initialized once on app mount. Components reactively read `map.value`; updates propagate via Vue reactivity.

### 7.3 Component-scoped streams

For a `LogViewer.vue` watching one container, set up the channel in `onMounted`, tear down in `onUnmounted`:

```typescript
const channel = new Channel<LogLine>();
channel.onmessage = (line) => buffer.value.push(line);
await invoke('tail_container_logs', { containerId: props.id, onLine: channel });
onUnmounted(() => channel.close()); // cancels Rust task
```

---

## 8. Patterns to Follow

### Pattern 1: Adapter trait + concrete impl
**What:** Define `trait DockerPort` in `core/`, implement in `adapters/docker/`.
**When:** Any adapter that needs mocking for tests.
**Example:** `core/services/stack_orchestrator.rs` depends on `dyn DockerPort`, not on `bollard::Docker` directly. Tests inject a `MockDocker`.

### Pattern 2: Command = validate + delegate + map error
**What:** Tauri command bodies stay thin. Heavy lifting in core services.
**When:** Always.
**Anti-example:** A command that does `bollard` calls inline, parses YAML, writes files, and emits events. Split it.

### Pattern 3: Atomic file writes
**What:** Write to `path.tmp`, fsync, rename to `path`. Never write in place.
**When:** All config persistence — `.orcker.json`, `docker-compose.yml`, `.env`, `php.ini`.
**Why:** Power loss mid-write corrupts user config (R08).

### Pattern 4: Event-sourced container state
**What:** Don't poll `docker ps` on a timer. Subscribe to `docker events` once, mutate cache.
**When:** All container status views.
**Why:** Lower latency, less CPU (RNF-01 < 2% idle).

### Pattern 5: Graceful daemon disconnect
**What:** UI degrades, doesn't crash. Banner + retry button when Docker is down.
**When:** App start, mid-session. RNF-04.

---

## 9. Anti-Patterns to Avoid

### Anti-Pattern 1: Sync commands for I/O
**What:** `#[tauri::command] fn list_containers() -> ...` (no `async`, blocking call inside).
**Why bad:** Blocks the Tauri main thread; UI freezes during Docker socket read.
**Instead:** Always `async fn` for I/O.

### Anti-Pattern 2: Holding `std::sync::Mutex` across `.await`
**What:** `let g = mutex.lock().unwrap(); some_async().await;`
**Why bad:** Future may be polled on another thread; deadlocks or panics.
**Instead:** `tokio::sync::Mutex` and drop guard before `.await` whenever possible.

### Anti-Pattern 3: Shelling out to `docker` CLI for queryable operations
**What:** `Command::new("docker").args(["ps", "--format", "json"])...`
**Why bad:** Loses typed errors, brittle output parsing, slower, requires CLI in PATH.
**Instead:** bollard (PRD §7.4 already mandates this; the only sanctioned exception is `docker compose up/down` orchestration).

### Anti-Pattern 4: Frontend mirrors Rust types by hand
**What:** Maintain `src/types/Project.ts` separately from `core/domain/project.rs`.
**Why bad:** Drift is inevitable; runtime errors when shapes diverge.
**Instead:** `tauri-specta` codegen.

### Anti-Pattern 5: Cross-adapter calls
**What:** `DockerAdapter` calling into `GitAdapter`.
**Why bad:** Couples low-level concerns; defeats layering.
**Instead:** Both are called from a core service that orchestrates.

### Anti-Pattern 6: Unbounded log buffers
**What:** Push every log line into a `Vec` that grows forever.
**Why bad:** Memory blowup on chatty containers (R04).
**Instead:** Ring buffer (5000 lines), drop-oldest, render throttle.

### Anti-Pattern 7: Storing secrets in `tauri-plugin-store`
**What:** ngrok token in `config.json`.
**Why bad:** RNF-03 violation; plain-text on disk.
**Instead:** `keyring-rs` exclusively.

---

## 10. Scalability Considerations

| Concern | At 5 projects | At 15 projects | At 50 projects (future) |
|---------|---------------|----------------|--------------------------|
| Container count | ~15-20 | ~45-60 | ~150-200 |
| Stats stream load | One task per container, fine | Aggregate every 2s, batch emits | Sample subset visible to UI; suspend off-screen |
| Log tail concurrency | Per-component channels | Same | Limit concurrent tails to N (e.g., 8); reuse channels |
| Config file count | ~10 files | ~30 | ~100 — consider lazy load on project open |
| Docker event volume | Negligible | Manageable | Filter at subscription (event_filters by label) |

PRD targets MVP at "≥ 5 projects without conflict" and 6-month at "≥ 15." 50+ is out of scope but the architecture (event-sourced cache, per-container DashMap, channel-scoped streams) scales there without rewrites.

---

## 11. Suggested Build Order (Dependencies Between Components)

This order minimizes rework and lets each phase produce something demonstrable.

**Phase A — Foundation (no UI features yet)**
1. `core/error.rs` — `AppError` enum
2. `adapters/docker/client.rs` — `bollard::Docker` connect, version negotiation, daemon ping command
3. `core/state.rs` — `AppState` skeleton with `docker: Arc<DockerAdapter>`
4. `lib.rs` — register state, register one smoke command (`docker_version`)
5. `tauri-specta` wired up + first generated `bindings.ts`
6. Frontend IPC wrapper layer scaffolded

**Phase B — Container observability (M4 prereq)**
7. `adapters/docker/containers.rs` — list/inspect
8. `adapters/docker/streams.rs` — events stream subscription
9. `core/services/container_cache.rs` — event-sourced cache
10. `commands/docker.rs` — `list_containers`, `start_container`, `stop_container`
11. Pinia `containers` store + global event listener
12. Minimal UI list view (validates the whole stack end-to-end)

**Phase C — Persistent config (M1, M2 prereq)**
13. `adapters/filesystem.rs` — atomic writes
14. `tauri-plugin-store` integration in `core/services/config_store.rs`
15. `core/domain/project.rs` + `service.rs`
16. `commands/projects.rs` CRUD

**Phase D — Stack orchestration (M1)**
17. `adapters/docker/networks.rs` — create/ensure `orcker-global`
18. `core/services/stack_orchestrator.rs`
19. `commands/stack.rs` — toggle ON/OFF

**Phase E — Logs & exec (M3, M4)**
20. `adapters/docker/exec.rs`
21. `commands/exec.rs` + `commands/logs.rs` (Channel-based)
22. xterm.js integration

**Phase F — Compose & infra (M5)**
23. `adapters/git.rs`
24. `core/services/config_sync.rs` (R09 diff algorithm)
25. `commands/compose.rs`

**Phase G — Sandbox (M8 — last, security-critical)**
26. `adapters/keychain.rs`
27. cloudflared bundling + SHA-256 verification
28. `core/services/sandbox_service.rs` with mandatory expiry
29. `commands/sandbox.rs` + access log streaming

**Why this order:**
- Docker connection first → everything else depends on it
- Event stream before features → UI is reactive from day 1
- Persistence before features that need it
- Sandbox last → highest security risk surface, benefits from mature core

---

## 12. Open Questions for Phase-Specific Research

| Topic | When to research | Why deferred |
|-------|------------------|--------------|
| `tauri-specta` current version + breaking changes | Phase A start | Verify against Tauri 2 latest before adopting |
| bollard `Stats` payload shape vs Docker API v1.44 | Phase B | Need exact field names for UI binding |
| Best `docker compose` invocation pattern (CLI shell-out exception per PRD §7.4) | Phase D | PRD permits but doesn't prescribe |
| xterm.js + Tauri 2 channel performance with high log volume | Phase E | R04 risk, needs benchmark |
| cloudflared bundling — code-signing implications per OS | Phase G | Distribution-specific, separate research |
| GeoLite2 license + bundling for sandbox access log | Phase G | License compliance |

---

## Sources

- PRD §7 (Arquitetura de Alto Nível) — `docs/PRD.md` lines 645-722
- PRD §10.2 (Stack Recomendada) — `docs/PRD.md` lines 911-985
- Tauri 2 official docs (training data, HIGH confidence): https://tauri.app — patterns for `tauri::command`, `tauri::State`, `Channel<T>`, event system
- bollard crate (training data, MEDIUM confidence — verify with Context7 before Phase A): https://docs.rs/bollard
- `tauri-specta` (training data, MEDIUM confidence): https://github.com/oscartbeaumont/tauri-specta
- Existing codebase architecture map: `.planning/codebase/ARCHITECTURE.md`

**Confidence summary:**
- HIGH: Layering pattern, command/event split, error enum strategy, build order — anchored in PRD + Tauri 2 well-documented patterns
- MEDIUM: bollard streaming specifics, `tauri-specta` current state — recommend Context7 verification at Phase A
- LOW: None of the architectural recommendations rely on unverified single-source claims
