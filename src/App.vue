<script setup lang="ts">
import Sonner from '@/components/ui/sonner/Sonner.vue'
import AppShell from '@/components/layout/AppShell.vue'
import { useDockerStore } from './stores/docker'
import { useSettingsStore } from './stores/useSettingsStore'

// Register Tauri event listeners at earliest possible point — before any child mounts.
const docker = useDockerStore()
docker.initEventListener()

// Load persisted settings (theme, locale) before rendering.
const settings = useSettingsStore()
settings.load()
</script>

<template>
  <AppShell>
    <router-view />
  </AppShell>

  <Sonner rich-colors position="bottom-right" />
</template>
