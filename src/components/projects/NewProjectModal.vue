<script setup lang="ts">
import { ref, computed } from 'vue'
import { Button } from '@/components/ui/button'
import { Card, CardHeader, CardContent, CardFooter } from '@/components/ui/card'
import { useProjectsStore } from '@/stores/useProjectsStore'

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
const scaffoldTemplate = ref<'tall' | 'inertia-vue' | 'inertia-react'>('tall')
const scaffoldName = ref('')
const scaffoldDest = ref('')

async function browseScaffoldDest() {
  const path = await store.pickFolder()
  if (path) scaffoldDest.value = path
}

const scaffoldTemplates = [
  { id: 'tall' as const, label: 'TALL Stack', description: 'Tailwind + Alpine + Livewire + Laravel' },
  { id: 'inertia-vue' as const, label: 'Inertia + Vue 3', description: 'Laravel + Inertia.js + Vue 3' },
  { id: 'inertia-react' as const, label: 'Inertia + React', description: 'Laravel + Inertia.js + React' },
]

function handleScaffold() {
  // Placeholder — scaffold command implemented in a later plan
  emit('close')
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
          <h2 class="text-lg font-semibold">New Project</h2>
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
            Import
          </button>
          <button
            class="pb-2 text-sm transition-colors"
            :class="tabClass('scaffold')"
            @click="activeTab = 'scaffold'"
          >
            Scaffold
          </button>
        </div>
      </CardHeader>

      <CardContent class="pt-4">
        <!-- Import tab -->
        <div v-if="activeTab === 'import'" class="space-y-4">
          <div>
            <label class="text-sm font-medium mb-1 block">Folder</label>
            <div class="flex gap-2">
              <input
                :value="importPath"
                readonly
                placeholder="Select a folder..."
                class="flex-1 rounded-md border border-input bg-background px-3 py-2 text-sm text-muted-foreground"
              />
              <Button variant="outline" size="sm" @click="browseImportFolder">Browse</Button>
            </div>
          </div>

          <div>
            <label class="text-sm font-medium mb-1 block">Project Name</label>
            <input
              v-model="importName"
              type="text"
              placeholder="my-app"
              class="w-full rounded-md border border-input bg-background px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-ring"
            />
          </div>
        </div>

        <!-- Scaffold tab -->
        <div v-else class="space-y-4">
          <div>
            <label class="text-sm font-medium mb-2 block">Template</label>
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
                </div>
              </label>
            </div>
          </div>

          <div>
            <label class="text-sm font-medium mb-1 block">Project Name</label>
            <input
              v-model="scaffoldName"
              type="text"
              placeholder="my-laravel-app"
              class="w-full rounded-md border border-input bg-background px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-ring"
            />
          </div>

          <div>
            <label class="text-sm font-medium mb-1 block">Destination Folder</label>
            <div class="flex gap-2">
              <input
                :value="scaffoldDest"
                readonly
                placeholder="Select destination..."
                class="flex-1 rounded-md border border-input bg-background px-3 py-2 text-sm text-muted-foreground"
              />
              <Button variant="outline" size="sm" @click="browseScaffoldDest">Browse</Button>
            </div>
          </div>
        </div>
      </CardContent>

      <CardFooter class="flex justify-end gap-2 pt-2">
        <Button variant="ghost" @click="emit('close')">Cancel</Button>
        <Button
          v-if="activeTab === 'import'"
          :disabled="!importName || !importPath || importLoading"
          @click="handleRegister"
        >
          {{ importLoading ? 'Registering...' : 'Register Project' }}
        </Button>
        <Button
          v-else
          :disabled="!scaffoldName || !scaffoldDest"
          @click="handleScaffold"
        >
          Scaffold Project
        </Button>
      </CardFooter>
    </Card>
  </div>
</template>
