<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { toast } from 'vue-sonner'
import { commands } from '@/ipc/bindings'
import { useInfraStore } from '@/stores/useInfraStore'

const store = useInfraStore()
const confirmingPrune = ref(false)
const pullImageName = ref('')
const isPulling = ref(false)

onMounted(() => {
  store.refreshImages()
})

function formatMb(bytes: number): string {
  return Math.round(bytes / 1_048_576).toString()
}

function truncateId(id: string): string {
  // Remove sha256: prefix if present, then truncate to 12 chars
  const stripped = id.startsWith('sha256:') ? id.slice(7) : id
  return stripped.slice(0, 12)
}

async function pullImage() {
  const raw = pullImageName.value.trim()
  if (!raw) return
  const colonIdx = raw.lastIndexOf(':')
  const image = colonIdx > 0 ? raw.slice(0, colonIdx) : raw
  const tag = colonIdx > 0 ? raw.slice(colonIdx + 1) : 'latest'

  isPulling.value = true
  const r = await commands.pullImage(image, tag).catch((e: Error) => {
    toast.error('Pull failed', { description: e.message })
    return null
  })
  isPulling.value = false
  if (!r) return
  if (r.status === 'error') {
    toast.error('Pull failed', { description: String(r.error) })
    return
  }
  toast.success(`Pulled ${image}:${tag}`)
  pullImageName.value = ''
  store.refreshImages()
}

async function removeImage(imageId: string) {
  const r = await commands.removeImage(imageId).catch((e: Error) => {
    toast.error('Remove failed', { description: e.message })
    return null
  })
  if (!r) return
  if (r.status === 'error') {
    toast.error('Remove failed', { description: String(r.error) })
    return
  }
  toast.success('Image removed')
  store.refreshImages()
}

async function pruneImages() {
  confirmingPrune.value = false
  const r = await commands.pruneImages().catch((e: Error) => {
    toast.error('Failed to prune images', { description: e.message })
    return null
  })
  if (!r) return
  if (r.status === 'error') {
    toast.error('Failed to prune images', { description: String(r.error) })
    return
  }
  const mb = Math.round(r.data / 1_048_576)
  toast.success(`Reclaimed ${mb} MB`)
  store.refreshImages()
}
</script>

<template>
  <div class="space-y-4">
    <!-- Header with pull input and prune button -->
    <div class="flex flex-wrap items-center justify-between gap-2">
      <h3 class="text-lg font-semibold">Images</h3>
      <div class="flex items-center gap-2">
        <input
          v-model="pullImageName"
          type="text"
          placeholder="image:tag"
          class="h-8 rounded-md border bg-background px-2 text-sm focus:outline-none focus:ring-1 focus:ring-ring"
          @keyup.enter="pullImage"
        />
        <button
          class="rounded-md bg-primary px-3 py-1 text-sm text-primary-foreground hover:bg-primary/90 disabled:opacity-50"
          :disabled="isPulling || !pullImageName.trim()"
          @click="pullImage"
        >
          {{ isPulling ? 'Pulling…' : 'Pull' }}
        </button>

        <template v-if="confirmingPrune">
          <span class="text-sm text-muted-foreground">Remove unused images. Continue?</span>
          <button
            class="rounded-md bg-destructive px-3 py-1 text-sm text-destructive-foreground hover:bg-destructive/90"
            @click="pruneImages"
          >
            Confirm
          </button>
          <button
            class="rounded-md border px-3 py-1 text-sm hover:bg-accent"
            @click="confirmingPrune = false"
          >
            Cancel
          </button>
        </template>
        <button
          v-else
          class="rounded-md border px-3 py-1 text-sm hover:bg-accent"
          @click="confirmingPrune = true"
        >
          Prune Unused Images
        </button>
      </div>
    </div>

    <!-- Loading skeleton -->
    <div v-if="store.isLoadingImages" class="space-y-2">
      <div v-for="n in 4" :key="n" class="h-10 animate-pulse rounded-md bg-muted" />
    </div>

    <!-- Image table -->
    <div v-else class="overflow-x-auto rounded-md border">
      <table class="w-full text-sm">
        <thead class="bg-muted/50">
          <tr>
            <th class="px-4 py-2 text-left font-medium">Tags</th>
            <th class="px-4 py-2 text-left font-medium">ID</th>
            <th class="px-4 py-2 text-right font-medium">Size (MB)</th>
            <th class="px-4 py-2 text-right font-medium">Actions</th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="img in store.images"
            :key="img.id"
            class="border-t transition-colors hover:bg-muted/30"
          >
            <td class="px-4 py-2">
              <span
                v-for="tag in img.tags"
                :key="tag"
                class="mr-1 rounded-sm bg-muted px-1 font-mono text-xs"
              >{{ tag }}</span>
              <span v-if="img.tags.length === 0" class="text-muted-foreground">&lt;none&gt;</span>
            </td>
            <td class="px-4 py-2 font-mono text-xs">{{ truncateId(img.id) }}</td>
            <td class="px-4 py-2 text-right">{{ formatMb(img.size) }}</td>
            <td class="px-4 py-2 text-right">
              <button
                class="rounded-md border px-2 py-0.5 text-xs hover:bg-destructive hover:text-destructive-foreground"
                @click="removeImage(img.id)"
              >
                Remove
              </button>
            </td>
          </tr>
          <tr v-if="store.images.length === 0">
            <td colspan="4" class="px-4 py-6 text-center text-muted-foreground">No images found</td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
</template>
