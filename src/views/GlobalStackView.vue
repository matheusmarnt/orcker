<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { listen } from '@tauri-apps/api/event'
import { Skeleton } from '@/components/ui/skeleton'
import ServiceCard from '@/components/global/ServiceCard.vue'
import { useGlobalStackStore } from '@/stores/useGlobalStackStore'

const store = useGlobalStackStore()
const loading = ref(true)
let unlistenShortcut: (() => void) | null = null

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
      <h1 class="text-xl font-bold">Global Stack</h1>
      <p class="mt-1 text-sm text-muted-foreground">
        Shared services for all Laravel projects
      </p>
    </div>

    <!-- Skeleton during initial load -->
    <div v-if="loading" class="flex gap-4">
      <Skeleton v-for="i in 3" :key="i" class="h-40 flex-1 rounded-lg" />
    </div>

    <!-- Service cards -->
    <div v-else class="flex gap-4">
      <ServiceCard
        service-id="redis"
        label="Redis"
        :default-port="6379"
        class="flex-1"
      />
      <ServiceCard
        service-id="postgres"
        label="PostgreSQL"
        :default-port="5432"
        class="flex-1"
      />
      <ServiceCard
        service-id="mailpit"
        label="Mailpit"
        :default-port="8025"
        class="flex-1"
      />
    </div>
  </div>
</template>
