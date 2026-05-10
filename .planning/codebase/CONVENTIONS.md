# Coding Conventions

**Analysis Date:** 2026-05-10

## Naming Patterns

**Files:**
- Vue components: PascalCase (e.g., `App.vue`)
- TypeScript files: camelCase for utilities, PascalCase for classes/types (e.g., `main.ts`)
- Rust modules: snake_case (e.g., `orcker_scaffold_lib`)

**Functions:**
- TypeScript/Vue: camelCase (e.g., `greet()`, `invoke()`)
- Rust: snake_case (e.g., `pub fn run()`)
- Tauri commands: snake_case with `#[tauri::command]` macro (e.g., `fn greet()`)

**Variables:**
- TypeScript: camelCase (e.g., `greetMsg`, `name`)
- Vue reactive refs: camelCase using `ref()` (e.g., `const name = ref("")`)
- Rust: snake_case (e.g., `crate_type`)

**Types:**
- TypeScript: PascalCase for interfaces/types (e.g., `DefineComponent`)
- Vue: PascalCase for component names and type exports
- Rust: PascalCase for structs/enums (e.g., `Builder`)

## Code Style

**Formatting:**
- No explicit Prettier config detected; defaults apply
- Double quotes in JavaScript/TypeScript (observed in config: `"vue"`, `"@tauri-apps/api"`)
- 2-space indentation (standard for Node.js projects)

**Linting:**
- ESLint 9.17.0 with `eslint.config.js` (flat config format)
- Plugins: `eslint-plugin-vue` for Vue 3, `@vue/eslint-config-typescript` for TypeScript
- ESLint rules: `@vue/eslint-config-typescript` applies strict type checking
- Run: `npm run lint` (max-warnings set to 0 — all warnings fail)
- Fix: `npm run lint:fix`

**TypeScript Strictness:**
- `strict: true` — enables all strict type checking flags
- `noUnusedLocals: true` — enforces no unused variables
- `noUnusedParameters: true` — enforces no unused function parameters
- `noFallthroughCasesInSwitch: true` — prevents accidental fallthrough in switch statements
- `isolatedModules: true` — ensures each file can be safely transpiled

## Import Organization

**Order:**
1. Vue/framework imports (e.g., `import { ref } from "vue"`)
2. Tauri API imports (e.g., `import { invoke } from "@tauri-apps/api/core"`)
3. Local component/utility imports (relative paths)

**Path Aliases:**
- No path aliases detected in `tsconfig.json`
- All imports use relative paths or module names from `node_modules`

**Module Targets:**
- Module resolution: `bundler` (supports modern bundling)
- `allowImportingTsExtensions: true` — allows importing `.ts` files directly

## Error Handling

**Patterns:**
- Tauri command errors: wrapped in `.expect()` for panics (e.g., `.expect("error while running tauri application")`)
- Async operations: use standard `async/await` (Vue setup with `async function greet()`)
- No explicit try-catch observed in current codebase — patterns to establish for phase 1+

**Recommendations:**
- Use try-catch for Tauri command invocations: `try { await invoke(...) } catch (e) { /* handle */ }`
- Rust: Use `Result<T, E>` for recoverable errors rather than `.expect()`

## Logging

**Framework:** No logging library detected; use `console.log/warn/error` for frontend

**Patterns:**
- Frontend: Direct console usage (seen in comments: `Learn more about Tauri commands`)
- Rust: No explicit logging framework in `Cargo.toml`; use `println!` or add `tracing` in future phases

**When to log:**
- Tauri command execution boundaries
- Error conditions with context
- Development-mode debug info only

## Comments

**When to Comment:**
- Explain WHY, not WHAT (code shows the WHAT)
- Comment non-obvious logic, workarounds, or business rules
- Mark sections with TODO/FIXME when blocking

**JSDoc/TSDoc:**
- Not currently used; recommended for public APIs
- Vue components: use `/** @description ... */` for setup scripts
- Rust: use `///` for doc comments on public items

**Example (recommended):**
```typescript
/**
 * Sends a greeting command to the Rust backend
 * @param name - User input name
 * @returns Promise resolving to greeting message
 */
async function greet() {
  greetMsg.value = await invoke("greet", { name: name.value });
}
```

## Function Design

**Size:** Keep functions under 50 lines; extract helpers for complex logic

**Parameters:**
- Prefer named parameters where function has 3+ arguments
- Use object destructuring in TypeScript
- Example: `async function greet({ name }: { name: string })`

**Return Values:**
- Explicitly type return values (enforced by TypeScript strict mode)
- Use `Promise<T>` for async functions
- Avoid implicit `undefined`; explicitly return or throw

**Rust Functions:**
- Mark public items with `pub`; private by default
- Use attributes: `#[tauri::command]` for exposed commands
- Always specify return type (e.g., `-> String`)

## Module Design

**Exports:**
- Vue components: default export of component object
- TypeScript utilities: named exports preferred for tree-shaking
- Example in `src/main.ts`: `import App from "./App.vue"` (default)

**Barrel Files:**
- Not currently used; OK for future utility exports
- Pattern: create `src/utils/index.ts` re-exporting helpers

**Component Structure (Vue 3 + TypeScript):**
- Use `<script setup lang="ts">` syntax
- Declare reactive state at top: `const name = ref("")`
- Methods directly below state
- Template references variables directly (no `this.`)
- Scoped styles for encapsulation

## Rust Conventions

**Crate Structure:**
- Library entry: `src/lib.rs` (exports public functions)
- Binary entry: `src/main.rs` (typically calls lib function)
- Current: `main.rs` calls `orcker_scaffold_lib::run()` from `lib.rs`

**Tauri Integration:**
- Commands exposed via `#[tauri::command]` macro
- All commands registered in `invoke_handler` via `generate_handler![]`
- Plugin initialization in `Builder::default()` chain

**Serialization:**
- Use `serde` with `#[derive(Serialize, Deserialize)]`
- Already in dependencies: `serde`, `serde_json`

---

*Convention analysis: 2026-05-10*
