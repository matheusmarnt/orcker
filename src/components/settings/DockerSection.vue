<script setup lang="ts">
import { useSettingsStore } from '@/stores/useSettingsStore'
import { toast } from 'vue-sonner'

const store = useSettingsStore()

async function saveDocker() {
  await store.save()
  toast.success('Settings saved')
}

function resetSocket() {
  store.dockerSocket = null
  store.save()
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

  </div>
</template>
