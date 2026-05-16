<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { getVersion } from '@tauri-apps/api/app'
import { Loader2 } from 'lucide-vue-next'
import { checkForUpdate } from '@/composables/useUpdater'

const { t } = useI18n()
const version = ref<string>('...')
const checking = ref(false)

onMounted(async () => {
  try {
    version.value = await getVersion()
  } catch {
    version.value = 'unknown'
  }
})

async function check() {
  checking.value = true
  try {
    await checkForUpdate()
  } finally {
    checking.value = false
  }
}
</script>

<template>
  <div class="space-y-6">
    <div>
      <h3 class="mb-1 text-sm font-semibold">{{ t('settings.currentVersion') }}</h3>
      <p class="font-mono text-lg font-bold">v{{ version }}</p>
    </div>

    <div>
      <button
        class="flex items-center gap-2 rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90 disabled:opacity-60"
        :disabled="checking"
        @click="check"
      >
        <Loader2 v-if="checking" class="h-4 w-4 animate-spin" />
        {{ t('settings.checkUpdates') }}
      </button>
      <p class="mt-2 text-xs text-muted-foreground">{{ t('settings.updatesAutomatic') }}</p>
    </div>
  </div>
</template>
