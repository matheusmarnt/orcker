<script setup lang="ts">
import { ref, watch, onUnmounted } from 'vue'
import { useProjectsStore } from '@/stores/useProjectsStore'
import { useLogsStore } from '@/stores/useLogsStore'
import type { LogSource } from '@/stores/useLogsStore'
import LogViewer from '@/components/logs/LogViewer.vue'
import LogFilterBar from '@/components/logs/LogFilterBar.vue'

const projectsStore = useProjectsStore()
const logsStore = useLogsStore()

const SOURCES: Array<'All' | LogSource> = ['All', 'Docker', 'Laravel', 'Nginx', 'Supervisor']

const selectedProjectId = ref<string>('')
const isStreaming = ref(false)

function appContainerName(projectName: string): string {
  return `${projectName.toLowerCase().replace(/\s+/g, '_')}_app_1`
}

async function onProjectChange(newId: string): Promise<void> {
  if (isStreaming.value && selectedProjectId.value) {
    await logsStore.stopStream(selectedProjectId.value)
    isStreaming.value = false
  }
  selectedProjectId.value = newId
  if (!newId) return

  const project = projectsStore.projects.find(p => p.id === newId)
  if (!project) return

  isStreaming.value = true
  await logsStore.startStream(project.id, project.path, appContainerName(project.name))
}

watch(selectedProjectId, async (newId, oldId) => {
  if (oldId && oldId !== newId && isStreaming.value) {
    await logsStore.stopStream(oldId)
    isStreaming.value = false
  }
})

onUnmounted(async () => {
  if (selectedProjectId.value && isStreaming.value) {
    await logsStore.stopStream(selectedProjectId.value)
  }
})
</script>

<template>
  <div class="flex flex-col h-full">
    <!-- Header: project selector + source tabs -->
    <div class="flex flex-col gap-2 px-4 pt-4 pb-2 border-b">
      <div class="flex items-center gap-3">
        <h1 class="text-lg font-semibold shrink-0">Logs</h1>
        <select
          :value="selectedProjectId"
          class="h-8 text-sm border rounded px-2 bg-background flex-1 max-w-xs"
          @change="onProjectChange(($event.target as HTMLSelectElement).value)"
        >
          <option value="">Select project...</option>
          <option
            v-for="project in projectsStore.projects"
            :key="project.id"
            :value="project.id"
          >
            {{ project.name }}
          </option>
        </select>
      </div>

      <!-- Source tabs -->
      <div class="flex gap-1">
        <button
          v-for="source in SOURCES"
          :key="source"
          class="px-3 py-1 text-xs rounded border transition-colors"
          :class="logsStore.activeSource === source
            ? 'bg-primary text-primary-foreground border-primary'
            : 'bg-background text-muted-foreground border-border hover:text-foreground'"
          @click="logsStore.activeSource = source"
        >
          {{ source }}
        </button>
      </div>
    </div>

    <!-- Filter bar: always visible -->
    <LogFilterBar />

    <!-- Log panel -->
    <div v-if="isStreaming" class="flex-1 overflow-hidden">
      <LogViewer />
    </div>
    <div v-else class="flex-1 flex items-center justify-center text-muted-foreground text-sm">
      Select a project to start streaming logs
    </div>
  </div>
</template>
