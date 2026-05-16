<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { commands } from '@/ipc/bindings'

const { t } = useI18n()

const props = defineProps<{
  projectId: string
}>()

const diff = ref<string>('')
const isLoading = ref(false)
const error = ref<string | null>(null)

onMounted(async () => {
  await loadDiff()
})

async function loadDiff() {
  isLoading.value = true
  error.value = null
  const result = await commands.getComposeDiff(props.projectId)
  isLoading.value = false
  if (result.status === 'ok') {
    diff.value = result.data
  } else {
    error.value = String(result.error)
  }
}

// Parse diff into colored lines
function diffLines(raw: string) {
  return raw.split('\n').map((line) => {
    if (line.startsWith('+') && !line.startsWith('+++')) return { text: line, kind: 'add' }
    if (line.startsWith('-') && !line.startsWith('---')) return { text: line, kind: 'remove' }
    if (line.startsWith('@@')) return { text: line, kind: 'hunk' }
    return { text: line, kind: 'context' }
  })
}
</script>

<template>
  <div class="rounded-md border border-border bg-card text-sm">
    <!-- Header -->
    <div class="flex items-center justify-between border-b border-border px-3 py-2">
      <span class="font-medium text-foreground">{{ t('composeEditor.configChanges') }}</span>
      <div class="flex items-center gap-2">
        <button
          class="text-xs text-muted-foreground hover:text-foreground transition-colors"
          :title="t('composeEditor.versionSoon')"
          disabled
        >
          {{ t('composeEditor.viewHistory') }}
        </button>
        <button
          class="text-xs text-muted-foreground hover:text-foreground transition-colors"
          @click="loadDiff"
        >
          {{ t('common.refresh') }}
        </button>
      </div>
    </div>

    <!-- Loading state -->
    <div v-if="isLoading" class="px-3 py-4 text-center text-muted-foreground text-xs">
      {{ t('composeEditor.loadingDiff') }}
    </div>

    <!-- Error state -->
    <div v-else-if="error" class="px-3 py-4 text-center text-destructive text-xs">
      {{ error }}
    </div>

    <!-- Empty state: no prior commits -->
    <div v-else-if="!diff" class="px-3 py-4 text-center text-muted-foreground text-xs">
      {{ t('composeEditor.noVersion') }}
    </div>

    <!-- Diff viewer -->
    <div v-else class="overflow-x-auto">
      <pre class="p-3 font-mono text-xs leading-5"><template v-for="(line, i) in diffLines(diff)" :key="i"><span
          :class="{
            'text-green-500': line.kind === 'add',
            'text-red-500': line.kind === 'remove',
            'text-blue-400': line.kind === 'hunk',
            'text-muted-foreground': line.kind === 'context',
          }"
        >{{ line.text }}</span><br /></template></pre>
    </div>

    <!-- Version history placeholder -->
    <div class="border-t border-border px-3 py-2 text-xs text-muted-foreground">
      {{ t('composeEditor.versionSoon') }}
    </div>
  </div>
</template>
