<script setup lang="ts">
import Sonner from '@/components/ui/sonner/Sonner.vue'
import AppShell from '@/components/layout/AppShell.vue'
import { useDockerStore } from './stores/docker'

// Register Tauri event listeners at earliest possible point — before any child mounts.
const docker = useDockerStore()
docker.initEventListener()
</script>

<template>
  <AppShell>
    <router-view />
  </AppShell>

  <Sonner rich-colors position="bottom-right" />
</template>
