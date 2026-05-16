<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { toast } from 'vue-sonner'
import { commands } from '@/ipc/bindings'
import { useInfraStore } from '@/stores/useInfraStore'

const PAGE_SIZE = 10

const { t } = useI18n()
const store = useInfraStore()
const confirming = ref(false)
const page = ref(1)

const totalPages = computed(() => Math.max(1, Math.ceil(store.volumes.length / PAGE_SIZE)))
const pageVolumes = computed(() => {
  const start = (page.value - 1) * PAGE_SIZE
  return store.volumes.slice(start, start + PAGE_SIZE)
})

onMounted(() => {
  store.refreshVolumes()
})

function formatSize(mb: number | null): string {
  if (mb === null) return '—'
  if (mb < 1) return `${(mb * 1024).toFixed(0)} KB`
  return `${mb.toFixed(1)} MB`
}

async function pruneVolumes() {
  confirming.value = false
  const r = await commands.pruneVolumes().catch((e: Error) => {
    toast.error('Failed to prune volumes', { description: e.message })
    return null
  })
  if (!r || r.status !== 'ok') {
    if (r?.status === 'error') toast.error('Failed to prune volumes', { description: String(r.error) })
    return
  }
  const mb = Math.round(r.data / 1_048_576)
  toast.success(`Reclaimed ${mb} MB`)
  page.value = 1
  store.refreshVolumes()
}
</script>

<template>
  <div class="space-y-4">
    <div class="flex items-center justify-between">
      <h3 class="text-lg font-semibold">
        {{ t('volumes.title') }}
        <span class="ml-1 text-sm font-normal text-muted-foreground">({{ store.volumes.length }})</span>
      </h3>
      <div class="flex items-center gap-2">
        <template v-if="confirming">
          <span class="text-sm text-muted-foreground">{{ t('volumes.pruneConfirm') }}</span>
          <button
            class="rounded-md bg-destructive px-3 py-1 text-sm text-destructive-foreground hover:bg-destructive/90"
            @click="pruneVolumes"
          >
            {{ t('volumes.confirm') }}
          </button>
          <button
            class="rounded-md border px-3 py-1 text-sm hover:bg-accent"
            @click="confirming = false"
          >
            {{ t('volumes.cancel') }}
          </button>
        </template>
        <button
          v-else
          class="rounded-md border px-3 py-1 text-sm hover:bg-accent"
          @click="confirming = true"
        >
          {{ t('volumes.prune') }}
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
            <th class="px-4 py-2 text-left font-medium">{{ t('volumes.name') }}</th>
            <th class="px-4 py-2 text-left font-medium">{{ t('volumes.driver') }}</th>
            <th class="px-4 py-2 text-left font-medium">{{ t('volumes.mountpoint') }}</th>
            <th class="px-4 py-2 text-right font-medium" title="Sizes computed by 'docker system df'">{{ t('volumes.size') }}</th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="vol in pageVolumes"
            :key="vol.name"
            class="border-t transition-colors hover:bg-muted/30"
          >
            <td class="px-4 py-2 font-mono text-xs">{{ vol.name }}</td>
            <td class="px-4 py-2">{{ vol.driver }}</td>
            <td class="max-w-xs truncate px-4 py-2 font-mono text-xs">{{ vol.mountpoint }}</td>
            <td class="px-4 py-2 text-right text-muted-foreground">
              {{ formatSize(vol.size_mb) }}
            </td>
          </tr>
          <tr v-if="store.volumes.length === 0">
            <td colspan="4" class="px-4 py-6 text-center text-muted-foreground">{{ t('volumes.empty') }}</td>
          </tr>
        </tbody>
      </table>
    </div>

    <!-- Pagination -->
    <div v-if="totalPages > 1" class="flex items-center justify-between text-sm">
      <span class="text-muted-foreground">
        {{ t('common.page', { page, total: totalPages }) }}
      </span>
      <div class="flex gap-1">
        <button
          :disabled="page <= 1"
          class="rounded-md border px-3 py-1 hover:bg-accent disabled:opacity-40"
          @click="page--"
        >
          ←
        </button>
        <button
          :disabled="page >= totalPages"
          class="rounded-md border px-3 py-1 hover:bg-accent disabled:opacity-40"
          @click="page++"
        >
          →
        </button>
      </div>
    </div>
  </div>
</template>
