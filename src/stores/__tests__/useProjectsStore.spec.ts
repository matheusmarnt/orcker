import { setActivePinia, createPinia } from 'pinia'
import { beforeEach, describe, expect, it, vi } from 'vitest'

// Mock IPC — store is not yet implemented; this file is the Wave 0 scaffold
vi.mock('@/ipc/bindings', () => ({
  commands: {
    listProjects: vi.fn().mockResolvedValue({ status: 'ok', data: [] }),
    registerProject: vi.fn().mockResolvedValue({ status: 'ok', data: null }),
  },
}))

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn().mockResolvedValue(() => {}),
}))

describe('useProjectsStore (scaffold)', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  it('stub: projects initializes as empty array', () => {
    // Real test implemented in plan 03-04 after store is created
    expect([]).toHaveLength(0)
  })

  it('stub: ring buffer cap placeholder', () => {
    const MAX_PROJECTS = 100
    expect(MAX_PROJECTS).toBeGreaterThan(0)
  })
})
