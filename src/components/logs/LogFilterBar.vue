<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import { useLogsStore, LOG_LEVELS } from '@/stores/useLogsStore'

const { t } = useI18n()
const store = useLogsStore()
</script>

<template>
  <div class="flex gap-2 items-center p-2 border-b bg-background">
    <select
      v-model="store.levelFilter"
      class="h-8 text-sm border rounded px-2 bg-background"
    >
      <option value="">{{ t('logs.allLevels') }}</option>
      <option v-for="level in LOG_LEVELS.filter(l => l)" :key="level" :value="level">
        {{ level }}
      </option>
    </select>
    <input
      v-model="store.keywordFilter"
      type="text"
      :placeholder="t('logs.filterKeyword')"
      class="h-8 text-sm border rounded px-2 flex-1 bg-background"
    />
    <button
      v-if="store.keywordFilter || store.levelFilter"
      class="h-8 text-xs px-2 border rounded text-muted-foreground hover:text-foreground"
      @click="store.keywordFilter = ''; store.levelFilter = ''"
    >
      {{ t('logs.clearFilter') }}
    </button>
  </div>
</template>
