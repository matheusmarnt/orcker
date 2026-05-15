import { defineStore } from 'pinia'
import { ref } from 'vue'
import { commands } from '@/ipc/bindings'
import type { ImageInfo, VolumeInfo } from '@/ipc/bindings'

export const useInfraStore = defineStore('infra', () => {
  const volumes = ref<VolumeInfo[]>([])
  const images = ref<ImageInfo[]>([])
  const isLoadingVolumes = ref(false)
  const isLoadingImages = ref(false)

  async function refreshVolumes() {
    isLoadingVolumes.value = true
    const r = await commands.listVolumes().catch(() => null)
    if (r?.status === 'ok') volumes.value = r.data
    isLoadingVolumes.value = false
  }

  async function refreshImages() {
    isLoadingImages.value = true
    const r = await commands.listImages().catch(() => null)
    if (r?.status === 'ok') images.value = r.data
    isLoadingImages.value = false
  }

  return { volumes, images, isLoadingVolumes, isLoadingImages, refreshVolumes, refreshImages }
})
