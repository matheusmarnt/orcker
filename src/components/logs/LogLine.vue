<script setup lang="ts">
import { computed } from 'vue'
import Convert from 'ansi-to-html'
import type { LogLineWithId } from '@/stores/useLogsStore'

const props = defineProps<{ line: LogLineWithId }>()
const convert = new Convert({ escapeXML: true })
const html = computed(() => convert.toHtml(props.line.text))
</script>

<template>
  <div class="font-mono text-xs leading-5 px-2 whitespace-pre-wrap break-all">
    <span
      :class="line.source === 'Docker' ? 'text-blue-400' : line.source === 'Laravel' ? 'text-green-400' : 'text-yellow-400'"
      class="mr-2 select-none"
    >[{{ line.source }}]</span>
    <span v-html="html" />
  </div>
</template>
