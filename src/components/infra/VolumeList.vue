<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { toast } from 'vue-sonner'
import { commands } from '@/ipc/bindings'
import { useInfraStore } from '@/stores/useInfraStore'

const store = useInfraStore()
const confirming = ref(false)

onMounted(() => {
  store.refreshVolumes()
})

async function pruneVolumes() {
  confirming.value = false
  const r = await commands.pruneVolumes().catch((e: Error) => {
    toast.error('Failed to prune volumes', { description: e.message })
    return null
  })
  if (!r) return
  if (r.status === 'error') {
    toast.error('Failed to prune volumes', { description: String(r.error) })
    return
  }
  const mb = Math.round(r.data / 1_048_576)
  toast.success(`Reclaimed ${mb} MB`)
  store.refreshVolumes()
}
</script>

<template>
  <div class="space-y-4">
    <div class="flex items-center justify-between">
      <h3 class="text-lg font-semibold">Volumes</h3>
      <div class="flex items-center gap-2">
        <template v-if="confirming">
          <span class="text-sm text-muted-foreground">Remove all unused volumes. Continue?</span>
          <button
            class="rounded-md bg-destructive px-3 py-1 text-sm text-destructive-foreground hover:bg-destructive/90"
            @click="pruneVolumes"
          >
            Confirm
          </button>
          <button
            class="rounded-md border px-3 py-1 text-sm hover:bg-accent"
            @click="confirming = false"
          >
            Cancel
          </button>
        </template>
        <button
          v-else
          class="rounded-md border px-3 py-1 text-sm hover:bg-accent"
          @click="confirming = true"
        >
          Prune Dangling Volumes
        </button>
      </div>
    </div>

    <!-- Loading skeleton -->
    <div v-if="store.isLoadingVolumes" class="space-y-2">
      <div v-for="n in 3" :key="n" class="h-10 animate-pulse rounded-md bg-muted" />
    </div>

    <!-- Volume table -->
    <div v-else class="overflow-x-auto rounded-md border">
      <table class="w-full text-sm">
        <thead class="bg-muted/50">
          <tr>
            <th class="px-4 py-2 text-left font-medium">Name</th>
            <th class="px-4 py-2 text-left font-medium">Driver</th>
            <th class="px-4 py-2 text-left font-medium">Mountpoint</th>
            <th class="px-4 py-2 text-right font-medium">Size (MB)</th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="vol in store.volumes"
            :key="vol.name"
            class="border-t transition-colors hover:bg-muted/30"
          >
            <td class="px-4 py-2 font-mono text-xs">{{ vol.name }}</td>
            <td class="px-4 py-2">{{ vol.driver }}</td>
            <td class="max-w-xs truncate px-4 py-2 font-mono text-xs">{{ vol.mountpoint }}</td>
            <td class="px-4 py-2 text-right">
              {{ vol.size_mb !== null ? vol.size_mb : '—' }}
            </td>
          </tr>
          <tr v-if="store.volumes.length === 0">
            <td colspan="4" class="px-4 py-6 text-center text-muted-foreground">No volumes found</td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
</template>
