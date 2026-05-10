# Technology Stack

**Analysis Date:** 2026-05-10

## Languages

**Primary:**
- TypeScript 5.6.2 - Frontend application logic and type safety
- Vue 3.5.13 - UI framework and reactive components
- Rust 2021 edition - Backend/desktop app logic via Tauri

**Secondary:**
- JavaScript - Build configuration and ESLint setup

## Runtime

**Environment:**
- Tauri 2.x - Desktop application runtime (Rust backend + Web frontend)
- Node.js - Development and build tooling (via pnpm)

**Package Manager:**
- pnpm - Monorepo-aware package manager
- Lockfile: `pnpm-lock.yaml` present (98.0K)

## Frameworks

**Core:**
- Tauri 2.11.x - Desktop framework providing Rust backend + Vue frontend integration
  - Purpose: Create cross-platform desktop app with native Rust commands
  - CLI: `@tauri-apps/cli` 2.11.1
  - API bindings: `@tauri-apps/api` 2.11.0

**UI:**
- Vue 3.5.13 - Reactive UI framework with Composition API
- Vue Test Utils 2.4.6 - Vue component testing

**Build/Dev:**
- Vite 6.0.3 - Frontend build tool and dev server
- @vitejs/plugin-vue 5.2.1 - Vue SFC support in Vite
- vue-tsc 2.1.10 - Vue-aware TypeScript type checking

**Testing:**
- Vitest 2.1.8 - Vite-native unit test runner

## Key Dependencies

**Critical:**
- `@tauri-apps/api` 2.11.0 - JavaScript bridge to Rust commands
- `@tauri-apps/plugin-opener` 2 - Native link/file opener plugin

**Build/Infrastructure:**
- `tauri-build` 2 - Build-time Tauri configuration
- `tauri-plugin-opener` 2 (Rust) - OS file/URL opener integration
- `serde` 1.0 - Serialization/deserialization (Rust)
- `serde_json` 1.0 - JSON handling (Rust)

## Configuration

**Environment:**
- No `.env` file detected - Configuration via `tauri.conf.json`
- Development server: `http://localhost:1420` (Vite)
- Tauri dev command: `pnpm dev` (via `vite.config.ts`)

**Build:**
- Frontend build config: `vite.config.ts`
  - Vue plugin integration
  - Dev server on fixed port 1420
  - HMR configuration for remote dev
  - Ignores `src-tauri/` from watch
- Tauri config: `src-tauri/tauri.conf.json`
  - App window: 1280x800 (min 900x600)
  - Security: CSP with inline styles allowed
  - Multi-platform bundling: all targets
- TypeScript: `tsconfig.json` (ES2020 target, strict mode enabled)

## Platform Requirements

**Development:**
- Node.js with pnpm
- Rust toolchain (for Tauri)
- Platform-specific build tools (XCode for macOS, Visual Studio for Windows)

**Production:**
- Desktop application (Windows, macOS, Linux via Tauri)
- Packaged as native executables with bundled resources
- Icons: `src-tauri/icons/` directory (PNG, ICNS, ICO formats)

---

*Stack analysis: 2026-05-10*
