import { setActivePinia, createPinia } from 'pinia'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { useProjectsStore } from '../useProjectsStore'

vi.mock('@/ipc/bindings', () => {
  const proj = { id: '1', name: 'my-app', path: '/home/user/my-app', vite_auto: true }
  return {
    commands: {
      listProjects: vi.fn().mockResolvedValue({ status: 'ok', data: [proj] }),
      registerProject: vi.fn().mockResolvedValue({ status: 'ok', data: proj }),
      importProject: vi.fn().mockResolvedValue({ status: 'ok', data: { path: '/home', detected_files: ['artisan'] } }),
      pickProjectFolder: vi.fn().mockResolvedValue({ status: 'ok', data: '/home/user/new-app' }),
      getProjectStatus: vi.fn().mockResolvedValue({ status: 'ok', data: 'running' }),
    },
  }
})

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn().mockResolvedValue(() => {}),
}))

vi.mock('vue-sonner', () => ({ toast: { success: vi.fn(), error: vi.fn() } }))

describe('useProjectsStore', () => {
  beforeEach(() => { setActivePinia(createPinia()) })

  it('init loads projects from backend', async () => {
    const store = useProjectsStore()
    await store.init()
    expect(store.projects).toHaveLength(1)
    expect(store.projects[0].name).toBe('my-app')
  })

  it('loading is false after init completes', async () => {
    const store = useProjectsStore()
    await store.init()
    expect(store.loading).toBe(false)
  })

  it('registerProject adds project to list', async () => {
    const store = useProjectsStore()
    const result = await store.registerProject('my-app', '/home/user/my-app')
    expect(result).not.toBeNull()
    expect(store.projects.some(p => p.id === '1')).toBe(true)
  })
})
