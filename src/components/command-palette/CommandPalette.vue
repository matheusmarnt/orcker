<script setup lang="ts">
import { onMounted, onUnmounted, nextTick, ref, watch } from 'vue'
import { useCommandPaletteStore } from '@/composables/useCommandPalette'

const emit = defineEmits<{
  run: [cmd: string]
}>()

const store = useCommandPaletteStore()
const inputRef = ref<HTMLInputElement | null>(null)
const selectedIndex = ref(0)

// Auto-focus input when palette opens
watch(
  () => store.isOpen,
  async (open) => {
    if (open) {
      selectedIndex.value = 0
      await nextTick()
      inputRef.value?.focus()
    }
  },
)

// Reset selection when query changes
watch(
  () => store.query,
  () => {
    selectedIndex.value = 0
  },
)

function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape') {
    store.close()
    return
  }
  if (e.key === 'ArrowDown') {
    e.preventDefault()
    selectedIndex.value = Math.min(selectedIndex.value + 1, store.filtered.length - 1)
    return
  }
  if (e.key === 'ArrowUp') {
    e.preventDefault()
    selectedIndex.value = Math.max(selectedIndex.value - 1, 0)
    return
  }
  if (e.key === 'Enter') {
    const cmd = store.filtered[selectedIndex.value]
    if (cmd) runCommand(cmd)
    return
  }
}

function runCommand(cmd: string) {
  const executed = store.execute(cmd)
  emit('run', executed)
}

function handleBackdropClick(e: MouseEvent) {
  if (e.target === e.currentTarget) {
    store.close()
  }
}

onMounted(() => {
  store.init()
})

onUnmounted(() => {
  // nothing to clean up — keyboard listener is on the element itself
})
</script>

<template>
  <div
    class="fixed inset-0 z-50 flex items-start justify-center bg-black/50 pt-24 backdrop-blur-sm"
    @click="handleBackdropClick"
    @keydown="handleKeydown"
  >
    <div class="bg-card border-border w-full max-w-lg rounded-lg border shadow-xl">
      <!-- Search input -->
      <div class="border-border flex items-center border-b px-3">
        <svg
          class="text-muted-foreground mr-2 h-4 w-4 shrink-0"
          xmlns="http://www.w3.org/2000/svg"
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
          />
        </svg>
        <input
          ref="inputRef"
          v-model="store.query"
          type="text"
          placeholder="Search commands..."
          class="placeholder:text-muted-foreground flex h-11 w-full bg-transparent py-3 text-sm outline-none"
        />
        <kbd
          class="border-border text-muted-foreground ml-2 hidden rounded border px-1.5 py-0.5 font-mono text-xs sm:block"
        >
          Esc
        </kbd>
      </div>

      <!-- Command list -->
      <div class="max-h-72 overflow-y-auto p-1">
        <div v-if="store.filtered.length === 0" class="text-muted-foreground py-6 text-center text-sm">
          No commands found.
        </div>
        <button
          v-for="(cmd, idx) in store.filtered"
          :key="cmd"
          class="flex w-full items-center gap-2 rounded-md px-3 py-2 text-left text-sm transition-colors"
          :class="idx === selectedIndex ? 'bg-accent text-accent-foreground' : 'hover:bg-accent/50'"
          @click="runCommand(cmd)"
          @mouseenter="selectedIndex = idx"
        >
          <!-- Clock icon for history items -->
          <svg
            v-if="store.history.includes(cmd)"
            class="text-muted-foreground h-3.5 w-3.5 shrink-0"
            xmlns="http://www.w3.org/2000/svg"
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z"
            />
          </svg>
          <!-- Terminal icon for non-history items -->
          <svg
            v-else
            class="text-muted-foreground h-3.5 w-3.5 shrink-0"
            xmlns="http://www.w3.org/2000/svg"
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M8 9l3 3-3 3m5 0h3"
            />
          </svg>
          <span class="font-mono">{{ cmd }}</span>
        </button>
      </div>

      <!-- Footer hint -->
      <div class="border-border text-muted-foreground flex items-center gap-3 border-t px-3 py-2 text-xs">
        <span><kbd class="font-mono">↑↓</kbd> navigate</span>
        <span><kbd class="font-mono">↵</kbd> execute</span>
        <span><kbd class="font-mono">Esc</kbd> close</span>
      </div>
    </div>
  </div>
</template>
