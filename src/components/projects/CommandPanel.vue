<script setup lang="ts">
import { ref } from 'vue'
import { Channel } from '@tauri-apps/api/core'
import Convert from 'ansi-to-html'
import { commands } from '@/ipc/bindings'
import type { ArtisanCommand, CommandChunk, ProjectConfig } from '@/ipc/bindings'
import { Button } from '@/components/ui/button'
import DestructiveConfirmDialog from './DestructiveConfirmDialog.vue'

const props = defineProps<{
  project: ProjectConfig
  artisanCommands: ArtisanCommand[]
}>()

const emit = defineEmits<{
  close: []
}>()

const convert = new Convert({ escapeXML: true })

interface OutputLine {
  html: string
  isStderr: boolean
}

const lines = ref<OutputLine[]>([])
const isRunning = ref(false)
const activeCommandId = ref<string | null>(null)

// Destructive confirm dialog state
const pendingDestructiveId = ref<string | null>(null)
const pendingDestructiveLabel = ref('')

async function runCommand(commandId: string) {
  lines.value = []
  isRunning.value = true
  activeCommandId.value = commandId

  const channel = new Channel<CommandChunk>()
  channel.onmessage = (chunk: CommandChunk) => {
    lines.value.push({
      html: convert.toHtml(chunk.text),
      isStderr: chunk.is_stderr,
    })
  }

  try {
    const result = await commands.runArtisanCommand(commandId, props.project.id, channel)
    if (result.status === 'error') {
      lines.value.push({
        html: convert.toHtml(`Error: ${JSON.stringify(result.error)}`),
        isStderr: true,
      })
    }
  } catch (e) {
    lines.value.push({
      html: convert.toHtml(`Error: ${String(e)}`),
      isStderr: true,
    })
  } finally {
    isRunning.value = false
    activeCommandId.value = null
  }
}

async function cancelCommand() {
  await commands.cancelArtisanCommand(props.project.id)
  isRunning.value = false
}

function onCommandClick(cmd: ArtisanCommand) {
  if (cmd.destructive) {
    pendingDestructiveId.value = cmd.id
    pendingDestructiveLabel.value = cmd.label
  } else {
    runCommand(cmd.id)
  }
}

function onDestructiveConfirm() {
  const id = pendingDestructiveId.value
  pendingDestructiveId.value = null
  pendingDestructiveLabel.value = ''
  if (id) runCommand(id)
}

function onDestructiveCancel() {
  pendingDestructiveId.value = null
  pendingDestructiveLabel.value = ''
}
</script>

<template>
  <div class="border-t bg-background flex flex-col">
    <!-- Panel header -->
    <div class="flex items-center justify-between px-4 py-2 border-b bg-muted/50">
      <span class="text-sm font-medium">Terminal</span>
      <div class="flex items-center gap-2">
        <Button
          v-if="isRunning"
          size="sm"
          variant="destructive"
          @click="cancelCommand"
        >
          Cancel
        </Button>
        <Button size="sm" variant="ghost" @click="emit('close')">✕</Button>
      </div>
    </div>

    <!-- Output area -->
    <div class="font-mono text-sm overflow-auto h-48 bg-black text-white p-2">
      <div
        v-for="(line, i) in lines"
        :key="i"
        :class="line.isStderr ? 'text-red-400' : ''"
      >
        <span v-html="line.html" />
      </div>
      <div v-if="lines.length === 0 && !isRunning" class="text-zinc-500">
        Run a command to see output here.
      </div>
    </div>

    <!-- Command buttons -->
    <div class="flex flex-wrap gap-2 p-3 border-t">
      <Button
        v-for="cmd in artisanCommands"
        :key="cmd.id"
        size="sm"
        :variant="cmd.destructive ? 'destructive' : 'outline'"
        :disabled="isRunning"
        @click="onCommandClick(cmd)"
      >
        {{ cmd.label }}
      </Button>
    </div>

    <!-- Destructive confirm dialog -->
    <DestructiveConfirmDialog
      v-if="pendingDestructiveId !== null"
      :project-name="project.name"
      :command-label="pendingDestructiveLabel"
      @confirm="onDestructiveConfirm"
      @cancel="onDestructiveCancel"
    />
  </div>
</template>
