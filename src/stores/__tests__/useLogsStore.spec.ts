import { setActivePinia, createPinia } from 'pinia'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { useLogsStore } from '../useLogsStore'

vi.mock('@tauri-apps/api/core', () => ({
  Channel: vi.fn().mockImplementation(() => ({ onmessage: null })),
}))

vi.mock('@/ipc/bindings', () => ({
  commands: {
    startLogStream: vi.fn().mockResolvedValue({ status: 'ok', data: null }),
    stopLogStream: vi.fn().mockResolvedValue({ status: 'ok', data: null }),
  },
}))

describe('useLogsStore', () => {
  beforeEach(() => { setActivePinia(createPinia()) })

  it('ring buffer caps at 5000 lines', () => {
    const store = useLogsStore()
    for (let i = 0; i < 6000; i++) {
      store.appendLine({ text: `line ${i}`, source: 'Docker', timestamp: null })
    }
    expect(store.lines.length).toBe(5000) // RNF-04
  })

  it('filteredLines filters by keyword (case-insensitive)', () => {
    const store = useLogsStore()
    store.appendLine({ text: 'ERROR: database connection failed', source: 'Laravel', timestamp: null })
    store.appendLine({ text: 'INFO: request received', source: 'Laravel', timestamp: null })
    store.keywordFilter = 'error'
    expect(store.filteredLines).toHaveLength(1)
    expect(store.filteredLines[0].text).toContain('ERROR')
  })

  it('filteredLines filters by source', () => {
    const store = useLogsStore()
    store.appendLine({ text: 'docker log line', source: 'Docker', timestamp: null })
    store.appendLine({ text: 'laravel log line', source: 'Laravel', timestamp: null })
    store.activeSource = 'Docker'
    expect(store.filteredLines).toHaveLength(1)
    expect(store.filteredLines[0].source).toBe('Docker')
  })
})
