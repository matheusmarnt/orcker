import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { Channel } from '@tauri-apps/api/core'
import { commands } from '@/ipc/bindings'
import type { LogLine } from '@/ipc/bindings'

export type LogSource = 'All' | 'Docker' | 'Laravel' | 'Nginx' | 'Supervisor'
export const LOG_LEVELS = ['', 'DEBUG', 'INFO', 'WARNING', 'ERROR', 'CRITICAL'] as const

export interface LogLineWithId extends LogLine {
  id: number
}

const MAX_LINES = 5000

export const useLogsStore = defineStore('logs', () => {
  const lines = ref<LogLineWithId[]>([])
  const activeSource = ref<LogSource>('All')
  const levelFilter = ref<string>('')
  const keywordFilter = ref<string>('')
  let _lineCounter = 0

  const filteredLines = computed(() => {
    return lines.value.filter(line => {
      if (activeSource.value !== 'All' && line.source !== activeSource.value) return false
      if (levelFilter.value && !line.text.toUpperCase().includes(levelFilter.value.toUpperCase())) return false
      if (keywordFilter.value && !line.text.toLowerCase().includes(keywordFilter.value.toLowerCase())) return false
      return true
    })
  })

  function appendLine(line: LogLine): void {
    lines.value.push({ ...line, id: _lineCounter++ })
    if (lines.value.length > MAX_LINES) {
      lines.value.splice(0, lines.value.length - MAX_LINES)
    }
  }

  async function startStream(projectId: string, projectPath: string): Promise<void> {
    clearLines()
    const channel = new Channel<LogLine>()
    channel.onmessage = (line) => appendLine(line)
    await commands.startLogStream(projectId, projectPath, channel)
  }

  async function stopStream(projectId: string): Promise<void> {
    await commands.stopLogStream(projectId)
  }

  function clearLines(): void {
    lines.value = []
    _lineCounter = 0
  }

  return {
    lines,
    filteredLines,
    activeSource,
    levelFilter,
    keywordFilter,
    appendLine,
    startStream,
    stopStream,
    clearLines,
    MAX_LINES,
  }
})
