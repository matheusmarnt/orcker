<script setup lang="ts">
import { ref } from 'vue'
import { Channel } from '@tauri-apps/api/core'
import Convert from 'ansi-to-html'
import { toast } from 'vue-sonner'
import { commands } from '@/ipc/bindings'
import type { CommandChunk } from '@/ipc/bindings'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'

const props = defineProps<{
  projectId: string
  projectName: string
}>()

const convert = new Convert({ escapeXML: true })

// ── State ─────────────────────────────────────────────────────────────────────

const isCreating = ref(false)
const isLoadingDump = ref(false)
const isLoadingRestore = ref(false)
const isOpeningCli = ref(false)

// CLI output panel
interface OutputLine {
  html: string
  isStderr: boolean
}
const cliLines = ref<OutputLine[]>([])
const showCli = ref(false)

// Derived DB name shown in status chip
const testingDbName = computed(() => {
  return `${props.projectName.replace(/-/g, '_').replace(/ /g, '_')}_testing`
})

// ── Actions ───────────────────────────────────────────────────────────────────

async function createDb() {
  isCreating.value = true
  try {
    const result = await commands.createTestingDb(props.projectName)
    if (result.status === 'error') {
      toast.error(`Failed to create DB: ${JSON.stringify(result.error)}`)
    } else {
      toast.success(`Database "${testingDbName.value}" created`)
    }
  } catch (e) {
    toast.error(`Failed to create DB: ${String(e)}`)
  } finally {
    isCreating.value = false
  }
}

async function dumpDb() {
  isLoadingDump.value = true
  try {
    const result = await commands.dumpDb(props.projectName)
    if (result.status === 'error') {
      toast.error(`Dump failed: ${JSON.stringify(result.error)}`)
    } else {
      toast.success('Database dump saved')
    }
  } catch (e) {
    toast.error(`Dump failed: ${String(e)}`)
  } finally {
    isLoadingDump.value = false
  }
}

async function restoreDb() {
  isLoadingRestore.value = true
  try {
    const result = await commands.restoreDb(props.projectName)
    if (result.status === 'error') {
      toast.error(`Restore failed: ${JSON.stringify(result.error)}`)
    } else {
      toast.success('Database restored successfully')
    }
  } catch (e) {
    toast.error(`Restore failed: ${String(e)}`)
  } finally {
    isLoadingRestore.value = false
  }
}

async function openCli() {
  isOpeningCli.value = true
  cliLines.value = []
  showCli.value = true

  const channel = new Channel<CommandChunk>()
  channel.onmessage = (chunk: CommandChunk) => {
    cliLines.value.push({
      html: convert.toHtml(chunk.text),
      isStderr: chunk.is_stderr,
    })
  }

  try {
    const result = await commands.openDbCli(props.projectName, channel)
    if (result.status === 'error') {
      toast.error(`psql failed: ${JSON.stringify(result.error)}`)
    }
  } catch (e) {
    toast.error(`psql failed: ${String(e)}`)
  } finally {
    isOpeningCli.value = false
  }
}

function closeCli() {
  showCli.value = false
  cliLines.value = []
}

// ── computed ──────────────────────────────────────────────────────────────────

import { computed } from 'vue'
</script>

<template>
  <div class="flex flex-col gap-4 p-4">
    <!-- Testing DB status chip -->
    <div class="flex items-center gap-3">
      <span class="text-sm font-medium text-muted-foreground">Testing DB:</span>
      <Badge variant="secondary" class="font-mono text-xs">{{ testingDbName }}</Badge>
      <Button
        size="sm"
        variant="outline"
        :disabled="isCreating"
        @click="createDb"
      >
        <span
          v-if="isCreating"
          class="mr-1.5 h-3 w-3 animate-spin rounded-full border-2 border-current border-t-transparent"
        />
        {{ isCreating ? 'Creating…' : 'Create / Reset' }}
      </Button>
    </div>

    <!-- Action buttons -->
    <div class="flex flex-wrap gap-2">
      <Button
        size="sm"
        variant="outline"
        :disabled="isLoadingDump || isLoadingRestore || isOpeningCli"
        @click="dumpDb"
      >
        <span
          v-if="isLoadingDump"
          class="mr-1.5 h-3 w-3 animate-spin rounded-full border-2 border-current border-t-transparent"
        />
        {{ isLoadingDump ? 'Dumping…' : 'Dump' }}
      </Button>

      <Button
        size="sm"
        variant="outline"
        :disabled="isLoadingDump || isLoadingRestore || isOpeningCli"
        @click="restoreDb"
      >
        <span
          v-if="isLoadingRestore"
          class="mr-1.5 h-3 w-3 animate-spin rounded-full border-2 border-current border-t-transparent"
        />
        {{ isLoadingRestore ? 'Restoring…' : 'Restore' }}
      </Button>

      <Button
        size="sm"
        variant="outline"
        :disabled="isLoadingDump || isLoadingRestore || isOpeningCli"
        @click="openCli"
      >
        <span
          v-if="isOpeningCli"
          class="mr-1.5 h-3 w-3 animate-spin rounded-full border-2 border-current border-t-transparent"
        />
        {{ isOpeningCli ? 'Connecting…' : 'Open CLI' }}
      </Button>
    </div>

    <!-- CLI output panel (v-if so Vue teardown clears Channel listeners) -->
    <div v-if="showCli" class="border rounded-md overflow-hidden">
      <div class="flex items-center justify-between px-4 py-2 border-b bg-muted/50">
        <span class="text-sm font-medium">psql — {{ testingDbName }}</span>
        <Button size="sm" variant="ghost" @click="closeCli">✕</Button>
      </div>
      <div class="font-mono text-sm overflow-auto h-48 bg-black text-white p-2">
        <div
          v-for="(line, i) in cliLines"
          :key="i"
          :class="line.isStderr ? 'text-red-400' : ''"
        >
          <span v-html="line.html" />
        </div>
        <div v-if="cliLines.length === 0 && isOpeningCli" class="text-zinc-500">
          Connecting to psql…
        </div>
        <div v-if="cliLines.length === 0 && !isOpeningCli" class="text-zinc-500">
          No output yet.
        </div>
      </div>
    </div>
  </div>
</template>
