<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRoute, useRouter } from 'vue-router'
import { useProjectsStore } from '@/stores/useProjectsStore'
import { commands } from '@/ipc/bindings'
import type { ArtisanCommand } from '@/ipc/bindings'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import CommandPanel from '@/components/projects/CommandPanel.vue'
import EnvEditor from '@/components/projects/EnvEditor.vue'
import PhpIniEditor from '@/components/projects/PhpIniEditor.vue'
import SupervisorPanel from '@/components/projects/SupervisorPanel.vue'

const { t } = useI18n()
const route = useRoute()
const router = useRouter()
const store = useProjectsStore()

const project = computed(() =>
  store.projects.find((p) => p.id === route.params.id)
)

const catalog = ref<ArtisanCommand[]>([])
const showPanel = ref(false)
const showEnvEditor = ref(false)
const showPhpIni = ref(false)
const showSupervisor = ref(false)

// Supervisor container convention: {project_name}_supervisor_1
const supervisorContainer = computed(() =>
  project.value ? `${project.value.name}_supervisor_1` : ''
)

onMounted(async () => {
  await store.init()
  const result = await commands.listArtisanCommands()
  if (result.status === 'ok') {
    catalog.value = result.data
  }
})
</script>

<template>
  <div class="p-6 flex flex-col h-full relative">
    <!-- Not found -->
    <div v-if="!project" class="flex flex-col items-center justify-center py-24 gap-4">
      <p class="text-lg font-medium text-muted-foreground">{{ t('projects.notFound') }}</p>
      <Button variant="outline" @click="router.push({ name: 'projects' })">
        {{ t('projects.backToProjects') }}
      </Button>
    </div>

    <template v-else>
      <!-- Header -->
      <div class="flex items-center gap-4 mb-6">
        <Button variant="ghost" size="sm" @click="router.push({ name: 'projects' })">
          {{ t('projects.back') }}
        </Button>
        <h1 class="text-2xl font-semibold">{{ project.name }}</h1>
        <Badge variant="secondary">{{ t('projects.badge') }}</Badge>
      </div>

      <!-- Info -->
      <div class="rounded-md border p-4 mb-6 space-y-2">
        <div class="flex items-start gap-2">
          <span class="text-sm text-muted-foreground w-20 shrink-0">{{ t('projects.path') }}</span>
          <span class="text-sm font-mono truncate" :title="project.path">{{ project.path }}</span>
        </div>
        <div class="flex items-center gap-2">
          <span class="text-sm text-muted-foreground w-20 shrink-0">{{ t('projects.viteAuto') }}</span>
          <Badge :variant="project.vite_auto ? 'default' : 'secondary'">
            {{ project.vite_auto ? t('projects.enabled') : t('projects.disabled') }}
          </Badge>
        </div>
      </div>

      <!-- Actions row -->
      <div class="mb-4 flex items-center gap-2">
        <Button variant="outline" @click="showPanel = true">
          {{ t('projects.openTerminal') }}
        </Button>
        <Button variant="outline" @click="showEnvEditor = !showEnvEditor">
          {{ t('projects.editEnv') }}
        </Button>
        <Button variant="outline" @click="showPhpIni = !showPhpIni">
          {{ t('projects.editPhpIni') }}
        </Button>
        <Button variant="outline" @click="showSupervisor = !showSupervisor">
          {{ showSupervisor ? t('projects.hideSupervisor') : t('projects.showSupervisor') }}
        </Button>
      </div>

      <!-- .env Editor — v-if so teardown clears state -->
      <div v-if="showEnvEditor" class="mb-4">
        <EnvEditor :project-id="project.id" />
      </div>

      <!-- php.ini Editor — v-if so teardown clears state -->
      <div v-if="showPhpIni" class="mb-4">
        <PhpIniEditor :project-id="project.id" />
      </div>

      <!-- Supervisor Panel — v-if so teardown clears state -->
      <div v-if="showSupervisor" class="mb-4">
        <SupervisorPanel
          :project-id="project.id"
          :supervisor-container="supervisorContainer"
        />
      </div>

      <!-- Command Panel — v-if so teardown clears the stream -->
      <CommandPanel
        v-if="showPanel"
        :project="project"
        :artisan-commands="catalog"
        @close="showPanel = false"
      />
    </template>
  </div>
</template>
