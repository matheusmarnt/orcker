<script setup lang="ts">
import { computed } from 'vue'
import { useRouter } from 'vue-router'
import { Card, CardHeader, CardContent, CardFooter } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import type { ProjectConfig } from '@/ipc/bindings'

const router = useRouter()

export type ProjectStatus = { kind: 'running' } | { kind: 'stopped' } | { kind: 'error'; message: string }

const props = defineProps<{
  project: ProjectConfig
  status?: ProjectStatus
}>()

const emit = defineEmits<{
  start: []
  stop: []
  terminal: []
  open: []
}>()

const badgeVariant = computed(() => {
  if (!props.status) return 'secondary' as const
  if (props.status.kind === 'running') return 'default' as const
  if (props.status.kind === 'error') return 'destructive' as const
  return 'secondary' as const
})

const statusLabel = computed(() => {
  if (!props.status) return 'Stopped'
  if (props.status.kind === 'running') return 'Running'
  if (props.status.kind === 'error') return 'Error'
  return 'Stopped'
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
      <Button size="sm" variant="outline" @click="emit('open')">Open</Button>
      <Button size="sm" variant="outline" @click="emit('start')">Start</Button>
      <Button size="sm" variant="outline" @click="emit('stop')">Stop</Button>
      <Button size="sm" variant="outline" @click="router.push({ name: 'project-detail', params: { id: project.id } })">Terminal</Button>
    </CardFooter>
  </Card>
</template>
