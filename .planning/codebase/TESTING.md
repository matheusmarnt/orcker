# Testing Patterns

**Analysis Date:** 2026-05-10

## Test Framework

**Runner:**
- Vitest 2.1.8
- Config: Not yet created (no `vitest.config.ts` detected)
- Inherits from Vite: Uses `vite.config.ts` with Vue plugin

**Assertion Library:**
- Vitest ships with Chai assertions by default
- Can be imported: `import { expect } from "vitest"`

**Run Commands:**
```bash
npm run test              # Run all tests (with --passWithNoTests flag)
npm run test:watch       # Watch mode for active development
```

**Command Details:**
- `npm run test` runs `vitest run --passWithNoTests` — allows test suites to pass if no tests exist (useful for early phases)
- `npm run test:watch` runs `vitest` without flags — continuous mode, re-runs on file changes
- No coverage reporting configured yet

## Test File Organization

**Location:**
- Pattern: Co-located tests (tests near source files)
- Recommended structure: `src/components/Button.vue` → `src/components/Button.spec.ts`
- Alternative: `tests/unit/components/Button.spec.ts` for separation

**Naming:**
- Test files: `*.spec.ts` or `*.test.ts` 
- Vitest discovers both patterns automatically

**Structure:**
```
src/
├── components/
│   ├── Button.vue
│   └── Button.spec.ts
├── utils/
│   ├── helpers.ts
│   └── helpers.spec.ts
└── main.ts
```

**Current Status:**
- No test files exist yet in codebase
- Vitest package installed but no configuration file created
- First test setup should create `vitest.config.ts` or use Vite's config

## Test Structure

**Suite Organization (recommended pattern):**
```typescript
import { describe, it, expect } from "vitest";

describe("Component: Button", () => {
  it("should render button element", () => {
    // arrange
    // act
    // assert
  });

  it("should emit click event", () => {
    // test implementation
  });
});
```

**Patterns:**
- Setup: Use `beforeEach()` for common state (fixtures, mocks)
- Teardown: Use `afterEach()` for cleanup (clear timers, reset mocks)
- Assertion: Use `expect()` with Chai matchers

**Example Setup:**
```typescript
import { beforeEach, afterEach, describe, it, expect } from "vitest";

describe("Utils: greetMessage", () => {
  let state: { name: string };

  beforeEach(() => {
    state = { name: "Test User" };
  });

  afterEach(() => {
    // cleanup
  });

  it("returns greeting with name", () => {
    const result = greetMessage(state.name);
    expect(result).toContain("Test User");
  });
});
```

## Mocking

**Framework:** Vitest includes `vi` mocking utilities

**Patterns (to establish):**
```typescript
import { vi, describe, it, expect } from "vitest";

// Mock Tauri invoke
vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue("mocked response"),
}));

// Mock internal modules
vi.mock("./myModule", () => ({
  myFunction: vi.fn().mockReturnValue(42),
}));

// Clear mocks between tests
afterEach(() => {
  vi.clearAllMocks();
});
```

**What to Mock:**
- External APIs (Tauri `invoke`, network calls)
- File system operations
- Time-dependent functions (`Date`, `setTimeout`)
- Heavy dependencies (crypto, compression)

**What NOT to Mock:**
- Core Vue reactivity (`ref`, `computed`)
- Simple utility functions
- Component lifecycle hooks
- DOM interactions (test real DOM behavior)

## Fixtures and Factories

**Test Data (recommended pattern):**
```typescript
// src/utils/test-fixtures.ts
export const mockGreetCommand = {
  name: "John",
  expected: "Hello, John! You've been greeted from Rust!",
};

export function createMockComponent() {
  return {
    name: ref(""),
    greetMsg: ref(""),
    greet: vi.fn(),
  };
}
```

**Location:**
- Shared fixtures: `src/utils/test-fixtures.ts`
- Component-specific: `src/components/__fixtures__/ButtonFixtures.ts`
- Keep fixtures close to what they test

## Coverage

**Requirements:** Not enforced yet — no coverage config in `package.json`

**View Coverage (to setup):**
```bash
vitest run --coverage
```

**Recommended config in `vitest.config.ts`:**
```typescript
export default defineConfig({
  test: {
    coverage: {
      provider: "v8",
      reporter: ["text", "json", "html"],
      all: true,
      include: ["src/**/*.{ts,vue}"],
      exclude: ["src/**/*.spec.ts"],
    },
  },
});
```

**Target:** 80%+ coverage for core modules (adapters, commands); 60%+ for UI

## Test Types

**Unit Tests:**
- Scope: Single function or component in isolation
- Approach: Mock all dependencies, test inputs/outputs
- Example: Test `greet()` function with mocked Tauri invoke
- Location: `src/utils/helpers.spec.ts`, `src/components/Button.spec.ts`

**Integration Tests:**
- Scope: Multiple modules working together (e.g., component + service)
- Approach: Use real module interactions, mock only external APIs
- Example: Test component that calls Tauri command
- Location: `src/components/__tests__/App.integration.spec.ts` or separate `tests/integration/`

**E2E Tests:**
- Framework: Not currently configured (Future phase)
- Recommended: Tauri's built-in E2E runner or Playwright
- Scope: Full app workflows (Tauri window, actual backend calls)
- Location: `tests/e2e/` (create when needed)

**Current Phase:** Unit tests only; integration after phase 1 scaffold

## Common Patterns

**Async Testing:**
```typescript
import { describe, it, expect } from "vitest";

describe("Async greet", () => {
  it("should resolve with greeting", async () => {
    const result = await greet("Alice");
    expect(result).toBe("Hello, Alice! You've been greeted from Rust!");
  });

  it("should reject on error", async () => {
    // Mock invoke to reject
    vi.mocked(invoke).mockRejectedValue(new Error("Tauri error"));
    
    await expect(greet("Bob")).rejects.toThrow("Tauri error");
  });
});
```

**Error Testing:**
```typescript
describe("Error handling", () => {
  it("should handle invalid input", () => {
    expect(() => {
      processName(null);
    }).toThrow("Name required");
  });

  it("should catch async errors", async () => {
    const promise = failingAsyncFn();
    await expect(promise).rejects.toThrow();
  });
});
```

**Vue Component Testing:**
```typescript
import { describe, it, expect } from "vitest";
import { mount } from "@vue/test-utils";
import Button from "./Button.vue";

describe("Button.vue", () => {
  it("renders slot content", () => {
    const wrapper = mount(Button, {
      slots: {
        default: "Click me",
      },
    });
    expect(wrapper.text()).toContain("Click me");
  });

  it("emits click event", async () => {
    const wrapper = mount(Button);
    await wrapper.trigger("click");
    expect(wrapper.emitted("click")).toBeDefined();
  });
});
```

## Configuration (To Create)

**Recommended `vitest.config.ts`:**
```typescript
import { defineConfig } from "vitest/config";
import vue from "@vitejs/plugin-vue";

export default defineConfig({
  plugins: [vue()],
  test: {
    globals: true,
    environment: "jsdom",
    setupFiles: [],
    include: ["src/**/*.{test,spec}.ts", "src/**/*.{test,spec}.vue"],
    coverage: {
      provider: "v8",
      reporter: ["text", "html"],
      all: true,
      include: ["src/**/*.{ts,vue}"],
      exclude: ["src/**/*.{spec,test}.ts"],
    },
  },
});
```

---

*Testing analysis: 2026-05-10*
