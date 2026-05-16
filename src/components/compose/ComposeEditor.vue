<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import MonacoEditor from 'monaco-editor-vue3'
import { configureMonacoYaml } from 'monaco-yaml'
import * as monaco from 'monaco-editor'
import { commands } from '@/ipc/bindings'
import { toast } from 'vue-sonner'
import { Button } from '@/components/ui/button'
import ComposeErrorPanel from './ComposeErrorPanel.vue'

const { t } = useI18n()

// Configure monaco-yaml once at module scope (not in setup())
// This avoids re-configuring on every component mount
let _yamlConfigured = false
function ensureYamlConfigured() {
  if (_yamlConfigured) return
  _yamlConfigured = true
  configureMonacoYaml(monaco, { validate: true, schemas: [] })
}

const props = defineProps<{
  projectId: string
  projectStatus: string
}>()

const emit = defineEmits<{
  close: []
  restart: []
}>()

const content = ref('')
const isLoading = ref(true)
const isSaving = ref(false)
const markers = ref<Array<{ message: string; severity: number }>>([])

const hasErrors = computed(() =>
  markers.value.some((m) => m.severity === monaco.MarkerSeverity.Error),
)

const editorOptions = {
  language: 'yaml',
  theme: 'vs-dark',
  minimap: { enabled: false },
  fontSize: 13,
  lineNumbers: 'on' as const,
  scrollBeyondLastLine: false,
  automaticLayout: true,
}

function onEditorMount(editorInstance: monaco.editor.IStandaloneCodeEditor) {
  ensureYamlConfigured()

  // Track YAML validation markers
  monaco.editor.onDidChangeMarkers(() => {
    const model = editorInstance.getModel()
    if (model) {
      const raw = monaco.editor.getModelMarkers({ resource: model.uri })
      markers.value = raw.map((m: monaco.editor.IMarker) => ({
        message: m.message,
        severity: m.severity,
      }))
    }
  })
}

onMounted(async () => {
  const result = await commands.readComposeFile(props.projectId)
  if (result.status === 'ok') {
    content.value = result.data
  } else {
    toast.error(`Failed to load compose file: ${(result.error as any).message ?? result.error}`)
  }
  isLoading.value = false
})

async function handleSave() {
  if (hasErrors.value || isSaving.value) return

  isSaving.value = true
  try {
    const result = await commands.saveComposeFile(props.projectId, content.value)
    if (result.status === 'ok') {
      const isRunning = ['running', 'partially_running', 'unhealthy'].includes(
        props.projectStatus,
      )
      if (isRunning) {
        toast(t('composeEditor.savedToast'), {
          action: {
            label: t('composeEditor.actions.restart'),
            onClick: () => emit('restart'),
          },
        })
      }
      emit('close')
    } else {
      toast.error(`Save failed: ${(result.error as any).message ?? result.error}`)
    }
  } finally {
    isSaving.value = false
  }
}
</script>

<template>
  <!-- Right-side drawer overlay — fixed position, same pattern as DestructiveConfirmDialog -->
  <div class="fixed inset-0 z-50 flex justify-end" @click.self="emit('close')">
    <!-- Backdrop -->
    <div class="absolute inset-0 bg-black/40" @click="emit('close')" />

    <!-- Drawer panel -->
    <div class="relative z-10 flex h-full w-full max-w-2xl flex-col bg-background shadow-xl">
      <!-- Header -->
      <div class="flex items-center justify-between border-b border-border px-4 py-3">
        <h2 class="text-base font-semibold">{{ t('composeEditor.title') }}</h2>
        <Button variant="ghost" size="sm" @click="emit('close')">✕</Button>
      </div>

      <!-- Loading state -->
      <div v-if="isLoading" class="flex flex-1 items-center justify-center">
        <span class="h-6 w-6 animate-spin rounded-full border-2 border-primary border-t-transparent" />
      </div>

      <!-- Editor area -->
      <template v-else>
        <div class="flex-1 overflow-hidden">
          <MonacoEditor
            v-model="content"
            :options="editorOptions"
            language="yaml"
            class="h-full w-full"
            @mount="onEditorMount"
          />
        </div>

        <!-- Error panel below editor -->
        <ComposeErrorPanel :markers="markers" />

        <!-- Footer actions -->
        <div class="flex items-center justify-end gap-2 border-t border-border px-4 py-3">
          <Button variant="outline" size="sm" @click="emit('close')">{{ t('common.cancel') }}</Button>
          <Button
            size="sm"
            :disabled="hasErrors || isSaving"
            data-testid="save-btn"
            @click="handleSave"
          >
            <span
              v-if="isSaving"
              class="mr-1.5 h-3 w-3 animate-spin rounded-full border-2 border-current border-t-transparent"
            />
            {{ isSaving ? t('composeEditor.saving') : t('composeEditor.save') }}
          </Button>
        </div>
      </template>
    </div>
  </div>
</template>
