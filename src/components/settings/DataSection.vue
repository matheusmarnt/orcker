<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import { open, save } from '@tauri-apps/plugin-dialog'
import { readTextFile, writeTextFile } from '@tauri-apps/plugin-fs'
import { toast } from 'vue-sonner'
import { useSettingsStore } from '@/stores/useSettingsStore'

const { t } = useI18n()
const store = useSettingsStore()

async function exportConfig() {
  try {
    const data = {
      theme: store.colorMode,
      locale: store.locale,
      tray_enabled: store.trayEnabled,
      autostart_enabled: store.autostartEnabled,
      docker_socket: store.dockerSocket,
    }
    const json = JSON.stringify(data, null, 2)
    const path = await save({
      defaultPath: 'orcker-config.json',
      filters: [{ name: 'JSON', extensions: ['json'] }],
    })
    if (!path) return
    await writeTextFile(path, json)
    toast.success('Config exported')
  } catch (err) {
    toast.error(`Export failed: ${String(err)}`)
  }
}

async function importConfig() {
  try {
    const path = await open({
      multiple: false,
      filters: [{ name: 'JSON', extensions: ['json'] }],
    })
    if (!path) return
    const raw = await readTextFile(path as string)
    const data = JSON.parse(raw)
    if (data.theme) store.setTheme(data.theme)
    if (data.locale) store.locale = data.locale
    if (typeof data.tray_enabled === 'boolean') store.trayEnabled = data.tray_enabled
    if (typeof data.autostart_enabled === 'boolean') store.autostartEnabled = data.autostart_enabled
    if ('docker_socket' in data) store.dockerSocket = data.docker_socket ?? null
    await store.save()
    toast.success('Config imported')
  } catch (err) {
    toast.error(`Import failed: ${String(err)}`)
  }
}
</script>

<template>
  <div class="space-y-6">
    <div>
      <h3 class="mb-1 text-sm font-semibold">{{ t('settings.exportConfig') }}</h3>
      <p class="mb-3 text-xs text-muted-foreground">
        {{ t('settings.exportConfigDescription') }}
      </p>
      <button
        class="rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90"
        @click="exportConfig"
      >
        {{ t('settings.exportButton') }}
      </button>
    </div>

    <div class="border-t border-border pt-4">
      <h3 class="mb-1 text-sm font-semibold">{{ t('settings.importConfig') }}</h3>
      <p class="mb-3 text-xs text-muted-foreground">
        {{ t('settings.importConfigDescription') }}
      </p>
      <button
        class="rounded-md border border-border bg-background px-4 py-2 text-sm font-medium hover:bg-accent hover:text-accent-foreground"
        @click="importConfig"
      >
        {{ t('settings.importButton') }}
      </button>
    </div>
  </div>
</template>
