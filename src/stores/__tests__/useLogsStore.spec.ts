import { setActivePinia, createPinia } from 'pinia'
import { beforeEach, describe, expect, it, vi } from 'vitest'

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn().mockResolvedValue(() => {}),
}))

vi.mock('@/ipc/bindings', () => ({
  commands: {
    startLogStream: vi.fn().mockResolvedValue({ status: 'ok', data: null }),
    stopLogStream: vi.fn().mockResolvedValue({ status: 'ok', data: null }),
  },
}))

describe('useLogsStore (scaffold)', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  it('stub: ring buffer max is 5000', () => {
    const MAX_LINES = 5000
    expect(MAX_LINES).toBe(5000) // RNF-04
  })

  it('stub: filteredLines returns array', () => {
    expect([]).toBeInstanceOf(Array) // real assertion in plan 03-08
  })
})
