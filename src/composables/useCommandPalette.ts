import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

const STORAGE_KEY = 'orcker-command-history'
const MAX_HISTORY = 10

const ALL_COMMANDS = [
  'php artisan migrate',
  'php artisan migrate:fresh --seed',
  'php artisan tinker',
  'php artisan cache:clear',
  'php artisan config:clear',
  'php artisan route:clear',
  'php artisan view:clear',
  'npm run dev',
  'pest',
]

export const useCommandPaletteStore = defineStore('commandPalette', () => {
  const isOpen = ref(false)
  const query = ref('')
  const history = ref<string[]>([])
  // Last command dispatched from palette — views pick this up to route to CommandPanel
  const pendingCommand = ref<string | null>(null)

  // Fuzzy filter: every char of query appears in order within the command string
  const filtered = computed(() => {
    if (!query.value) {
      // Show history first, then remaining commands
      return [...new Set([...history.value, ...ALL_COMMANDS])]
    }
    const q = query.value.toLowerCase()
    return ALL_COMMANDS.filter((cmd) => {
      let i = 0
      for (const ch of cmd) {
        if (ch === q[i]) i++
        if (i === q.length) return true
      }
      return false
    })
  })

  function open() {
    isOpen.value = true
    query.value = ''
  }

  function close() {
    isOpen.value = false
  }

  function execute(cmd: string): string {
    // Prepend to history (dedup + max 10)
    history.value = [cmd, ...history.value.filter((c) => c !== cmd)].slice(0, MAX_HISTORY)
    localStorage.setItem(STORAGE_KEY, JSON.stringify(history.value))
    close()
    return cmd
  }

  function init() {
    try {
      const stored = localStorage.getItem(STORAGE_KEY)
      history.value = stored ? (JSON.parse(stored) as string[]) : []
    } catch {
      history.value = []
    }
  }

  return { isOpen, query, filtered, history, pendingCommand, open, close, execute, init }
})
