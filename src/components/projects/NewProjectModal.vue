<script setup lang="ts">
import { ref, computed, nextTick } from 'vue'
import { useI18n } from 'vue-i18n'
import { Button } from '@/components/ui/button'

const { t } = useI18n()
import { Card, CardHeader, CardContent, CardFooter } from '@/components/ui/card'
import { useProjectsStore } from '@/stores/useProjectsStore'
import { commands } from '@/ipc/bindings'
import { Channel } from '@tauri-apps/api/core'
import type { ScaffoldChunk, ScaffoldTemplate } from '@/ipc/bindings'

const emit = defineEmits<{
  close: []
}>()

const store = useProjectsStore()

const activeTab = ref<'import' | 'scaffold'>('import')

// Import tab state
const importPath = ref('')
const importName = ref('')
const importLoading = ref(false)

async function browseImportFolder() {
  const path = await store.pickFolder()
  if (path) {
    importPath.value = path
    // Auto-fill name from last folder segment
    const parts = path.split('/').filter(Boolean)
    importName.value = parts[parts.length - 1] ?? ''
  }
}

async function handleRegister() {
  if (!importName.value || !importPath.value) return
  importLoading.value = true
  const result = await store.registerProject(importName.value, importPath.value)
  importLoading.value = false
  if (result) emit('close')
}

// Scaffold tab state
const scaffoldTemplate = ref<ScaffoldTemplate>('Tall')
const scaffoldName = ref('')
const scaffoldDest = ref('')
const isScaffolding = ref(false)
const progressLines = ref<string[]>([])
const progressContainer = ref<HTMLElement | null>(null)

async function browseScaffoldDest() {
  const path = await store.pickFolder()
  if (path) scaffoldDest.value = path
}

const scaffoldTemplates: { id: ScaffoldTemplate; label: string; description: string; warning?: string }[] = [
  { id: 'Tall', label: 'TALL Stack', description: 'Tailwind + Alpine + Livewire + Laravel' },
  { id: 'InertiaVue3', label: 'Inertia + Vue 3', description: 'Laravel + Inertia.js + Vue 3' },
  { id: 'InertiaReact', label: 'Inertia + React', description: 'Laravel + Inertia.js + React' },
  { id: 'Filament', label: 'Filament v3', description: 'Laravel + Filament admin panel', warning: 'Requires network access — may take several minutes' },
  { id: 'ApiOnly', label: 'API Only', description: 'Laravel API without frontend (Laravel 11+)' },
  { id: 'Jetstream', label: 'Jetstream (Livewire)', description: 'Laravel Jetstream with Livewire stack', warning: 'Requires network access — may take several minutes' },
]

async function handleScaffold() {
  if (!scaffoldName.value || !scaffoldDest.value) return

  isScaffolding.value = true
  progressLines.value = []

  const channel = new Channel<ScaffoldChunk>()
  channel.onmessage = async (chunk) => {
    progressLines.value.push(chunk.text)
    // Auto-scroll to bottom
    await nextTick()
    if (progressContainer.value) {
      progressContainer.value.scrollTop = progressContainer.value.scrollHeight
    }
    if (chunk.done) {
      isScaffolding.value = false
      if (!chunk.error) {
        const projectPath = `${scaffoldDest.value}/${scaffoldName.value}`
        store.registerProject(scaffoldName.value, projectPath)
        emit('close')
      }
    }
  }

  await commands.scaffoldProject(
    scaffoldTemplate.value,
    scaffoldName.value,
    scaffoldDest.value,
    channel,
  )
}

const tabClass = computed(() => (tab: 'import' | 'scaffold') =>
  activeTab.value === tab
    ? 'border-b-2 border-primary text-primary font-medium'
    : 'text-muted-foreground hover:text-foreground'
)
</script>

<template>
  <!-- Overlay backdrop -->
  <div
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
    @click.self="emit('close')"
  >
    <Card class="w-full max-w-lg mx-4">
      <!-- Header -->
      <CardHeader class="pb-0">
        <div class="flex items-center justify-between">
          <h2 class="text-lg font-semibold">{{ t('newProject.title') }}</h2>
          <button
            class="text-muted-foreground hover:text-foreground transition-colors"
            aria-label="Close"
            @click="emit('close')"
          >
            &#x2715;
          </button>
        </div>

        <!-- Tabs -->
        <div class="flex gap-6 mt-4 border-b">
          <button
            class="pb-2 text-sm transition-colors"
            :class="tabClass('import')"
            @click="activeTab = 'import'"
          >
            {{ t('newProject.tabImport') }}
          </button>
          <button
            class="pb-2 text-sm transition-colors"
            :class="tabClass('scaffold')"
            @click="activeTab = 'scaffold'"
          >
            {{ t('newProject.tabScaffold') }}
          </button>
        </div>
      </CardHeader>

      <CardContent class="pt-4">
        <!-- Import tab -->
        <div v-if="activeTab === 'import'" class="space-y-4">
          <div>
            <label class="text-sm font-medium mb-1 block">{{ t('newProject.folder') }}</label>
            <div class="flex gap-2">
              <input
                :value="importPath"
                readonly
                :placeholder="t('newProject.selectFolder')"
                class="flex-1 rounded-md border border-input bg-background px-3 py-2 text-sm text-muted-foreground"
              />
              <Button variant="outline" size="sm" @click="browseImportFolder">{{ t('common.browse') }}</Button>
            </div>
          </div>

          <div>
            <label class="text-sm font-medium mb-1 block">{{ t('newProject.name') }}</label>
            <input
              v-model="importName"
              type="text"
              :placeholder="t('newProject.namePlaceholder')"
              class="w-full rounded-md border border-input bg-background px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-ring"
            />
          </div>
        </div>

        <!-- Scaffold tab -->
        <div v-else class="space-y-4">
          <!-- Template selector — hidden while scaffolding -->
          <template v-if="!isScaffolding">
            <div>
              <label class="text-sm font-medium mb-2 block">{{ t('newProject.template') }}</label>
              <div class="space-y-2">
                <label
                  v-for="tpl in scaffoldTemplates"
                  :key="tpl.id"
                  class="flex items-start gap-3 rounded-md border p-3 cursor-pointer transition-colors"
                  :class="scaffoldTemplate === tpl.id ? 'border-primary bg-primary/5' : 'border-border hover:border-muted-foreground'"
                >
                  <input
                    type="radio"
                    :value="tpl.id"
                    v-model="scaffoldTemplate"
                    class="mt-0.5"
                  />
                  <div>
                    <p class="text-sm font-medium">{{ tpl.label }}</p>
                    <p class="text-xs text-muted-foreground">{{ tpl.description }}</p>
                    <p v-if="tpl.warning" class="text-xs text-amber-500 mt-0.5">⚠ {{ t('newProject.networkWarning') }}</p>
                  </div>
                </label>
              </div>
            </div>

            <div>
              <label class="text-sm font-medium mb-1 block">{{ t('newProject.name') }}</label>
              <input
                v-model="scaffoldName"
                type="text"
                :placeholder="t('newProject.laravelAppPlaceholder')"
                class="w-full rounded-md border border-input bg-background px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-ring"
              />
            </div>

            <div>
              <label class="text-sm font-medium mb-1 block">{{ t('newProject.destination') }}</label>
              <div class="flex gap-2">
                <input
                  :value="scaffoldDest"
                  readonly
                  :placeholder="t('newProject.selectFolder')"
                  class="flex-1 rounded-md border border-input bg-background px-3 py-2 text-sm text-muted-foreground"
                />
                <Button variant="outline" size="sm" @click="browseScaffoldDest">{{ t('common.browse') }}</Button>
              </div>
            </div>
          </template>

          <!-- Progress panel — shown while scaffolding or if there are lines -->
          <div v-if="isScaffolding || progressLines.length > 0">
            <label class="text-sm font-medium mb-1 block">{{ t('newProject.progress') }}</label>
            <div
              ref="progressContainer"
              class="h-48 overflow-y-auto rounded-md border border-border bg-black/80 p-3 font-mono text-xs text-green-400 space-y-0.5"
            >
              <div v-for="(line, i) in progressLines" :key="i" class="whitespace-pre-wrap break-all">{{ line }}</div>
              <div v-if="isScaffolding" class="animate-pulse text-muted-foreground">{{ t('newProject.running') }}</div>
            </div>
          </div>
        </div>
      </CardContent>

      <CardFooter class="flex justify-end gap-2 pt-2">
        <Button variant="ghost" @click="emit('close')">{{ t('newProject.cancel') }}</Button>
        <Button
          v-if="activeTab === 'import'"
          :disabled="!importName || !importPath || importLoading"
          @click="handleRegister"
        >
          {{ importLoading ? t('newProject.registering') : t('newProject.register') }}
        </Button>
        <Button
          v-else
          :disabled="!scaffoldName || !scaffoldDest || isScaffolding"
          @click="handleScaffold"
        >
          {{ isScaffolding ? t('newProject.scaffolding') : t('newProject.scaffold') }}
        </Button>
      </CardFooter>
    </Card>
  </div>
</template>
