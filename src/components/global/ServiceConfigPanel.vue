<script setup lang="ts">
import { ref, watch, onMounted } from 'vue'
import { Button } from '@/components/ui/button'
import { useGlobalStackStore } from '@/stores/useGlobalStackStore'
import type { ServiceId } from '@/ipc/bindings'

const DEFAULTS: Record<ServiceId, { image_tag: string; port: number }> = {
  redis: { image_tag: 'redis:7-alpine', port: 6379 },
  postgres: { image_tag: 'postgres:16-alpine', port: 5432 },
  mailpit: { image_tag: 'axllent/mailpit:latest', port: 8025 },
}

const props = defineProps<{ serviceId: ServiceId }>()

const store = useGlobalStackStore()

const imageTag = ref('')
const port = ref(0)

function seedFromStore() {
  const cfg = store.configs[props.serviceId]
  if (cfg) {
    imageTag.value = cfg.image_tag
    port.value = cfg.port
  } else {
    imageTag.value = DEFAULTS[props.serviceId].image_tag
    port.value = DEFAULTS[props.serviceId].port
  }
}

onMounted(seedFromStore)
watch(() => store.configs[props.serviceId], seedFromStore, { deep: true })

async function applyConfig() {
  await store.setConfig(props.serviceId, { image_tag: imageTag.value, port: port.value })
}
</script>

<template>
  <div class="mt-2 rounded-md border border-border bg-muted/40 p-4">
    <p class="mb-3 text-xs font-semibold uppercase tracking-wide text-muted-foreground">
      Configuration
    </p>

    <div class="flex flex-col gap-3">
      <div>
        <label class="mb-1 block text-xs text-muted-foreground" :for="`image-tag-${serviceId}`">
          Image tag
        </label>
        <input
          :id="`image-tag-${serviceId}`"
          v-model="imageTag"
          type="text"
          class="w-full rounded-md border border-input bg-background px-3 py-1.5 text-sm focus:outline-none focus:ring-2 focus:ring-ring"
        />
      </div>

      <div>
        <label class="mb-1 block text-xs text-muted-foreground" :for="`port-${serviceId}`">
          Port
        </label>
        <input
          :id="`port-${serviceId}`"
          v-model.number="port"
          type="number"
          min="1"
          max="65535"
          class="w-full rounded-md border border-input bg-background px-3 py-1.5 text-sm focus:outline-none focus:ring-2 focus:ring-ring"
        />
      </div>

      <Button
        size="sm"
        :disabled="store.isTransitioning(serviceId)"
        @click="applyConfig"
      >
        Apply
      </Button>
    </div>
  </div>
</template>
