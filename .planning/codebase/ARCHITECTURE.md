# Architecture

**Analysis Date:** 2026-05-10

## Pattern Overview

**Overall:** Tauri Desktop Application with Layered Architecture (UI → IPC Bridge → Rust Backend)

**Key Characteristics:**
- Desktop-first cross-platform architecture (Linux, macOS, Windows via Tauri 2)
- Two-tier separation: Vue 3 frontend communicates with Rust backend via IPC
- Modular design with planned 8 functional modules (M1-M8) per PRD
- Pre-MVP phase — scaffolding only; core features not yet implemented

## Layers

**Frontend (Vue 3 + TypeScript):**
- Purpose: Desktop UI for Docker infrastructure management
- Location: `src/`
- Contains: Vue 3 components, TypeScript logic, TailwindCSS styling
- Depends on: `@tauri-apps/api` for IPC, Vue 3, Tauri runtime
- Used by: End users; consumes Rust commands

**Tauri IPC Bridge:**
- Purpose: Message passing between frontend and Rust backend
- Location: Implicit in `@tauri-apps/api/core` invocations
- Contains: `invoke()` calls from Vue, command handlers in Rust
- Depends on: Tauri 2 framework
- Used by: Frontend → Backend bidirectional communication

**Rust Backend (Tauri + Tokio):**
- Purpose: Docker client integration, system operations, command execution
- Location: `src-tauri/src/`
- Contains: Tauri commands, Docker operations, async task handling
- Depends on: `tauri`, `serde`, `serde_json`, planned `bollard` (Docker client)
- Used by: IPC bridge from frontend

## Data Flow

**Command Invocation (Frontend → Backend):**

1. User interacts with Vue component (button click, form submit)
2. Component calls `invoke("command_name", { params })` via Tauri API
3. Rust command handler executes in `lib.rs` or `main.rs`
4. Handler returns `String` or JSON serialized result
5. Frontend receives result and updates UI state

**Example:** `greet` command in current scaffold
```
App.vue:greet() → invoke("greet", {name}) → lib.rs:greet() → "Hello, ...!" → Vue reactive update
```

**Real Usage (Planned):**
```
ProjectList.vue:startService() → invoke("docker_container_start", {project_id}) 
  → Backend calls bollard → container starts 
  → JSON {status, logs} returned 
  → UI updates service state
```

**State Management:**
- Frontend state: Vue reactive refs (implicit for now; Pinia planned for complex state)
- Backend state: None persisted (stateless request-response model)
- Persistence: Via Docker state + configuration files (not yet scoped)

## Key Abstractions

**Tauri Commands:**
- Purpose: Rust functions exposed to frontend via `#[tauri::command]` macro
- Examples: `src-tauri/src/lib.rs:greet()`
- Pattern: Async functions returning serializable types (String, JSON)

**Vue Components:**
- Purpose: Reactive UI elements for each module (M1-M8)
- Examples: `src/App.vue` (scaffold only)
- Pattern: Single-file components with `<script setup>` and TypeScript

**Docker Operations (Planned):**
- Purpose: Wrapper around `bollard` crate for Docker API
- Examples: Container lifecycle, image pull, log streaming
- Pattern: Async Rust functions called via Tauri commands

## Entry Points

**Frontend Entry:**
- Location: `src/main.ts`
- Triggers: Application startup
- Responsibilities: Mount Vue app, initialize Tauri runtime, attach root component

**Backend Entry:**
- Location: `src-tauri/src/main.rs`
- Triggers: Tauri application launch
- Responsibilities: Call `orcker_scaffold_lib::run()` (prevents console window on Windows)

**Library Root:**
- Location: `src-tauri/src/lib.rs`
- Triggers: When `src/main.rs` calls `run()`
- Responsibilities: Initialize Tauri builder, register commands, set plugins, run event loop

## Error Handling

**Strategy:** Minimal error handling in scaffold; will adopt per-module error types in implementation

**Patterns:**
- Rust functions return `Result<T, E>` implicitly (Tauri serializes Err as JSON)
- Frontend catches via Promise rejection in `invoke()` calls
- No global error boundary or middleware yet

**Current State:** Simple `expect()` calls in `lib.rs:run()` — suitable for early phase

## Cross-Cutting Concerns

**Logging:** Not yet implemented; will use structured logging (tracing crate) in Rust, console in Vue

**Validation:** Frontend input validation implicit in Vue templates; Rust validation in command params

**Authentication:** Not in scope for MVP (desktop-only, local machine)

**Permissions:** Tauri capabilities system via `src-tauri/capabilities/default.json` (Docker socket access pending)

---

*Architecture analysis: 2026-05-10*
