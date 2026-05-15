<script setup lang="ts">
import { useSettingsStore } from '@/stores/useSettingsStore'
import { enable, disable } from '@tauri-apps/plugin-autostart'

const store = useSettingsStore()

async function onTrayToggle() {
  await store.save()
  if (!store.trayEnabled) {
    store.autostartEnabled = false
    await store.save()
    try { await disable() } catch { /* ignore */ }
  }
}

async function onAutostartToggle() {
  await store.save()
  try {
    if (store.autostartEnabled) {
      await enable()
    } else {
      await disable()
    }
  } catch {
    // autostart may not be supported in dev
  }
}
</script>

<template>
  <div class="space-y-6">
    <div class="flex items-center justify-between">
      <div>
        <p class="text-sm font-semibold">System Tray</p>
        <p class="text-xs text-muted-foreground">Keep Orcker in the system tray when closed.</p>
      </div>
      <button
        :class="[
          'relative inline-flex h-6 w-11 items-center rounded-full transition-colors focus:outline-none focus:ring-2 focus:ring-ring',
          store.trayEnabled ? 'bg-primary' : 'bg-muted',
        ]"
        role="switch"
        :aria-checked="store.trayEnabled"
        @click="store.trayEnabled = !store.trayEnabled; onTrayToggle()"
      >
        <span
          :class="[
            'inline-block h-4 w-4 rounded-full bg-white shadow transition-transform',
            store.trayEnabled ? 'translate-x-6' : 'translate-x-1',
          ]"
        />
      </button>
    </div>

    <div v-if="store.trayEnabled" class="flex items-center justify-between">
      <div>
        <p class="text-sm font-semibold">Start at Login</p>
        <p class="text-xs text-muted-foreground">Launch Orcker automatically on system startup.</p>
      </div>
      <button
        :class="[
          'relative inline-flex h-6 w-11 items-center rounded-full transition-colors focus:outline-none focus:ring-2 focus:ring-ring',
          store.autostartEnabled ? 'bg-primary' : 'bg-muted',
        ]"
        role="switch"
        :aria-checked="store.autostartEnabled"
        @click="store.autostartEnabled = !store.autostartEnabled; onAutostartToggle()"
      >
        <span
          :class="[
            'inline-block h-4 w-4 rounded-full bg-white shadow transition-transform',
            store.autostartEnabled ? 'translate-x-6' : 'translate-x-1',
          ]"
        />
      </button>
    </div>
  </div>
</template>
