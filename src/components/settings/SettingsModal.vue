<script setup lang="ts">
import { ref } from 'vue'
import { X } from 'lucide-vue-next'
import AppearanceSection from './AppearanceSection.vue'
import DockerSection from './DockerSection.vue'
import PreferencesSection from './PreferencesSection.vue'
import UpdatesSection from './UpdatesSection.vue'
import DataSection from './DataSection.vue'

defineProps<{ open: boolean }>()
const emit = defineEmits<{ close: [] }>()

type Section = 'appearance' | 'preferences' | 'docker' | 'updates' | 'data'

const activeSection = ref<Section>('appearance')

const sections: { key: Section; label: string }[] = [
  { key: 'appearance', label: 'Appearance' },
  { key: 'preferences', label: 'Preferences' },
  { key: 'docker', label: 'Docker' },
  { key: 'updates', label: 'Updates' },
  { key: 'data', label: 'Data' },
]

const sectionComponents = {
  appearance: AppearanceSection,
  preferences: PreferencesSection,
  docker: DockerSection,
  updates: UpdatesSection,
  data: DataSection,
}
</script>

<template>
  <Teleport to="body">
    <div
      v-if="open"
      class="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
      @click.self="emit('close')"
    >
      <div class="relative flex h-[540px] w-[760px] max-w-[90vw] overflow-hidden rounded-xl border border-border bg-card shadow-2xl">
        <!-- Left sidebar -->
        <nav class="flex w-44 flex-shrink-0 flex-col border-r border-border bg-background p-2">
          <p class="mb-2 px-3 pt-2 text-xs font-semibold uppercase tracking-wider text-muted-foreground">
            Settings
          </p>
          <button
            v-for="section in sections"
            :key="section.key"
            :class="[
              'rounded-md px-3 py-2 text-left text-sm font-medium transition-colors',
              activeSection === section.key
                ? 'bg-accent text-accent-foreground'
                : 'text-foreground hover:bg-accent/50',
            ]"
            @click="activeSection = section.key"
          >
            {{ section.label }}
          </button>
        </nav>

        <!-- Right panel -->
        <div class="flex flex-1 flex-col overflow-hidden">
          <div class="flex items-center justify-between border-b border-border px-6 py-4">
            <h2 class="text-base font-semibold capitalize">{{ activeSection }}</h2>
            <button
              class="rounded-md p-1 text-muted-foreground transition-colors hover:bg-accent hover:text-foreground"
              @click="emit('close')"
            >
              <X class="h-4 w-4" />
            </button>
          </div>
          <div class="flex-1 overflow-y-auto p-6">
            <component :is="sectionComponents[activeSection]" />
          </div>
        </div>
      </div>
    </div>
  </Teleport>
</template>
