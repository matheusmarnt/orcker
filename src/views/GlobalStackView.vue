<script setup lang="ts">
import { ref, watch, onMounted, onUnmounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { listen } from '@tauri-apps/api/event'
import { toast } from 'vue-sonner'
import { Skeleton } from '@/components/ui/skeleton'
import ServiceCard from '@/components/global/ServiceCard.vue'
import { useGlobalStackStore } from '@/stores/useGlobalStackStore'
import type { ServiceStatus } from '@/ipc/bindings'

const { t } = useI18n()
const store = useGlobalStackStore()
const loading = ref(true)
let unlistenShortcut: (() => void) | null = null

const labels: Record<string, string> = {
  redis: 'Redis',
  postgres: 'PostgreSQL',
  mysql: 'MySQL',
  mailpit: 'Mailpit',
  minio: 'MinIO',
  soketi: 'Soketi',
  meilisearch: 'Meilisearch',
}

// Track previous statuses manually — deep watch gives same ref for old/new
const prevStatuses = ref<Record<string, ServiceStatus>>({})

watch(
  () => store.statuses,
  (current) => {
    for (const [id, status] of Object.entries(current)) {
      const prev = prevStatuses.value[id]
      const label = labels[id] ?? id

      if (!prev) {
        // Initial load — seed baseline without toasting
        prevStatuses.value[id] = { ...status }
        continue
      }

      if (status.kind === 'running' && prev.kind !== 'running') {
        toast.success(`${label} started`)
      } else if (status.kind === 'stopped' && prev.kind !== 'stopped') {
        toast.info(`${label} stopped`)
      } else if (status.kind === 'error' && prev.kind !== 'error') {
        const msg = 'message' in status ? String(status.message) : 'error'
        toast.error(`${label}: ${msg}`)
      }

      prevStatuses.value[id] = { ...status }
    }
  },
  { deep: true },
)

onMounted(async () => {
  await store.init()
  loading.value = false

  const unlisten = await listen('global://shortcut-toggle', () => {
    store.smartToggle()
  })
  unlistenShortcut = unlisten
})

onUnmounted(() => {
  unlistenShortcut?.()
})
</script>

<template>
  <div class="flex flex-col gap-6 p-6">
    <!-- Header -->
    <div>
      <h1 class="text-xl font-bold">{{ t('global.title') }}</h1>
      <p class="mt-1 text-sm text-muted-foreground">
        {{ t('global.description') }}
      </p>
    </div>

    <!-- Skeleton during initial load -->
    <div v-if="loading" class="grid grid-cols-3 gap-4">
      <Skeleton v-for="i in 7" :key="i" class="h-40 rounded-lg" />
    </div>

    <!-- Service cards -->
    <div v-else class="grid grid-cols-3 gap-4">
      <ServiceCard service-id="redis" label="Redis" :default-port="6379" />
      <ServiceCard service-id="postgres" label="PostgreSQL" :default-port="5432" />
      <ServiceCard service-id="mysql" label="MySQL" :default-port="3306" />
      <ServiceCard service-id="mailpit" label="Mailpit" :default-port="8025" />
      <ServiceCard service-id="minio" label="MinIO" :default-port="9000" />
      <ServiceCard service-id="soketi" label="Soketi" :default-port="6001" />
      <ServiceCard service-id="meilisearch" label="Meilisearch" :default-port="7700" />
    </div>
  </div>
</template>
