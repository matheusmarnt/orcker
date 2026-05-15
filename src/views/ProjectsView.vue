<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useProjectsStore } from '@/stores/useProjectsStore'
import { commands } from '@/ipc/bindings'
import { Button } from '@/components/ui/button'
import { Skeleton } from '@/components/ui/skeleton'
import ProjectCard from '@/components/projects/ProjectCard.vue'
import NewProjectModal from '@/components/projects/NewProjectModal.vue'

const store = useProjectsStore()
const showModal = ref(false)

onMounted(() => {
  store.init()
})

async function openFolder(projectId: string) {
  try {
    await commands.openProjectFolder(projectId)
  } catch (e) {
    console.error('Failed to open folder:', e)
  }
}

function notImplemented(action: string) {
  // Start/Stop requires docker compose — inform user
  console.info(`${action}: run 'sail up -d' or 'docker compose up -d' in the project directory`)
}
</script>

<template>
  <div class="p-6">
    <!-- Header -->
    <div class="flex items-center justify-between mb-6">
      <h1 class="text-2xl font-semibold">Projects</h1>
      <Button v-if="!store.loading && store.projects.length > 0" @click="showModal = true">
        + New Project
      </Button>
    </div>

    <!-- Skeleton loading state -->
    <div v-if="store.loading" class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
      <div v-for="n in 3" :key="n" class="rounded-lg border p-4 space-y-3">
        <Skeleton class="h-5 w-2/3" />
        <Skeleton class="h-4 w-full" />
        <div class="flex gap-2 pt-2">
          <Skeleton class="h-8 w-16" />
          <Skeleton class="h-8 w-16" />
          <Skeleton class="h-8 w-16" />
        </div>
      </div>
    </div>

    <!-- Empty state -->
    <div
      v-else-if="store.projects.length === 0"
      class="flex flex-col items-center justify-center py-24 gap-4 text-center"
    >
      <p class="text-4xl">🗂</p>
      <p class="text-lg font-medium">No projects yet</p>
      <p class="text-sm text-muted-foreground max-w-xs">
        Import an existing Laravel project or scaffold a new one.
      </p>
      <Button @click="showModal = true">+ New Project</Button>
    </div>

    <!-- Projects grid -->
    <div v-else class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
      <ProjectCard
        v-for="project in store.projects"
        :key="project.id"
        :project="project"
        @open="openFolder(project.id)"
        @start="notImplemented('Start')"
        @stop="notImplemented('Stop')"
        @terminal="() => {}"
      />
    </div>

    <!-- New Project Modal -->
    <NewProjectModal v-if="showModal" @close="showModal = false" />
  </div>
</template>
