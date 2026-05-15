import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mount } from '@vue/test-utils'
import { defineComponent, ref } from 'vue'

// Mock monaco-editor-vue3 to avoid web worker / DOM complexity in Vitest
vi.mock('monaco-editor-vue3', () => ({
  default: defineComponent({
    name: 'MonacoEditor',
    props: ['modelValue', 'options', 'language'],
    emits: ['update:modelValue'],
    setup() { return {} },
    template: '<div class="monaco-stub" />',
  }),
}))

// Mock monaco-yaml to avoid ESM/worker issues
vi.mock('monaco-yaml', () => ({
  configureMonacoYaml: vi.fn(),
}))

// Mock @tauri-apps/api/core to avoid IPC in unit tests
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn().mockResolvedValue(''),
}))

// Mock the IPC bindings
vi.mock('@/ipc/bindings', () => ({
  commands: {
    readComposeFile: vi.fn().mockResolvedValue({ status: 'ok', data: 'version: "3"\n' }),
    saveComposeFile: vi.fn().mockResolvedValue({ status: 'ok', data: null }),
  },
}))

// Import component AFTER mocks are set up
import ComposeEditor from '../ComposeEditor.vue'

describe('ComposeEditor', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('blocks save when Monaco reports Error-severity markers (R-M5.1)', async () => {
    const wrapper = mount(ComposeEditor, {
      props: {
        projectId: 'test-project-id',
        projectStatus: 'stopped',
      },
      global: {
        stubs: {
          MonacoEditor: true,
        },
      },
    })

    // Simulate error marker (severity 8 = Error in Monaco)
    const vm = wrapper.vm as any
    vm.markers = [{ message: 'Unexpected token', severity: 8 }]
    await wrapper.vm.$nextTick()

    const saveBtn = wrapper.find('[data-testid="save-btn"]')
    expect(saveBtn.exists()).toBe(true)
    expect(saveBtn.attributes('disabled')).toBeDefined()
  })

  it('allows save when Monaco reports only Warning-severity markers', async () => {
    const wrapper = mount(ComposeEditor, {
      props: {
        projectId: 'test-project-id',
        projectStatus: 'stopped',
      },
      global: {
        stubs: {
          MonacoEditor: true,
        },
      },
    })

    // Simulate warning marker only (severity 4 = Warning in Monaco)
    const vm = wrapper.vm as any
    vm.markers = [{ message: 'Use quotes around string values', severity: 4 }]
    await wrapper.vm.$nextTick()

    const saveBtn = wrapper.find('[data-testid="save-btn"]')
    expect(saveBtn.exists()).toBe(true)
    expect(saveBtn.attributes('disabled')).toBeUndefined()
  })
})
