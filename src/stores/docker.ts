import { defineStore } from 'pinia'
import { ref } from 'vue'
import { listen } from '@tauri-apps/api/event'
import { getDockerVersion, listContainers } from '../ipc/docker'

export type ConnectionStatus = 'connecting' | 'connected' | 'disconnected' | 'error'

export interface ContainerSummary {
  id: string
  name: string
  status: string
  image: string
  ports: string
}

interface RustContainerSummary {
  id: string
  names: string[]
  image: string
  status: string
  state: string
}

interface RustAppError {
  kind: string
  message: string
}

export const useDockerStore = defineStore('docker', () => {
  const connectionStatus = ref<ConnectionStatus>('connecting')
  const socketPath = ref<string | null>(null)
  const dockerVersion = ref<string | null>(null)
  const containers = ref<Map<string, ContainerSummary>>(new Map())
  const errorKind = ref<string | null>(null)
  const errorMessage = ref<string | null>(null)

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

  async function refreshContainers() {
    try {
      const raw = (await listContainers()) as RustContainerSummary[]
      const map = new Map<string, ContainerSummary>()
      for (const c of raw) {
        map.set(c.id, {
          id: c.id,
          name: c.names?.[0]?.replace(/^\//, '') ?? c.id.slice(0, 12),
          status: c.status,
          image: c.image,
          ports: '',
        })
      }
      containers.value = map
    } catch {
      // non-fatal: show toast in future phases
    }
  }

  async function initEventListener() {
    // docker://connected — Rust emits socket_path string after probe succeeds
    await listen<string>('docker://connected', async (event) => {
      const socket = event.payload
      try {
        const version = await getDockerVersion()
        setConnected(version, socket)
      } catch {
        setConnected('unknown', socket)
      }
      await refreshContainers()
    })

    // docker://error — Rust emits JSON-stringified AppError
    await listen<string>('docker://error', (event) => {
      try {
        const err = JSON.parse(event.payload) as RustAppError
        setError(err.kind, err.message)
      } catch {
        setError('Internal', event.payload)
      }
    })

    // docker://container-event — refresh list on any container change
    await listen('docker://container-event', async () => {
      if (connectionStatus.value === 'connected') {
        await refreshContainers()
      }
    })
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
    refreshContainers,
  }
})
