<script setup lang="ts">
import { useSettingsStore } from '@/stores/useSettingsStore'
import { toast } from 'vue-sonner'
import { enable, disable } from '@tauri-apps/plugin-autostart'

const store = useSettingsStore()

async function saveDocker() {
  await store.save()
  toast.success('Settings saved')
}

function resetSocket() {
  store.dockerSocket = null
  store.save()
}

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
    <div>
      <h3 class="mb-1 text-sm font-semibold">Docker Socket Path</h3>
      <p class="mb-3 text-xs text-muted-foreground">
        Leave empty to use the auto-detected socket.
      </p>
      <div class="flex gap-2">
        <input
          v-model="store.dockerSocket"
          type="text"
          placeholder="Auto-detect (default)"
          class="flex-1 rounded-md border border-border bg-background px-3 py-2 text-sm placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-ring"
        />
        <button
          class="rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90"
          @click="saveDocker"
        >
          Save
        </button>
      </div>
      <button
        class="mt-1 text-xs text-muted-foreground underline hover:text-foreground"
        @click="resetSocket"
      >
        Reset to default
      </button>
    </div>

    <div class="space-y-4 border-t border-border pt-4">
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
  </div>
</template>
