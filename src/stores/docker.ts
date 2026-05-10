import { defineStore } from 'pinia'
import { ref } from 'vue'
import { listen } from '@tauri-apps/api/event'

export type ConnectionStatus = 'connecting' | 'connected' | 'disconnected' | 'error'

export interface ContainerSummary {
  id: string
  name: string
  status: string
  image: string
  ports: string
}

export const useDockerStore = defineStore('docker', () => {
  const connectionStatus = ref<ConnectionStatus>('connecting')
  const socketPath = ref<string | null>(null)
  const dockerVersion = ref<string | null>(null)
  const containers = ref<Map<string, ContainerSummary>>(new Map())
  const errorKind = ref<string | null>(null)
  const errorMessage = ref<string | null>(null)

  async function initEventListener() {
    await listen<Record<string, unknown>>('docker://container-event', () => {
      // Trigger container list refresh — implemented in Plan 05
    })
  }

  function setConnected(version: string, socket: string) {
    connectionStatus.value = 'connected'
    dockerVersion.value = version
    socketPath.value = socket
    errorKind.value = null
    errorMessage.value = null
  }

  function setError(kind: string, message: string) {
    connectionStatus.value = 'error'
    errorKind.value = kind
    errorMessage.value = message
  }

  function setConnecting() {
    connectionStatus.value = 'connecting'
  }

  return {
    connectionStatus,
    socketPath,
    dockerVersion,
    containers,
    errorKind,
    errorMessage,
    initEventListener,
    setConnected,
    setError,
    setConnecting,
  }
})
