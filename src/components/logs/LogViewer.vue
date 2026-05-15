<script setup lang="ts">
import { ref, watch, nextTick } from 'vue'
import { RecycleScroller } from 'vue-virtual-scroller'
import 'vue-virtual-scroller/dist/vue-virtual-scroller.css'
import { useLogsStore } from '@/stores/useLogsStore'
import LogLineComponent from './LogLine.vue'

const store = useLogsStore()
const scrollerRef = ref<{ $el: HTMLElement } | null>(null)

// Auto-scroll to bottom on new lines
watch(() => store.filteredLines.length, async () => {
  await nextTick()
  if (scrollerRef.value) {
    const el = scrollerRef.value.$el
    el.scrollTop = el.scrollHeight
  }
})
</script>

<template>
  <RecycleScroller
    ref="scrollerRef"
    class="h-full bg-[#1e1e1e] text-[#d4d4d4]"
    :items="store.filteredLines"
    :item-size="20"
    key-field="id"
    v-slot="{ item }"
  >
    <LogLineComponent :line="item" />
  </RecycleScroller>
</template>
