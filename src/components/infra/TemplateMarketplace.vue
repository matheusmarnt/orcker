<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { toast } from 'vue-sonner'
import { commands, type TemplateEntry } from '@/ipc/bindings'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from '@/components/ui/card'
import { Skeleton } from '@/components/ui/skeleton'

const templates = ref<TemplateEntry[]>([])
const isLoading = ref(true)
const error = ref<string | null>(null)
const activeTag = ref<string | null>(null)
const installing = ref<Set<string>>(new Set())

const allTags = computed(() => {
  const tags = new Set<string>()
  for (const t of templates.value) {
    for (const tag of t.tags) tags.add(tag)
  }
  return Array.from(tags).sort()
})

const filteredTemplates = computed(() => {
  if (!activeTag.value) return templates.value
  return templates.value.filter(t => t.tags.includes(activeTag.value!))
})

function setTag(tag: string | null) {
  activeTag.value = tag
}

async function loadTemplates() {
  isLoading.value = true
  error.value = null
  const result = await commands.fetchTemplateManifest()
  isLoading.value = false
  if (result.status === 'ok') {
    templates.value = result.data
  } else {
    error.value = 'No templates available — check your network connection'
  }
}

async function install(template: TemplateEntry) {
  installing.value = new Set([...installing.value, template.id])
  try {
    const result = await commands.installTemplate(template.compose_url)
    if (result.status === 'ok') {
      toast.success(`Template installed to ${result.data}`)
    } else {
      toast.error(`Install failed: ${String(result.error)}`)
    }
  } catch (e) {
    toast.error(`Install failed: ${e instanceof Error ? e.message : String(e)}`)
  } finally {
    const next = new Set(installing.value)
    next.delete(template.id)
    installing.value = next
  }
}

onMounted(loadTemplates)
</script>

<template>
  <div>
    <div class="mb-4 flex items-center justify-between">
      <div>
        <h3 class="text-lg font-semibold">Template Marketplace</h3>
        <p class="text-sm text-muted-foreground">Browse and install docker-compose templates</p>
      </div>
    </div>

    <!-- Loading skeletons -->
    <template v-if="isLoading">
      <div class="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
        <Card v-for="i in 3" :key="i">
          <CardHeader>
            <Skeleton class="h-5 w-32" />
            <Skeleton class="h-4 w-48 mt-1" />
          </CardHeader>
          <CardContent>
            <Skeleton class="h-4 w-full" />
            <Skeleton class="h-4 w-3/4 mt-2" />
          </CardContent>
          <CardFooter>
            <Skeleton class="h-9 w-20" />
          </CardFooter>
        </Card>
      </div>
    </template>

    <!-- Error / empty state -->
    <template v-else-if="error || templates.length === 0">
      <div class="rounded-lg border border-dashed p-8 text-center text-sm text-muted-foreground">
        {{ error ?? 'No templates available — check your network connection' }}
      </div>
    </template>

    <!-- Tag filter + card grid -->
    <template v-else>
      <div class="mb-4 flex flex-wrap gap-2">
        <button
          class="rounded-full border px-3 py-1 text-xs transition-colors"
          :class="activeTag === null ? 'bg-primary text-primary-foreground' : 'hover:bg-muted'"
          @click="setTag(null)"
        >
          All
        </button>
        <button
          v-for="tag in allTags"
          :key="tag"
          class="rounded-full border px-3 py-1 text-xs transition-colors"
          :class="activeTag === tag ? 'bg-primary text-primary-foreground' : 'hover:bg-muted'"
          @click="setTag(tag)"
        >
          {{ tag }}
        </button>
      </div>

      <div class="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
        <Card v-for="template in filteredTemplates" :key="template.id">
          <CardHeader>
            <CardTitle class="text-base">{{ template.name }}</CardTitle>
            <CardDescription>{{ template.description }}</CardDescription>
          </CardHeader>
          <CardContent>
            <div class="flex flex-wrap gap-1">
              <Badge v-for="tag in template.tags" :key="tag" variant="secondary" class="text-xs">
                {{ tag }}
              </Badge>
            </div>
          </CardContent>
          <CardFooter>
            <Button
              size="sm"
              :disabled="installing.has(template.id)"
              @click="install(template)"
            >
              {{ installing.has(template.id) ? 'Installing...' : 'Install' }}
            </Button>
          </CardFooter>
        </Card>
      </div>
    </template>
  </div>
</template>
