import { ref, onMounted, onUnmounted } from 'vue'
import { commands } from '@/ipc/bindings'

export interface ResourcePoint {
  time: string
  cpu: number
  mem: number
}

export function useResourceMonitor(projectId: string) {
  const series = ref<ResourcePoint[]>([])
  let intervalId: ReturnType<typeof setInterval> | null = null

  async function poll() {
    const r = await commands.getResourceStats(projectId).catch(() => null)
    if (!r || r.status !== 'ok') return
    const { cpu_percent, mem_mb } = r.data
    if (cpu_percent === null || mem_mb === null) return
    series.value = [
      ...series.value.slice(-30), // keep last 30 points (60 seconds)
      {
        time: new Date().toLocaleTimeString(),
        cpu: cpu_percent,
        mem: mem_mb,
      },
    ]
  }

  onMounted(() => {
    poll()
    intervalId = setInterval(poll, 2000)
  })

  onUnmounted(() => {
    if (intervalId) clearInterval(intervalId)
  })

  return { series }
}
