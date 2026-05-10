# Codebase Structure

**Analysis Date:** 2026-05-10

## Directory Layout

```
orcker/
├── .github/                    # GitHub workflows and issue templates
│   ├── ISSUE_TEMPLATE/         # Issue templates
│   └── workflows/              # CI/CD pipelines
├── .planning/codebase/         # GSD planning documents (this directory)
├── docs/                       # Product and architecture documentation
│   ├── PRD.md                  # Product Requirements Document
│   └── plans/                  # Implementation roadmap and phase plans
├── public/                     # Static assets served by desktop app
│   ├── tauri.svg
│   └── vite.svg
├── src/                        # Frontend Vue 3 + TypeScript
│   ├── App.vue                 # Root component (scaffold only)
│   ├── main.ts                 # Frontend entry point
│   ├── assets/                 # Images, fonts, etc.
│   └── vite-env.d.ts           # Vite environment types
├── src-tauri/                  # Tauri + Rust backend
│   ├── src/                    # Rust source code
│   │   ├── lib.rs              # Library root (Tauri app builder)
│   │   └── main.rs             # Binary entry point
│   ├── capabilities/           # Tauri permissions/capabilities
│   ├── gen/                    # Generated schemas (Tauri build)
│   ├── icons/                  # App icons (multi-format)
│   ├── Cargo.toml              # Rust dependencies and metadata
│   ├── Cargo.lock              # Rust lockfile
│   ├── build.rs                # Build script (Tauri integration)
│   ├── tauri.conf.json         # Tauri configuration
│   └── target/                 # Build artifacts (not committed)
├── .git/                       # Git repository
├── .claude/                    # Claude Code context (not committed)
├── index.html                  # HTML entry point for Vite
├── eslint.config.js            # ESLint configuration (flat config)
├── vite.config.ts              # Vite build configuration
├── tsconfig.json               # TypeScript configuration
├── tsconfig.node.json          # TypeScript config for Vite/Node
├── package.json                # npm scripts and frontend dependencies
├── pnpm-workspace.yaml         # pnpm workspace config
├── pnpm-lock.yaml              # pnpm lockfile
├── CHANGELOG.md                # Release notes
├── CLAUDE.md                   # Claude Code project instructions
├── CODE_OF_CONDUCT.md          # Community guidelines
├── CONTRIBUTING.md             # Contribution guide
├── LICENSE                     # MIT license
├── README.md                   # Project overview
└── SECURITY.md                 # Security policy
```

## Directory Purposes

**`src/`** - Frontend Vue 3 Application
- Purpose: Desktop UI components and logic for all 8 modules (M1-M8)
- Contains: Vue 3 single-file components (.vue), TypeScript scripts (.ts), styles (scoped CSS/TailwindCSS)
- Key files: `App.vue` (root), `main.ts` (initialization)
- Structure (planned): Will expand to `src/components/`, `src/pages/`, `src/stores/` as modules are built

**`src-tauri/src/`** - Rust Backend
- Purpose: Tauri commands, Docker integration, async operations
- Contains: Rust source files (.rs), command handlers, Docker client logic
- Key files: `lib.rs` (app builder), `main.rs` (entry point)
- Structure (planned): Will expand to module-specific files as Docker operations grow

**`src-tauri/`** - Tauri Configuration and Build
- Purpose: Rust project configuration and assets
- Contains: Cargo.toml, tauri.conf.json, app icons, capabilities
- Key files: `tauri.conf.json` (window size, menu, permissions), `Cargo.toml` (dependencies)

**`docs/`** - Documentation
- Purpose: PRD, roadmap, architecture decisions
- Contains: Markdown documents for product and technical planning
- Key files: `PRD.md` (8 modules, requirements)

**`public/`** - Static Assets
- Purpose: Assets bundled with desktop application
- Contains: Logos, icons (Vite/Tauri)
- Served from: File system; accessible as `/public/` in app

**`.github/`** - GitHub Configuration
- Purpose: CI/CD workflows, issue templates
- Contains: YAML workflow files, issue templates
- Key files: `workflows/*.yml` (build, test, release pipelines)

## Key File Locations

**Entry Points:**
- `src/main.ts`: Frontend initialization (Vue app mount)
- `src-tauri/src/main.rs`: Rust binary entry (calls lib.rs)
- `src-tauri/src/lib.rs`: Tauri app builder (command registration, plugins)

**Configuration:**
- `vite.config.ts`: Vite build configuration (Tauri dev port 1420)
- `tsconfig.json`: TypeScript compiler options (ES2020 target, strict mode)
- `src-tauri/tauri.conf.json`: Window dimensions, menu, permissions
- `src-tauri/Cargo.toml`: Rust dependencies (tauri 2, serde, planned: bollard)
- `eslint.config.js`: ESLint rules (Vue 3 + TypeScript, ignores src-tauri)

**Core Logic:**
- `src/App.vue`: Root Vue component (scaffold template)
- `src-tauri/src/lib.rs`: Tauri command handlers (currently just `greet`)

**Testing:**
- Not yet implemented; will use Vitest (npm package present)

**Package Management:**
- `package.json`: Frontend dependencies (Vue 3, Tauri API, build tools)
- `pnpm-lock.yaml`: Locked versions of all npm packages
- `src-tauri/Cargo.lock`: Locked versions of all Rust dependencies

## Naming Conventions

**Files:**
- Frontend: camelCase for components and scripts (`App.vue`, `main.ts`)
- Rust: snake_case for files (`lib.rs`, `main.rs`)
- Configuration: kebab-case or lowercase (`vite.config.ts`, `tauri.conf.json`)

**Directories:**
- Lowercase, no hyphens: `src/`, `docs/`, `public/`
- Special prefixes: `src-tauri/` (Tauri convention), `.github/` (GitHub convention)

**Functions/Variables:**
- Rust: snake_case functions (`greet()`, `run()`)
- TypeScript: camelCase functions, PascalCase components
- Vue: PascalCase for component exports, camelCase for methods

**Types:**
- Rust: PascalCase structs/enums (no types yet defined)
- TypeScript: PascalCase interfaces/types (Vue 3 using implicit any for now)

## Where to Add New Code

**New Module (M1, M2, etc.):**
- Primary code: `src/components/M{N}/` (Vue components)
- Rust backend: `src-tauri/src/commands/{module_name}.rs` (new file; import in lib.rs)
- Tests: `src/__tests__/M{N}.spec.ts` or `src-tauri/tests/` (when testing is added)

**New Vue Component:**
- Location: `src/components/{ModuleName}/` or `src/pages/{RouteName}.vue`
- Pattern: Single-file component with `<script setup lang="ts">` and scoped styles
- Import in parent: `import Component from '@/components/...'` (alias TBD)

**New Tauri Command:**
- Location: `src-tauri/src/commands/{feature}.rs` (create if new feature) or `lib.rs` (if simple)
- Pattern: `#[tauri::command] async fn command_name(param: Type) -> ResultType { ... }`
- Register in: `src-tauri/src/lib.rs` in `.invoke_handler(tauri::generate_handler![...])` macro

**Utilities/Helpers:**
- Frontend: `src/utils/{utility_name}.ts` (when needed)
- Rust: `src-tauri/src/utils.rs` or module-specific file

**Shared State:**
- Frontend: `src/stores/` using Pinia (when needed)
- Rust: None (stateless command pattern)

## Special Directories

**`src-tauri/target/`:**
- Purpose: Rust build artifacts
- Generated: Yes (by `cargo build`)
- Committed: No (in `.gitignore`)

**`src-tauri/gen/`:**
- Purpose: Generated Tauri schemas and bindings
- Generated: Yes (by Tauri build process)
- Committed: No (in `.gitignore`)

**`node_modules/`:**
- Purpose: npm package cache
- Generated: Yes (by pnpm install)
- Committed: No (in `.gitignore`)

**`.planning/codebase/`:**
- Purpose: GSD planning documents (ARCHITECTURE.md, STRUCTURE.md, etc.)
- Generated: Yes (by `/gsd:map-codebase` command)
- Committed: Yes (for team reference)

**`.github/workflows/`:**
- Purpose: CI/CD pipeline definitions (GitHub Actions)
- Generated: No (manual)
- Committed: Yes (part of repo)

---

*Structure analysis: 2026-05-10*
