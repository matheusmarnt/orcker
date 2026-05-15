import { describe, it, expect, vi, beforeEach } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'

// ---------------------------------------------------------------------------
// Mock @tauri-apps/api/event
// ---------------------------------------------------------------------------
let capturedListener: ((event: { payload: unknown }) => void) | null = null

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn((_channel: string, cb: (event: { payload: unknown }) => void) => {
    capturedListener = cb
    return Promise.resolve(() => {})
  }),
}))

// ---------------------------------------------------------------------------
// Mock @/ipc/bindings
// ---------------------------------------------------------------------------
vi.mock('@/ipc/bindings', () => ({
  commands: {
    getServicesStatus: vi.fn(() =>
      Promise.resolve({ status: 'ok', data: { redis: { kind: 'stopped' } } }),
    ),
    getServiceConfigs: vi.fn(() => Promise.resolve({ status: 'ok', data: {} })),
    toggleService: vi.fn(() => Promise.resolve({ status: 'ok', data: null })),
    setServiceConfig: vi.fn(() => Promise.resolve({ status: 'ok', data: false })),
    globalOn: vi.fn(() => Promise.resolve({ status: 'ok', data: null })),
    globalOff: vi.fn(() => Promise.resolve({ status: 'ok', data: null })),
  },
}))

// ---------------------------------------------------------------------------
// Also mock vue-sonner (no DOM in vitest)
// ---------------------------------------------------------------------------
vi.mock('vue-sonner', () => ({
  toast: { error: vi.fn(), success: vi.fn() },
}))

import { useGlobalStackStore } from '../useGlobalStackStore'

describe('useGlobalStackStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    capturedListener = null
  })

  it('updates statuses[service] reactively when global://service-status event arrives', async () => {
    const store = useGlobalStackStore()
    await store.init()

    // Simulate Rust emitting a service-status event
    expect(capturedListener).not.toBeNull()
    capturedListener!({ payload: { service: 'redis', status: { kind: 'running' } } })

    expect(store.statuses['redis']?.kind).toBe('running')
  })

  it('isTransitioning returns true for starting/stopping and false for other statuses', () => {
    const store = useGlobalStackStore()

    store.statuses['redis'] = { kind: 'starting' }
    expect(store.isTransitioning('redis')).toBe(true)

    store.statuses['redis'] = { kind: 'stopping' }
    expect(store.isTransitioning('redis')).toBe(true)

    store.statuses['redis'] = { kind: 'running' }
    expect(store.isTransitioning('redis')).toBe(false)

    store.statuses['redis'] = { kind: 'stopped' }
    expect(store.isTransitioning('redis')).toBe(false)

    store.statuses['redis'] = { kind: 'error', message: 'boom' }
    expect(store.isTransitioning('redis')).toBe(false)
  })
})
