<script setup lang="ts">
import AppSidebar from './AppSidebar.vue'
</script>

<template>
  <div class="flex h-screen w-screen overflow-hidden bg-background text-foreground">
    <AppSidebar />
    <main class="flex-1 overflow-auto">
      <slot />
    </main>
  </div>
</template>
