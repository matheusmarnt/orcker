<script setup lang="ts">
import { ref, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { Card, CardHeader, CardContent } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Switch } from '@/components/ui/switch'
import ServiceConfigPanel from './ServiceConfigPanel.vue'
import { useGlobalStackStore } from '@/stores/useGlobalStackStore'
import type { ServiceId } from '@/ipc/bindings'

const props = defineProps<{
  serviceId: ServiceId
  label: string
  defaultPort: number
}>()

const { t } = useI18n()
const store = useGlobalStackStore()
const configOpen = ref(false)

// ---------------------------------------------------------------------------
// Badge
// ---------------------------------------------------------------------------

const badgeVariant = computed(() => {
  const kind = store.statuses[props.serviceId]?.kind ?? 'stopped'
  if (kind === 'running') return 'default'
  if (kind === 'stopped') return 'secondary'
  if (kind === 'starting' || kind === 'stopping') return 'outline'
  if (kind === 'error' || kind === 'unhealthy') return 'destructive'
  return 'secondary'
})

const badgeClass = computed(() => {
  const kind = store.statuses[props.serviceId]?.kind ?? 'stopped'
  if (kind === 'running') return 'bg-green-600 text-white'
  if (kind === 'starting' || kind === 'stopping') return 'text-yellow-500'
  if (kind === 'unhealthy') return 'bg-orange-500 text-white'
  return ''
})

const badgeText = computed(() => {
  const status = store.statuses[props.serviceId]
  if (!status) return t('global.status.stopped')
  switch (status.kind) {
    case 'running': return t('global.status.running')
    case 'stopped': return t('global.status.stopped')
    case 'starting': return t('global.status.starting')
    case 'stopping': return t('global.status.stopping')
    case 'unhealthy': return t('global.status.unhealthy')
    case 'error': return t('global.status.error')
    default: return t('global.status.stopped')
  }
})

// ---------------------------------------------------------------------------
// Port display
// ---------------------------------------------------------------------------

const activePort = computed(
  () => store.configs[props.serviceId]?.port ?? props.defaultPort,
)
</script>

<template>
  <Card class="flex flex-col">
    <CardHeader class="pb-2">
      <div class="flex items-center justify-between">
        <span class="text-sm font-semibold">{{ label }}</span>
        <Switch
          :model-value="store.isRunning(serviceId)"
          :disabled="store.isTransitioning(serviceId)"
          @update:model-value="store.toggleService(serviceId)"
        />
      </div>

      <div class="mt-1 flex items-center gap-2">
        <Badge :variant="badgeVariant" :class="badgeClass">
          {{ badgeText }}
        </Badge>
        <span class="text-xs text-muted-foreground">:{{ activePort }}</span>
      </div>
    </CardHeader>

    <CardContent class="flex flex-col gap-2 pb-4">
      <!-- Restart required warning -->
      <span
        v-if="store.restartRequired[serviceId]"
        class="text-xs text-yellow-600"
      >
        {{ t('global.restartRequired') }}
      </span>

      <!-- Config toggle -->
      <button
        class="self-start text-xs text-muted-foreground underline-offset-2 hover:underline"
        @click="configOpen = !configOpen"
      >
        {{ configOpen ? t('global.hideConfig') : t('global.configure') }}
      </button>

      <!-- Config panel (expand in-place) -->
      <div v-if="configOpen">
        <ServiceConfigPanel :service-id="serviceId" />
      </div>
    </CardContent>
  </Card>
</template>
