<script setup lang="ts">
// Severity values from Monaco MarkerSeverity enum:
// Error = 8, Warning = 4, Info = 2, Hint = 1

defineProps<{
  markers: Array<{ message: string; severity: number }>
}>()
</script>

<template>
  <div v-if="markers.length > 0" class="border-t border-border bg-background px-3 py-2 text-sm space-y-1 max-h-32 overflow-y-auto">
    <div
      v-for="(marker, i) in markers"
      :key="i"
      :class="marker.severity === 8 ? 'text-destructive' : 'text-amber-500'"
      class="flex items-start gap-1.5"
    >
      <span class="shrink-0 font-medium">{{ marker.severity === 8 ? 'Error' : 'Warning' }}:</span>
      <span>{{ marker.message }}</span>
    </div>
  </div>
</template>
