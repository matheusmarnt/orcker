<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { commands } from '@/ipc/bindings'
import type { SupervisorWorker } from '@/ipc/bindings'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { toast } from 'vue-sonner'

const props = defineProps<{
  projectId: string
  supervisorContainer: string
}>()

const workers = ref<SupervisorWorker[]>([])
const loading = ref(false)
const restarting = ref<Set<string>>(new Set())

const statusVariant = (status: string): 'default' | 'secondary' | 'destructive' => {
  const s = status.toUpperCase()
  if (s === 'RUNNING') return 'default'
  if (s === 'STOPPED' || s === 'EXITED') return 'secondary'
  return 'destructive'
}

async function load() {
  loading.value = true
  const result = await commands.listSupervisorWorkers(props.supervisorContainer)
  loading.value = false
  if (result.status === 'ok') {
    workers.value = result.data
  } else {
    toast.error('Failed to list workers', { description: String(result.error) })
  }
}

async function restart(workerName: string) {
  restarting.value = new Set([...restarting.value, workerName])
  const result = await commands.restartSupervisorWorker(props.supervisorContainer, workerName)
  restarting.value = new Set([...restarting.value].filter((n) => n !== workerName))
  if (result.status === 'ok') {
    toast.success(`Restarted ${workerName}`)
    await load()
  } else {
    toast.error(`Failed to restart ${workerName}`, { description: String(result.error) })
  }
}

onMounted(load)
</script>

<template>
  <div class="rounded-md border p-4 space-y-4">
    <div class="flex items-center justify-between">
      <h3 class="text-sm font-semibold">Supervisor Workers</h3>
      <Button variant="outline" size="sm" :disabled="loading" @click="load">
        {{ loading ? 'Loading…' : 'Refresh' }}
      </Button>
    </div>

    <div v-if="loading && workers.length === 0" class="text-sm text-muted-foreground py-4 text-center">
      Loading workers…
    </div>

    <div v-else-if="workers.length === 0" class="text-sm text-muted-foreground py-4 text-center">
      No workers found. Is the supervisor container running?
    </div>

    <ul v-else class="space-y-2">
      <li
        v-for="worker in workers"
        :key="worker.name"
        class="flex items-center justify-between rounded-md border px-3 py-2"
      >
        <div class="flex items-center gap-3">
          <span class="font-mono text-sm">{{ worker.name }}</span>
          <Badge :variant="statusVariant(worker.status)">{{ worker.status }}</Badge>
        </div>
        <Button
          variant="outline"
          size="sm"
          :disabled="restarting.has(worker.name)"
          @click="restart(worker.name)"
        >
          {{ restarting.has(worker.name) ? 'Restarting…' : 'Restart' }}
        </Button>
      </li>
    </ul>
  </div>
</template>
