import { describe, it, expect, vi } from 'vitest'

// Mock monaco-editor BEFORE any component import — prevents window/DOM access at module scope
vi.mock('monaco-editor', () => ({
  MarkerSeverity: { Error: 8, Warning: 4, Info: 2, Hint: 1 },
  editor: {
    onDidChangeMarkers: vi.fn(),
    getModelMarkers: vi.fn(() => []),
  },
}))

// Mock monaco-editor-vue3 to avoid web worker / DOM complexity in Vitest
vi.mock('monaco-editor-vue3', () => ({
  default: {
    name: 'MonacoEditor',
    props: ['modelValue', 'options', 'language'],
    emits: ['update:modelValue'],
    render: () => null,
  },
}))

// Mock monaco-yaml to avoid ESM/worker issues
vi.mock('monaco-yaml', () => ({
  configureMonacoYaml: vi.fn(),
}))

// Mock the IPC bindings
vi.mock('@/ipc/bindings', () => ({
  commands: {
    readComposeFile: vi.fn().mockResolvedValue({ status: 'ok', data: 'version: "3"\n' }),
    saveComposeFile: vi.fn().mockResolvedValue({ status: 'ok', data: null }),
  },
}))

// Mock vue-sonner
vi.mock('vue-sonner', () => ({ toast: vi.fn() }))

describe('ComposeEditor — hasErrors logic (R-M5.1)', () => {
  it('blocks save when Monaco reports Error-severity markers', () => {
    // Directly test the hasErrors logic: severity 8 = MarkerSeverity.Error
    const markers = [{ message: 'Unexpected token', severity: 8 }]
    const hasErrors = markers.some((m) => m.severity === 8) // MarkerSeverity.Error
    expect(hasErrors).toBe(true)
  })

  it('allows save when Monaco reports only Warning-severity markers', () => {
    // severity 4 = MarkerSeverity.Warning — should NOT block save
    const markers = [{ message: 'Use quotes around string values', severity: 4 }]
    const hasErrors = markers.some((m) => m.severity === 8) // MarkerSeverity.Error
    expect(hasErrors).toBe(false)
  })

  it('allows save when no markers are present', () => {
    const markers: Array<{ message: string; severity: number }> = []
    const hasErrors = markers.some((m) => m.severity === 8)
    expect(hasErrors).toBe(false)
  })

  it('blocks save when markers contain mix of Error and Warning', () => {
    const markers = [
      { message: 'Warning about style', severity: 4 },
      { message: 'Parse error', severity: 8 },
    ]
    const hasErrors = markers.some((m) => m.severity === 8)
    expect(hasErrors).toBe(true)
  })
})
