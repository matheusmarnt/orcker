import { defineStore } from 'pinia'
import { ref } from 'vue'
import { listen } from '@tauri-apps/api/event'
import { toast } from 'vue-sonner'
import { commands } from '@/ipc/bindings'
import type { ProjectConfig, ImportResult } from '@/ipc/bindings'

export const useProjectsStore = defineStore('projects', () => {
  const projects = ref<ProjectConfig[]>([])
  const loading = ref(false)

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

  return { projects, loading, init, registerProject, importProject, pickFolder }
})
