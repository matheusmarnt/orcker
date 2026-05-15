<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import AppSidebar from './AppSidebar.vue'
import CommandPalette from '@/components/command-palette/CommandPalette.vue'
import SettingsModal from '@/components/settings/SettingsModal.vue'
import { useCommandPaletteStore } from '@/composables/useCommandPalette'

const paletteStore = useCommandPaletteStore()
const showSettings = ref(false)

function onKeydown(e: KeyboardEvent) {
  if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
    e.preventDefault()
    paletteStore.open()
  }
}

onMounted(() => {
  document.addEventListener('keydown', onKeydown)
})

onUnmounted(() => {
  document.removeEventListener('keydown', onKeydown)
})

function handleCommand(cmd: string) {
  // Store the pending command — active views (ProjectDetailView) watch this
  // and route it to their CommandPanel automatically
  paletteStore.pendingCommand = cmd
}
</script>

<template>
  <div class="flex h-screen w-screen overflow-hidden bg-background text-foreground">
    <AppSidebar @open-settings="showSettings = true" />
    <main class="flex-1 overflow-auto">
      <slot />
    </main>

    <!-- Command Palette overlay -->
    <CommandPalette v-if="paletteStore.isOpen" @run="handleCommand" />

    <!-- Settings modal -->
    <SettingsModal :open="showSettings" @close="showSettings = false" />
  </div>
</template>
