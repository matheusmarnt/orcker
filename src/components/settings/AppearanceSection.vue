<script setup lang="ts">
import { useSettingsStore } from '@/stores/useSettingsStore'
import { useI18n } from 'vue-i18n'

const store = useSettingsStore()
const { locale: i18nLocale } = useI18n()

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
          v-for="option in [{ value: 'light', label: 'Light' }, { value: 'dark', label: 'Dark' }, { value: 'auto', label: 'System' }]"
          :key="option.value"
          :class="[
            'rounded-md border px-4 py-2 text-sm font-medium transition-colors',
            store.colorMode === option.value
              ? 'border-primary bg-primary text-primary-foreground'
              : 'border-border bg-background hover:bg-accent hover:text-accent-foreground',
          ]"
          @click="setTheme(option.value as 'light' | 'dark' | 'auto')"
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
          v-for="option in [{ value: 'en', label: 'EN' }, { value: 'pt-BR', label: 'PT-BR' }]"
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
