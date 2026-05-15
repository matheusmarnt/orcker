import { defineStore } from 'pinia'
import { ref } from 'vue'
import { useColorMode } from '@vueuse/core'
import { commands } from '@/ipc/bindings'

export const useSettingsStore = defineStore('settings', () => {
  const colorMode = useColorMode({ storageKey: 'orcker-theme' })
  const locale = ref<string>('en')
  const trayEnabled = ref<boolean>(false)
  const autostartEnabled = ref<boolean>(false)
  const dockerSocket = ref<string | null>(null)

  async function load() {
    const result = await commands.getSettings()
    if (result.status === 'ok') {
      const d = result.data
      colorMode.value = d.theme as 'dark' | 'light' | 'auto'
      locale.value = d.locale
      trayEnabled.value = d.tray_enabled
      autostartEnabled.value = d.autostart_enabled
      dockerSocket.value = d.docker_socket ?? null
    }
  }

  async function save() {
    await commands.saveSettings({
      theme: colorMode.value,
      locale: locale.value,
      tray_enabled: trayEnabled.value,
      autostart_enabled: autostartEnabled.value,
      docker_socket: dockerSocket.value,
    })
  }

  function setTheme(t: 'dark' | 'light' | 'auto') {
    colorMode.value = t
  }

  return {
    colorMode,
    locale,
    trayEnabled,
    autostartEnabled,
    dockerSocket,
    load,
    save,
    setTheme,
  }
})
