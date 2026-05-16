<script setup lang="ts">
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import ImageList from '@/components/infra/ImageList.vue'
import VolumeList from '@/components/infra/VolumeList.vue'
import TemplateMarketplace from '@/components/infra/TemplateMarketplace.vue'

const { t } = useI18n()

type Tab = 'volumes' | 'images' | 'templates'
const activeTab = ref<Tab>('volumes')

const tabs: Tab[] = ['volumes', 'images', 'templates']
</script>

<template>
  <div class="flex h-full flex-col p-6">
    <div class="mb-6">
      <h2 class="mb-1 text-xl font-bold">{{ t('infra.title') }}</h2>
      <p class="text-sm text-muted-foreground">{{ t('infra.description') }}</p>
    </div>

    <!-- Tab bar -->
    <div class="mb-6 flex gap-1 border-b border-border">
      <button
        v-for="tab in tabs"
        :key="tab"
        :class="[
          'px-4 py-2 text-sm font-medium transition-colors',
          activeTab === tab
            ? 'border-b-2 border-primary text-foreground'
            : 'text-muted-foreground hover:text-foreground',
        ]"
        @click="activeTab = tab"
      >
        {{ t('infra.tabs.' + tab) }}
      </button>
    </div>

    <!-- Tab content -->
    <div class="flex-1 overflow-y-auto">
      <VolumeList v-if="activeTab === 'volumes'" />
      <ImageList v-else-if="activeTab === 'images'" />
      <TemplateMarketplace v-else />
    </div>
  </div>
</template>
