<script setup lang="ts">
import { computed } from 'vue'
import { CircleX, Loader2 } from 'lucide-vue-next'

const props = defineProps<{
  errorKind: string | null
  errorMessage: string | null
}>()

const title = computed(() => {
  switch (props.errorKind) {
    case 'DockerUnavailable':
      return 'Docker is not running'
    case 'DockerPermission':
      return 'Permission denied'
    default:
      return 'Connection error'
  }
})
</script>

<template>
  <div class="flex h-full w-full items-center justify-center">
    <div class="flex max-w-md flex-col items-center gap-4 text-center">
      <!-- Error icon -->
      <CircleX class="h-14 w-14 text-destructive" />

      <!-- Title -->
      <h2 class="text-lg font-semibold text-foreground">{{ title }}</h2>

      <!-- Human-readable message -->
      <p class="text-sm text-muted-foreground">
        {{ errorMessage ?? 'An unexpected error occurred while connecting to Docker.' }}
      </p>

      <!-- Expandable raw detail -->
      <details class="w-full text-left">
        <summary
          class="cursor-pointer select-none text-xs text-muted-foreground hover:text-foreground"
        >
          Show raw error
        </summary>
        <pre
          class="mt-2 overflow-x-auto rounded-md border border-border bg-muted px-3 py-2 text-xs font-mono text-muted-foreground"
        >{{ errorMessage }}</pre>
      </details>

      <!-- Auto-reconnecting indicator (always visible, no manual retry button) -->
      <div class="flex items-center gap-2 text-sm text-muted-foreground">
        <Loader2 class="h-4 w-4 animate-spin" />
        <span>Reconnecting…</span>
      </div>
    </div>
  </div>
</template>
