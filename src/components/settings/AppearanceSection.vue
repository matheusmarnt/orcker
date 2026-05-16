<script setup lang="ts">
import { useSettingsStore } from '@/stores/useSettingsStore'
import { useI18n } from 'vue-i18n'

const store = useSettingsStore()
const { t } = useI18n()

const themeOptions = [
  { value: 'light' as const, key: 'theme.light' },
  { value: 'dark' as const, key: 'theme.dark' },
  { value: 'auto' as const, key: 'theme.system' },
]

const localeOptions = [
  { value: 'en', label: 'EN' },
  { value: 'pt-BR', label: 'PT-BR' },
  { value: 'es', label: 'ES' },
]

function setTheme(theme: 'light' | 'dark' | 'auto') {
  store.setTheme(theme)
  store.save()
}

function setLocale(l: string) {
  store.setLocale(l)
  store.save()
}
</script>

<template>
  <div class="space-y-6">
    <div>
      <h3 class="mb-1 text-sm font-semibold">{{ t('settings.theme') }}</h3>
      <p class="mb-3 text-xs text-muted-foreground">{{ t('settings.themeDescription') }}</p>
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
          {{ t(option.key) }}
        </button>
      </div>
    </div>

    <div>
      <h3 class="mb-1 text-sm font-semibold">{{ t('settings.language') }}</h3>
      <p class="mb-3 text-xs text-muted-foreground">{{ t('settings.languageDescription') }}</p>
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
