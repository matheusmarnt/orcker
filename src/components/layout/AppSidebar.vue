<script setup lang="ts">
import { useLocalStorage } from '@vueuse/core'
import { useRoute } from 'vue-router'
import {
  ChevronLeft,
  ChevronRight,
  Container,
  FolderOpen,
  LayoutDashboard,
  ScrollText,
  Server,
  Settings,
} from 'lucide-vue-next'

const emit = defineEmits<{ openSettings: [] }>()

const collapsed = useLocalStorage('sidebar:collapsed', false)
const route = useRoute()

const navItems = [
  { to: '/dashboard', icon: LayoutDashboard, label: 'Dashboard' },
  { to: '/projects', icon: FolderOpen, label: 'Projects' },
  { to: '/global', icon: Server, label: 'Global Stack' },
  { to: '/logs', icon: ScrollText, label: 'Logs' },
  { to: '/infra', icon: Container, label: 'Infra' },
]

function toggle() {
  collapsed.value = !collapsed.value
}
</script>

<template>
  <aside
    :class="[
      'flex flex-shrink-0 flex-col border-r border-border bg-card transition-all duration-200',
      collapsed ? 'w-14' : 'w-52',
    ]"
  >
    <!-- Nav items -->
    <div class="flex-1 overflow-y-auto p-2">
      <nav class="flex flex-col gap-1">
        <RouterLink
          v-for="item in navItems"
          :key="item.to"
          :to="item.to"
          :class="[
            'flex items-center gap-2 rounded-md px-3 py-2 text-sm font-medium transition-colors hover:bg-accent hover:text-accent-foreground',
            route.path === item.to ? 'bg-accent text-accent-foreground' : '',
            collapsed ? 'justify-center px-2' : '',
          ]"
          :title="collapsed ? item.label : undefined"
        >
          <component :is="item.icon" class="h-4 w-4 flex-shrink-0" />
          <span v-if="!collapsed">{{ item.label }}</span>
        </RouterLink>
      </nav>
    </div>

    <!-- Settings + Collapse -->
    <div class="border-t border-border p-2 space-y-1">
      <button
        class="flex w-full items-center gap-2 rounded-md px-3 py-2 text-sm text-muted-foreground transition-colors hover:bg-accent hover:text-accent-foreground"
        :class="collapsed ? 'justify-center px-2' : ''"
        title="Settings"
        @click="emit('openSettings')"
      >
        <Settings class="h-4 w-4 flex-shrink-0" />
        <span v-if="!collapsed">Settings</span>
      </button>
      <button
        class="flex w-full items-center justify-center rounded-md p-2 text-muted-foreground transition-colors hover:bg-accent hover:text-accent-foreground"
        :title="collapsed ? 'Expand sidebar' : 'Collapse sidebar'"
        @click="toggle"
      >
        <ChevronLeft v-if="!collapsed" class="h-4 w-4" />
        <ChevronRight v-else class="h-4 w-4" />
      </button>
    </div>
  </aside>
</template>
