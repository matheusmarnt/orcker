<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { commands } from '@/ipc/bindings'
import type { EnvEntry } from '@/ipc/bindings'
import { Button } from '@/components/ui/button'
import { toast } from 'vue-sonner'

const props = defineProps<{
  projectId: string
}>()

const envEntries = ref<EnvEntry[]>([])
const exampleEntries = ref<EnvEntry[]>([])
const loading = ref(true)
const saving = ref(false)
const error = ref<string | null>(null)

// Build a Set of keys present in .env.example for diff detection
const exampleKeys = computed(() =>
  new Set(
    exampleEntries.value
      .filter((e) => !e.is_comment)
      .map((e) => e.key),
  ),
)

// Build a map of key → example value for the diff column
const exampleValueMap = computed(() => {
  const map = new Map<string, string>()
  for (const entry of exampleEntries.value) {
    if (!entry.is_comment) {
      map.set(entry.key, entry.value)
    }
  }
  return map
})

// Keys present in .env.example but absent from .env (missing from env)
const missingKeys = computed(() => {
  const envKeys = new Set(
    envEntries.value.filter((e) => !e.is_comment).map((e) => e.key),
  )
  const missing: string[] = []
  for (const key of exampleKeys.value) {
    if (!envKeys.has(key)) {
      missing.push(key)
    }
  }
  return new Set(missing)
})

onMounted(async () => {
  loading.value = true
  error.value = null
  const result = await commands.readEnvFile(props.projectId)
  if (result.status === 'ok') {
    envEntries.value = result.data.env.entries.map((e) => ({ ...e }))
    exampleEntries.value = result.data.example.entries
  } else {
    error.value = String(result.error)
  }
  loading.value = false
})

async function save() {
  saving.value = true
  const result = await commands.saveEnvFile(props.projectId, envEntries.value)
  if (result.status === 'ok') {
    toast.success('.env saved')
  } else {
    toast.error(`Failed to save .env: ${result.error}`)
  }
  saving.value = false
}

function addMissingKey(key: string) {
  envEntries.value.push({
    key,
    value: exampleValueMap.value.get(key) ?? '',
    is_comment: false,
  })
}
</script>

<template>
  <div class="rounded-md border p-4 space-y-4">
    <div class="flex items-center justify-between mb-2">
      <h3 class="text-sm font-semibold">Edit .env</h3>
      <Button size="sm" :disabled="saving" @click="save">
        {{ saving ? 'Saving…' : 'Save .env' }}
      </Button>
    </div>

    <!-- Loading state -->
    <div v-if="loading" class="text-sm text-muted-foreground py-4 text-center">
      Loading .env…
    </div>

    <!-- Error state -->
    <div v-else-if="error" class="text-sm text-destructive py-4">
      {{ error }}
    </div>

    <template v-else>
      <!-- Missing keys banner -->
      <div v-if="missingKeys.size > 0" class="rounded border border-yellow-400 bg-yellow-50 dark:bg-yellow-950 p-3 space-y-1">
        <p class="text-xs font-semibold text-yellow-700 dark:text-yellow-300">
          Keys in .env.example but missing from .env:
        </p>
        <div class="flex flex-wrap gap-2">
          <button
            v-for="key in missingKeys"
            :key="key"
            class="text-xs underline text-yellow-800 dark:text-yellow-200 hover:no-underline"
            @click="addMissingKey(key)"
          >
            + {{ key }}
          </button>
        </div>
      </div>

      <!-- Table -->
      <div class="overflow-x-auto">
        <table class="w-full text-sm border-collapse">
          <thead>
            <tr class="border-b">
              <th class="text-left py-1 px-2 text-muted-foreground font-medium w-2/5">Key</th>
              <th class="text-left py-1 px-2 text-muted-foreground font-medium w-2/5">.env value</th>
              <th class="text-left py-1 px-2 text-muted-foreground font-medium w-1/5">.env.example</th>
            </tr>
          </thead>
          <tbody>
            <template v-for="(entry, idx) in envEntries" :key="idx">
              <!-- Comment row -->
              <tr v-if="entry.is_comment" class="border-b border-dashed">
                <td colspan="3" class="py-1 px-2 text-muted-foreground italic text-xs">
                  {{ entry.key }}
                </td>
              </tr>
              <!-- Key=Value row -->
              <tr
                v-else
                class="border-b"
                :class="{
                  'bg-yellow-50 dark:bg-yellow-950/40 border-l-2 border-l-yellow-400':
                    !exampleKeys.has(entry.key) === false && exampleValueMap.get(entry.key) !== entry.value && exampleKeys.has(entry.key),
                }"
              >
                <td class="py-1 px-2 font-mono text-xs align-middle">{{ entry.key }}</td>
                <td class="py-1 px-2 align-middle">
                  <input
                    v-model="entry.value"
                    class="w-full bg-transparent border border-input rounded px-2 py-0.5 text-xs font-mono focus:outline-none focus:ring-1 focus:ring-ring"
                  />
                </td>
                <td class="py-1 px-2 align-middle">
                  <span
                    v-if="exampleKeys.has(entry.key)"
                    class="text-xs font-mono text-muted-foreground"
                    :title="exampleValueMap.get(entry.key)"
                  >
                    {{ exampleValueMap.get(entry.key) || '(empty)' }}
                  </span>
                  <span v-else class="text-xs text-muted-foreground italic">—</span>
                </td>
              </tr>
            </template>
          </tbody>
        </table>
      </div>
    </template>
  </div>
</template>
