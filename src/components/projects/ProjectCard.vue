<script setup lang="ts">
import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRouter } from 'vue-router'
import { Card, CardHeader, CardContent, CardFooter } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import type { ProjectConfig, ProjectStatus } from '@/ipc/bindings'
import DatabaseTab from '@/components/database/DatabaseTab.vue'
import ComposeEditor from '@/components/compose/ComposeEditor.vue'

const { t } = useI18n()
const router = useRouter()

const props = defineProps<{
  project: ProjectConfig
  status?: ProjectStatus
  opening?: boolean
  starting?: boolean
  stopping?: boolean
}>()

const emit = defineEmits<{
  start: []
  stop: []
  terminal: []
  open: []
}>()

// Database tab toggle — DatabaseTab component implemented in 04-04
const showDatabaseTab = ref(false)

// Compose editor drawer toggle
const showComposeEditor = ref(false)

const projectStatusString = computed(() => {
  if (!props.status) return 'stopped'
  return props.status.kind
})

const badgeVariant = computed(() => {
  if (!props.status) return 'secondary' as const
  if (props.status.kind === 'running') return 'default' as const
  if (props.status.kind === 'unhealthy') return 'destructive' as const
  return 'secondary' as const
})

const statusLabel = computed(() => {
  if (!props.status) return t('projects.status.stopped')
  if (props.status.kind === 'running') return t('projects.status.running')
  if (props.status.kind === 'partially_running') return t('projects.status.partial')
  if (props.status.kind === 'unhealthy') return t('projects.status.unhealthy')
  return t('projects.status.stopped')
})
</script>

<template>
  <Card class="flex flex-col">
    <CardHeader class="pb-2">
      <div class="flex items-center justify-between gap-2">
        <span class="font-semibold text-base truncate">{{ project.name }}</span>
        <Badge :variant="badgeVariant">{{ statusLabel }}</Badge>
      </div>
    </CardHeader>

    <CardContent class="flex-1 pb-2">
      <p class="text-sm text-muted-foreground truncate" :title="project.path">
        {{ project.path }}
      </p>
    </CardContent>

    <CardFooter class="flex gap-2 flex-wrap pt-2">
      <Button
        size="sm"
        variant="outline"
        :disabled="opening || starting || stopping"
        @click="emit('open')"
      >
        <span v-if="opening" class="mr-1.5 h-3 w-3 animate-spin rounded-full border-2 border-current border-t-transparent" />
        {{ opening ? t('projects.actions.opening') : t('common.browse') }}
      </Button>
      <Button
        size="sm"
        variant="outline"
        :disabled="opening || starting || stopping"
        @click="emit('start')"
      >
        <span v-if="starting" class="mr-1.5 h-3 w-3 animate-spin rounded-full border-2 border-current border-t-transparent" />
        {{ starting ? t('projects.actions.starting') : t('projects.actions.start') }}
      </Button>
      <Button
        size="sm"
        variant="outline"
        :disabled="opening || starting || stopping"
        @click="emit('stop')"
      >
        <span v-if="stopping" class="mr-1.5 h-3 w-3 animate-spin rounded-full border-2 border-current border-t-transparent" />
        {{ stopping ? t('projects.actions.stopping') : t('projects.actions.stop') }}
      </Button>
      <Button
        size="sm"
        variant="outline"
        :disabled="opening || starting || stopping"
        @click="router.push({ name: 'project-detail', params: { id: project.id } })"
      >
        {{ t('projects.actions.terminal') }}
      </Button>
      <Button
        size="sm"
        :variant="showDatabaseTab ? 'secondary' : 'outline'"
        :disabled="opening || starting || stopping"
        @click="showDatabaseTab = !showDatabaseTab"
      >
        {{ t('projects.actions.database') }}
      </Button>
      <Button
        size="sm"
        variant="outline"
        :disabled="opening || starting || stopping"
        @click="showComposeEditor = true"
      >
        {{ t('projects.actions.editCompose') }}
      </Button>
    </CardFooter>

    <!-- Database tab panel (v-if so Vue teardown clears Channel listeners) -->
    <div v-if="showDatabaseTab" class="border-t">
      <DatabaseTab :project-id="project.id" :project-name="project.name" />
    </div>
  </Card>

  <!-- Compose editor right-side drawer (mounted only when open) -->
  <ComposeEditor
    v-if="showComposeEditor"
    :project-id="project.id"
    :project-status="projectStatusString"
    @close="showComposeEditor = false"
    @restart="emit('start')"
  />
</template>
