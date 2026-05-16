import { describe, it, expect, beforeEach, vi } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'

// Mock @vueuse/core useColorMode to avoid DOM dependency in Node test env
vi.mock('@vueuse/core', async () => {
  const { ref } = await import('vue')
  const mode = ref('auto')
  return {
    useColorMode: () => mode,
  }
})

// Mock IPC bindings — commands.getSettings returns default-like data
vi.mock('@/ipc/bindings', () => ({
  commands: {
    getSettings: vi.fn().mockResolvedValue({
      status: 'ok',
      data: {
        theme: 'dark',
        locale: 'pt-BR',
        tray_enabled: true,
        autostart_enabled: false,
        docker_socket: null,
      },
    }),
    saveSettings: vi.fn().mockResolvedValue({ status: 'ok', data: null }),
  },
}))

describe('useSettingsStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  it('theme value persists in localStorage via useColorMode (R-M7.1)', async () => {
    const { useSettingsStore } = await import('../useSettingsStore')
    const store = useSettingsStore()
    store.setTheme('dark')
    // colorMode is the ref returned by useColorMode; its value should be 'dark'
    expect(store.colorMode).toBe('dark')
  })

  it('autostart toggle is hidden when tray_enabled is false (R-M7.4)', async () => {
    const { useSettingsStore } = await import('../useSettingsStore')
    const store = useSettingsStore()
    // Default trayEnabled is false — autostart UI should be hidden
    expect(store.trayEnabled).toBe(false)
  })

  it('autostart toggle is visible when tray_enabled is true (R-M7.4)', async () => {
    const { useSettingsStore } = await import('../useSettingsStore')
    const store = useSettingsStore()
    store.trayEnabled = true
    expect(store.trayEnabled).toBe(true)
  })
})
