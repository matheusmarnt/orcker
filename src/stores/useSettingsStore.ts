import { defineStore } from 'pinia'
import { ref } from 'vue'
import { useColorMode } from '@vueuse/core'
import { commands } from '@/ipc/bindings'
import { i18n } from '@/i18n'

export const useSettingsStore = defineStore('settings', () => {
  const colorMode = useColorMode({ storageKey: 'orcker-theme' })
  // Tracks the explicit user preference ('light'|'dark'|'auto').
  // colorMode.value resolves 'auto' to the OS value, so we track preference separately.
  const theme = ref<'light' | 'dark' | 'auto'>('auto')
  const locale = ref<string>('en')
  const trayEnabled = ref<boolean>(false)
  const autostartEnabled = ref<boolean>(false)
  const dockerSocket = ref<string | null>(null)

  async function load() {
    const result = await commands.getSettings()
    if (result.status === 'ok') {
      const d = result.data
      theme.value = (d.theme as 'dark' | 'light' | 'auto') ?? 'auto'
      colorMode.value = theme.value
      setLocale(d.locale)
      trayEnabled.value = d.tray_enabled
      autostartEnabled.value = d.autostart_enabled
      dockerSocket.value = d.docker_socket ?? null
    }
  }

  async function save() {
    await commands.saveSettings({
      theme: theme.value,
      locale: locale.value,
      tray_enabled: trayEnabled.value,
      autostart_enabled: autostartEnabled.value,
      docker_socket: dockerSocket.value,
    })
  }

  function setTheme(t: 'dark' | 'light' | 'auto') {
    theme.value = t
    colorMode.value = t
  }

  function setLocale(l: string) {
    locale.value = l
    i18n.global.locale.value = l as 'en' | 'pt-BR' | 'es'
  }

  return {
    colorMode,
    theme,
    locale,
    trayEnabled,
    autostartEnabled,
    dockerSocket,
    load,
    save,
    setTheme,
    setLocale,
  }
})
