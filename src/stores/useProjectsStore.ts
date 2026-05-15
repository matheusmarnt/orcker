import { defineStore } from 'pinia'
import { ref } from 'vue'
import { listen } from '@tauri-apps/api/event'
import { toast } from 'vue-sonner'
import { isPermissionGranted, requestPermission, sendNotification } from '@tauri-apps/plugin-notification'
import { commands } from '@/ipc/bindings'
import type { ProjectConfig, ImportResult, ProjectStatus, ProjectStatusEvent } from '@/ipc/bindings'

export const useProjectsStore = defineStore('projects', () => {
  const projects = ref<ProjectConfig[]>([])
  const loading = ref(false)
  const statuses = ref<Map<string, ProjectStatus>>(new Map())

  async function refreshStatuses() {
    await Promise.all(
      projects.value.map(async (p) => {
        const res = await commands.getProjectStatus(p.id)
        if (res.status === 'ok') statuses.value.set(p.id, res.data)
      }),
    )
  }

  async function init(): Promise<void> {
    loading.value = true
    try {
      const result = await commands.listProjects()
      if (result.status === 'ok') {
        projects.value = result.data
      }
    } finally {
      loading.value = false
    }

    await refreshStatuses()

    await listen<ProjectStatusEvent>('project://status', async (event) => {
      const prev = statuses.value.get(event.payload.project_id)
      statuses.value.set(event.payload.project_id, event.payload.status)

      // Send OS notification when a project transitions to unhealthy
      if (
        event.payload.status.kind === 'unhealthy' &&
        prev?.kind !== 'unhealthy'
      ) {
        const project = projects.value.find((p) => p.id === event.payload.project_id)
        try {
          let granted = await isPermissionGranted()
          if (!granted) {
            const permission = await requestPermission()
            granted = permission === 'granted'
          }
          if (granted) {
            sendNotification({
              title: 'Orcker',
              body: `Project "${project?.name ?? event.payload.project_id}" is unhealthy`,
            })
          }
        } catch {
          // Notification not supported in this environment — ignore
        }
      }
    })

    await listen<ProjectConfig>('projects://registered', (event) => {
      if (!projects.value.find(p => p.id === event.payload.id)) {
        projects.value.push(event.payload)
      }
    })
  }

  async function registerProject(name: string, path: string): Promise<ProjectConfig | null> {
    const result = await commands.registerProject(name, path)
    if (result.status === 'ok') {
      if (!projects.value.find(p => p.id === result.data.id)) {
        projects.value.push(result.data)
      }
      toast.success(`Project "${name}" registered`)
      return result.data
    } else {
      toast.error(`Failed to register project: ${result.error}`)
      return null
    }
  }

  async function importProject(folderPath: string): Promise<ImportResult | null> {
    const result = await commands.importProject(folderPath)
    if (result.status === 'ok') return result.data
    toast.error(`Import failed: ${result.error}`)
    return null
  }

  async function pickFolder(): Promise<string | null> {
    const result = await commands.pickProjectFolder()
    if (result.status === 'ok') return result.data
    return null
  }

  return { projects, loading, statuses, init, refreshStatuses, registerProject, importProject, pickFolder }
})
