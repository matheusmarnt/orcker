<script setup lang="ts">
import { useSettingsStore } from '@/stores/useSettingsStore'
import { useI18n } from 'vue-i18n'

const store = useSettingsStore()
// useScope: 'global' ensures locale change propagates to all components
const { locale: i18nLocale } = useI18n({ useScope: 'global' })

const themeOptions = [
  { value: 'light' as const, label: 'Light' },
  { value: 'dark' as const, label: 'Dark' },
  { value: 'auto' as const, label: 'System' },
]

const localeOptions = [
  { value: 'en', label: 'EN' },
  { value: 'pt-BR', label: 'PT-BR' },
  { value: 'es', label: 'ES' },
]

function setTheme(t: 'light' | 'dark' | 'auto') {
  store.setTheme(t)
  store.save()
}

function setLocale(l: string) {
  store.locale = l
  i18nLocale.value = l
  store.save()
}
</script>

<template>
  <div class="space-y-6">
    <div>
      <h3 class="mb-1 text-sm font-semibold">Theme</h3>
      <p class="mb-3 text-xs text-muted-foreground">Select the color theme for the application.</p>
      <div class="flex gap-2">
        <button
          v-for="option in themeOptions"
          :key="option.value"
          :class="[
            'rounded-md border px-4 py-2 text-sm font-medium transition-colors',
            store.theme === option.value
              ? 'border-primary bg-primary text-primary-foreground'
              : 'border-border bg-background hover:bg-accent hover:text-accent-foreground',
          ]"
          @click="setTheme(option.value)"
        >
          {{ option.label }}
        </button>
      </div>
    </div>

    <div>
      <h3 class="mb-1 text-sm font-semibold">Language</h3>
      <p class="mb-3 text-xs text-muted-foreground">Choose the display language.</p>
      <div class="flex gap-2">
        <button
          v-for="option in localeOptions"
          :key="option.value"
          :class="[
            'rounded-md border px-4 py-2 text-sm font-medium transition-colors',
            store.locale === option.value
              ? 'border-primary bg-primary text-primary-foreground'
              : 'border-border bg-background hover:bg-accent hover:text-accent-foreground',
          ]"
          @click="setLocale(option.value)"
        >
          {{ option.label }}
        </button>
      </div>
    </div>
  </div>
</template>
