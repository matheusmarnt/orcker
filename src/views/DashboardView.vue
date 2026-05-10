<script setup lang="ts">
import { onMounted } from 'vue'
import { Skeleton } from '../components/ui/skeleton'
import ContainerTable from '../components/ContainerTable.vue'
import ErrorScreen from '../components/ErrorScreen.vue'
import { useDockerStore } from '../stores/docker'

const docker = useDockerStore()

onMounted(() => {
  docker.initEventListener()
})
</script>

<template>
  <div class="flex flex-1 flex-col overflow-auto p-6">
    <!-- Skeleton while connecting -->
    <template v-if="docker.connectionStatus === 'connecting'">
      <div class="flex flex-col gap-3">
        <Skeleton class="h-8 w-48" />
        <Skeleton class="h-10 w-full rounded-md" />
        <Skeleton class="h-10 w-full rounded-md" />
        <Skeleton class="h-10 w-5/6 rounded-md" />
        <Skeleton class="h-10 w-4/6 rounded-md" />
      </div>
    </template>

    <!-- Container table when connected -->
    <template v-else-if="docker.connectionStatus === 'connected'">
      <ContainerTable :containers="Array.from(docker.containers.values())" />
    </template>

    <!-- Full-screen fatal error -->
    <template v-else-if="docker.connectionStatus === 'error'">
      <ErrorScreen :error-kind="docker.errorKind" :error-message="docker.errorMessage" />
    </template>
  </div>
</template>
