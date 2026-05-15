<script setup lang="ts">
import { computed } from 'vue'
import { Line } from 'vue-chartjs'
import {
  Chart as ChartJS,
  CategoryScale,
  LinearScale,
  PointElement,
  LineElement,
  Title,
  Tooltip,
  Filler,
} from 'chart.js'
import { useResourceMonitor } from '@/composables/useResourceMonitor'

ChartJS.register(CategoryScale, LinearScale, PointElement, LineElement, Title, Tooltip, Filler)

const props = defineProps<{ projectId: string }>()

const { series } = useResourceMonitor(props.projectId)

const labels = computed(() => series.value.map((p) => p.time))

const cpuData = computed(() => ({
  labels: labels.value,
  datasets: [
    {
      label: 'CPU %',
      data: series.value.map((p) => p.cpu),
      borderColor: 'rgb(99, 102, 241)',
      backgroundColor: 'rgba(99, 102, 241, 0.1)',
      fill: true,
      tension: 0.3,
      pointRadius: 0,
    },
  ],
}))

const memData = computed(() => ({
  labels: labels.value,
  datasets: [
    {
      label: 'Memory MB',
      data: series.value.map((p) => p.mem),
      borderColor: 'rgb(16, 185, 129)',
      backgroundColor: 'rgba(16, 185, 129, 0.1)',
      fill: true,
      tension: 0.3,
      pointRadius: 0,
    },
  ],
}))

const chartOptions = {
  responsive: true,
  maintainAspectRatio: false,
  animation: false as const,
  plugins: {
    title: { display: false },
  },
  scales: {
    x: {
      ticks: { maxTicksLimit: 6, maxRotation: 0 },
      grid: { display: false },
    },
    y: {
      beginAtZero: true,
      ticks: { maxTicksLimit: 5 },
    },
  },
}
</script>

<template>
  <div class="space-y-4">
    <div>
      <p class="mb-1 text-xs font-medium text-muted-foreground">CPU %</p>
      <div class="h-24 w-full">
        <Line :data="cpuData" :options="chartOptions" />
      </div>
    </div>
    <div>
      <p class="mb-1 text-xs font-medium text-muted-foreground">Memory MB</p>
      <div class="h-24 w-full">
        <Line :data="memData" :options="chartOptions" />
      </div>
    </div>
  </div>
</template>
