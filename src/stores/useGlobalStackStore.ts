import { defineStore } from 'pinia'
import { ref } from 'vue'
import { listen } from '@tauri-apps/api/event'
import { toast } from 'vue-sonner'
import { commands } from '@/ipc/bindings'
import type { ServiceId, ServiceStatus, ServiceConfig } from '@/ipc/bindings'

export const useGlobalStackStore = defineStore('globalStack', () => {
  const statuses = ref<Record<string, ServiceStatus>>({})
  const configs = ref<Record<string, ServiceConfig>>({})
  const restartRequired = ref<Record<string, boolean>>({})

  // ---------------------------------------------------------------------------
  // Helpers
  // ---------------------------------------------------------------------------

  function isTransitioning(id: ServiceId): boolean {
    const s = statuses.value[id]
    return s?.kind === 'starting' || s?.kind === 'stopping'
  }

  function isRunning(id: ServiceId): boolean {
    return statuses.value[id]?.kind === 'running'
  }

  function hasAnyRunning(): boolean {
    return Object.values(statuses.value).some((s) => s?.kind === 'running')
  }

  // ---------------------------------------------------------------------------
  // Init — loads statuses and subscribes to live events
  // ---------------------------------------------------------------------------

  async function init() {
    const result = await commands.getServicesStatus()
    if (result.status === 'ok') {
      statuses.value = result.data as Record<string, ServiceStatus>
    } else {
      toast.error(`Failed to load service statuses: ${result.error.message}`)
    }

    await listen<{ service: ServiceId; status: ServiceStatus }>(
      'global://service-status',
      (event) => {
        const { service, status } = event.payload
        statuses.value[service] = status
        if (status.kind === 'error') {
          console.error(`[orcker] ${service} error:`, status.message)
          toast.error(`${service}: ${status.message}`)
        }
      },
    )
  }

  // ---------------------------------------------------------------------------
  // Actions
  // ---------------------------------------------------------------------------

  async function toggleService(id: ServiceId) {
    const result = await commands.toggleService(id)
    if (result.status === 'error') {
      toast.error(`toggle ${id}: ${result.error.message}`)
    }
  }

  async function setConfig(id: ServiceId, config: ServiceConfig) {
    const result = await commands.setServiceConfig(id, config)
    if (result.status === 'ok') {
      if (result.data) restartRequired.value[id] = true
      configs.value[id] = config
    } else {
      toast.error(`setConfig ${id}: ${result.error.message}`)
    }
  }

  async function globalOn() {
    const result = await commands.globalOn()
    if (result.status === 'error') {
      toast.error(`globalOn: ${result.error.message}`)
    }
  }

  async function globalOff() {
    const result = await commands.globalOff()
    if (result.status === 'error') {
      toast.error(`globalOff: ${result.error.message}`)
    }
  }

  async function smartToggle() {
    if (hasAnyRunning()) await globalOff()
    else await globalOn()
  }

  // ---------------------------------------------------------------------------
  // Expose
  // ---------------------------------------------------------------------------

  return {
    statuses,
    configs,
    restartRequired,
    isTransitioning,
    isRunning,
    hasAnyRunning,
    init,
    toggleService,
    setConfig,
    globalOn,
    globalOff,
    smartToggle,
  }
})
