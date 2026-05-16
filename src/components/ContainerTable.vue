<script setup lang="ts">
import { ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { Badge } from './ui/badge'
import type { ContainerSummary } from '../stores/docker'

const { t } = useI18n()

const props = defineProps<{
  containers: ContainerSummary[]
}>()

// Track previous statuses per container ID to trigger row highlight on change
const prevStatuses = ref<Map<string, string>>(new Map())
const highlightedIds = ref<Set<string>>(new Set())

watch(
  () => props.containers,
  (next) => {
    const newHighlights = new Set<string>()
    for (const container of next) {
      const prev = prevStatuses.value.get(container.id)
      if (prev !== undefined && prev !== container.status) {
        newHighlights.add(container.id)
      }
    }

    if (newHighlights.size > 0) {
      highlightedIds.value = newHighlights

      setTimeout(() => {
        for (const id of newHighlights) {
          highlightedIds.value.delete(id)
        }
        // Force reactivity
        highlightedIds.value = new Set(highlightedIds.value)
      }, 1500)
    }

    // Update previous statuses map
    const nextMap = new Map<string, string>()
    for (const c of next) {
      nextMap.set(c.id, c.status)
    }
    prevStatuses.value = nextMap
  },
  { deep: true },
)

function statusVariant(status: string): 'default' | 'secondary' | 'destructive' | 'outline' {
  const s = status.toLowerCase()
  if (s.startsWith('up') || s === 'running') return 'default'
  if (s === 'paused') return 'secondary'
  if (s.startsWith('exit') || s === 'stopped' || s === 'dead') return 'destructive'
  return 'outline'
}

function statusLabel(status: string): string {
  const s = status.toLowerCase()
  if (s.startsWith('up') || s === 'running') return t('containerTable.statuses.running')
  if (s === 'paused') return t('containerTable.statuses.paused')
  if (s.startsWith('exit') || s === 'stopped') return t('containerTable.statuses.stopped')
  if (s === 'dead') return t('containerTable.statuses.dead')
  return status
}
</script>

<template>
  <div class="overflow-x-auto rounded-md border border-border">
    <table class="w-full text-sm">
      <thead>
        <tr class="border-b border-border bg-muted/50">
          <th class="px-4 py-2 text-left font-medium text-muted-foreground">{{ t('containerTable.name') }}</th>
          <th class="px-4 py-2 text-left font-medium text-muted-foreground">{{ t('containerTable.status') }}</th>
          <th class="px-4 py-2 text-left font-medium text-muted-foreground">{{ t('containerTable.image') }}</th>
          <th class="px-4 py-2 text-left font-medium text-muted-foreground">{{ t('containerTable.ports') }}</th>
        </tr>
      </thead>
      <tbody>
        <!-- Empty state -->
        <tr v-if="containers.length === 0">
          <td colspan="4" class="px-4 py-8 text-center text-muted-foreground">
            {{ t('containerTable.empty') }}
          </td>
        </tr>

        <!-- Container rows -->
        <tr
          v-for="container in containers"
          :key="container.id"
          class="border-b border-border last:border-0 transition-colors duration-300"
          :class="{
            'bg-yellow-50 dark:bg-yellow-900/20': highlightedIds.has(container.id),
          }"
        >
          <td class="px-4 py-2 font-mono text-xs">
            {{ container.name }}
          </td>
          <td class="px-4 py-2">
            <Badge :variant="statusVariant(container.status)">
              {{ statusLabel(container.status) }}
            </Badge>
          </td>
          <td class="px-4 py-2 font-mono text-xs text-muted-foreground">
            {{ container.image }}
          </td>
          <td class="px-4 py-2 font-mono text-xs text-muted-foreground">
            {{ container.ports || '—' }}
          </td>
        </tr>
      </tbody>
    </table>
  </div>
</template>
