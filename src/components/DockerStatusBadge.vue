<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { onClickOutside } from '@vueuse/core'
import { CircleDot } from 'lucide-vue-next'
import { useDockerStore } from '../stores/docker'
import { getDockerVersion } from '../ipc/docker'

const { t } = useI18n()
const docker = useDockerStore()

const panelOpen = ref(false)
const panelRef = ref<HTMLElement | null>(null)
const latencyMs = ref<number | null>(null)

onClickOutside(panelRef, () => {
  panelOpen.value = false
})

const dotColor = computed(() => {
  switch (docker.connectionStatus) {
    case 'connected':
      return 'bg-green-500'
    case 'error':
      return 'bg-red-500'
    case 'connecting':
    default:
      return 'bg-muted-foreground'
  }
})

const label = computed(() => {
  switch (docker.connectionStatus) {
    case 'connected':
      return t('docker.connected')
    case 'error':
      return t('docker.disconnected')
    case 'connecting':
    default:
      return t('docker.connecting')
  }
})

const runningCount = computed(() => {
  return Array.from(docker.containers.values()).filter(
    (c) => c.status.toLowerCase().startsWith('up') || c.status.toLowerCase() === 'running',
  ).length
})

const stoppedCount = computed(() => {
  return docker.containers.size - runningCount.value
})

function togglePanel() {
  panelOpen.value = !panelOpen.value
}

let latencyInterval: ReturnType<typeof setInterval> | null = null

async function measureLatency() {
  if (docker.connectionStatus !== 'connected') return
  const start = Date.now()
  try {
    await getDockerVersion()
    latencyMs.value = Date.now() - start
  } catch {
    latencyMs.value = null
  }
}

onMounted(() => {
  measureLatency()
  latencyInterval = setInterval(measureLatency, 10_000)
})

onUnmounted(() => {
  if (latencyInterval !== null) {
    clearInterval(latencyInterval)
  }
})
</script>

<template>
  <div class="relative" ref="panelRef">
    <!-- Badge button -->
    <button
      type="button"
      class="flex w-full cursor-pointer items-center gap-2 rounded-md px-2 py-1.5 text-xs text-muted-foreground transition-colors hover:bg-accent hover:text-accent-foreground"
      @click="togglePanel"
      :aria-expanded="panelOpen"
    >
      <span :class="['h-2 w-2 flex-shrink-0 rounded-full', dotColor]" />
      <span class="truncate">{{ label }}</span>
    </button>

    <!-- Mini panel (absolute, above badge) -->
    <div
      v-if="panelOpen"
      class="absolute bottom-full left-0 mb-2 w-64 rounded-md border border-border bg-card p-3 shadow-lg"
    >
      <div class="flex flex-col gap-2 text-xs">
        <div class="font-semibold text-foreground">{{ t('docker.info') }}</div>

        <div class="flex items-center justify-between gap-2 text-muted-foreground">
          <span>{{ t('docker.version') }}</span>
          <span class="font-mono text-foreground">{{ docker.dockerVersion ?? '—' }}</span>
        </div>

        <div class="flex items-start justify-between gap-2 text-muted-foreground">
          <span class="flex-shrink-0">{{ t('docker.socket') }}</span>
          <span class="break-all text-right font-mono text-foreground">
            {{ docker.socketPath ?? '—' }}
          </span>
        </div>

        <div class="flex items-center justify-between gap-2 text-muted-foreground">
          <span>{{ t('docker.containers') }}</span>
          <span class="text-foreground">
            {{ runningCount }} {{ t('containerTable.statuses.running').toLowerCase() }}, {{ stoppedCount }} {{ t('containerTable.statuses.stopped').toLowerCase() }}
          </span>
        </div>

        <div class="flex items-center justify-between gap-2 text-muted-foreground">
          <span>{{ t('docker.latency') }}</span>
          <span class="text-foreground">
            {{ latencyMs !== null ? `~${latencyMs}ms` : '—' }}
          </span>
        </div>

        <div class="flex items-center gap-1.5 text-muted-foreground">
          <CircleDot class="h-3 w-3" :class="dotColor.replace('bg-', 'text-')" />
          <span class="capitalize">{{ docker.connectionStatus }}</span>
        </div>
      </div>
    </div>
  </div>
</template>
