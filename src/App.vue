<script setup lang="ts">
import Sonner from '@/components/ui/sonner/Sonner.vue'
import DockerStatusBadge from './components/DockerStatusBadge.vue'
import { useDockerStore } from './stores/docker'

// Register Tauri event listeners at earliest possible point — before any child mounts.
// DashboardView.vue must NOT call initEventListener() again.
const docker = useDockerStore()
docker.initEventListener()
</script>

<template>
  <div class="flex h-screen w-screen overflow-hidden bg-background text-foreground">
    <!-- Left sidebar — 240px fixed -->
    <aside class="flex w-60 flex-shrink-0 flex-col border-r border-border bg-card">
      <!-- Nav area (top) -->
      <div class="flex-1 overflow-y-auto p-3">
        <nav class="flex flex-col gap-1">
          <router-link
            to="/dashboard"
            class="flex items-center gap-2 rounded-md px-3 py-2 text-sm font-medium transition-colors hover:bg-accent hover:text-accent-foreground"
            active-class="bg-accent text-accent-foreground"
          >
            Dashboard
          </router-link>
          <router-link
            to="/global"
            class="flex items-center gap-2 rounded-md px-3 py-2 text-sm font-medium transition-colors hover:bg-accent hover:text-accent-foreground"
            active-class="bg-accent text-accent-foreground"
          >
            Global Stack
          </router-link>
        </nav>
      </div>

      <!-- Docker status badge (bottom) -->
      <div class="border-t border-border p-3">
        <DockerStatusBadge />
      </div>
    </aside>

    <!-- Main content -->
    <main class="flex flex-1 flex-col overflow-hidden">
      <router-view />
    </main>
  </div>

  <Sonner rich-colors position="bottom-right" />
</template>
