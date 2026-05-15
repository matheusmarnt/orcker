<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { commands } from '@/ipc/bindings'
import type { IniSection } from '@/ipc/bindings'
import { Button } from '@/components/ui/button'
import { toast } from 'vue-sonner'

const props = defineProps<{
  projectId: string
}>()

const sections = ref<IniSection[]>([])
const activeTab = ref<string>('')
const showRaw = ref(false)
const rawContent = ref('')
const loading = ref(false)
const saving = ref(false)

// Derive raw INI text from sections for the raw textarea
const sectionsToRaw = (secs: IniSection[]): string =>
  secs
    .map((s) => {
      const entries = s.entries
        .map((e) => (e.is_comment ? e.key : `${e.key} = ${e.value}`))
        .join('\n')
      return `[${s.name}]\n${entries}`
    })
    .join('\n\n')

// Parse raw INI text back to sections (minimal parser for raw tab)
const rawToSections = (raw: string): IniSection[] => {
  const result: IniSection[] = []
  let current: IniSection | null = null
  for (const line of raw.split('\n')) {
    const trimmed = line.trim()
    if (!trimmed) continue
    if (trimmed.startsWith('[') && trimmed.endsWith(']')) {
      current = { name: trimmed.slice(1, -1), entries: [] }
      result.push(current)
    } else if (current) {
      if (trimmed.startsWith(';')) {
        current.entries.push({ key: trimmed, value: '', is_comment: true })
      } else {
        const pos = trimmed.indexOf('=')
        if (pos !== -1) {
          current.entries.push({
            key: trimmed.slice(0, pos).trim(),
            value: trimmed.slice(pos + 1).trim(),
            is_comment: false,
          })
        }
      }
    }
  }
  return result
}

const activeSectionEntries = computed(() => {
  const sec = sections.value.find((s) => s.name === activeTab.value)
  return sec ? sec.entries : []
})

function setEntryValue(key: string, newValue: string) {
  const sec = sections.value.find((s) => s.name === activeTab.value)
  if (!sec) return
  const entry = sec.entries.find((e) => e.key === key)
  if (entry) entry.value = newValue
}

function toggleRaw() {
  if (!showRaw.value) {
    rawContent.value = sectionsToRaw(sections.value)
  }
  showRaw.value = !showRaw.value
}

async function load() {
  loading.value = true
  const result = await commands.readPhpIni(props.projectId)
  loading.value = false
  if (result.status === 'ok') {
    sections.value = result.data
    activeTab.value = result.data[0]?.name ?? ''
  } else {
    toast.error('Failed to load php.ini', { description: String(result.error) })
  }
}

async function save() {
  saving.value = true
  // Sync raw textarea back to sections if in raw mode
  if (showRaw.value) {
    sections.value = rawToSections(rawContent.value)
  }
  const result = await commands.savePhpIni(props.projectId, sections.value)
  saving.value = false
  if (result.status === 'ok') {
    toast.success('php.ini saved')
  } else {
    toast.error('Failed to save php.ini', { description: String(result.error) })
  }
}

onMounted(load)
</script>

<template>
  <div class="rounded-md border p-4 space-y-4">
    <div class="flex items-center justify-between">
      <h3 class="text-sm font-semibold">php.ini Editor</h3>
      <div class="flex items-center gap-2">
        <Button variant="ghost" size="sm" @click="toggleRaw">
          {{ showRaw ? 'Structured' : 'Raw' }}
        </Button>
        <Button size="sm" :disabled="saving" @click="save">
          {{ saving ? 'Saving…' : 'Save php.ini' }}
        </Button>
      </div>
    </div>

    <div v-if="loading" class="text-sm text-muted-foreground py-4 text-center">
      Loading…
    </div>

    <!-- Raw fallback textarea -->
    <div v-else-if="showRaw">
      <textarea
        v-model="rawContent"
        class="w-full h-64 font-mono text-xs bg-muted rounded p-3 border resize-y focus:outline-none"
        spellcheck="false"
      />
    </div>

    <!-- Structured section tabs -->
    <div v-else-if="sections.length">
      <!-- Tab bar -->
      <div class="flex gap-1 flex-wrap mb-3">
        <button
          v-for="sec in sections"
          :key="sec.name"
          class="px-3 py-1 text-xs rounded border transition-colors"
          :class="
            activeTab === sec.name
              ? 'bg-primary text-primary-foreground border-primary'
              : 'bg-background text-muted-foreground border-border hover:bg-muted'
          "
          @click="activeTab = sec.name"
        >
          {{ sec.name }}
        </button>
      </div>

      <!-- Entries table -->
      <table class="w-full text-xs">
        <tbody>
          <tr
            v-for="entry in activeSectionEntries"
            :key="entry.key"
            class="border-b last:border-b-0"
          >
            <td
              v-if="entry.is_comment"
              colspan="2"
              class="py-1 pl-1 text-muted-foreground font-mono italic"
            >
              {{ entry.key }}
            </td>
            <template v-else>
              <td class="py-1 w-1/2 font-mono text-foreground pr-4">{{ entry.key }}</td>
              <td class="py-1 w-1/2">
                <input
                  :value="entry.value"
                  class="w-full bg-muted rounded px-2 py-0.5 font-mono focus:outline-none focus:ring-1 focus:ring-ring"
                  @input="setEntryValue(entry.key, ($event.target as HTMLInputElement).value)"
                />
              </td>
            </template>
          </tr>
        </tbody>
      </table>
    </div>

    <div v-else class="text-sm text-muted-foreground py-4 text-center">
      No php.ini found at docker/php.ini or php.ini in project root.
    </div>
  </div>
</template>
