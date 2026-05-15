<script setup lang="ts">
import { ref } from 'vue'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardFooter, CardHeader } from '@/components/ui/card'

defineProps<{
  projectName: string
  commandLabel: string
}>()

const emit = defineEmits<{
  confirm: []
  cancel: []
}>()

const typedName = ref('')

function onConfirm() {
  emit('confirm')
  typedName.value = ''
}
</script>

<template>
  <!-- Backdrop -->
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/60">
    <Card class="w-full max-w-md mx-4">
      <CardHeader class="pb-2">
        <p class="text-base font-semibold">Confirm destructive action</p>
      </CardHeader>

      <CardContent class="space-y-3">
        <p class="text-sm text-muted-foreground">
          This action cannot be undone. Type
          <strong class="text-foreground">{{ projectName }}</strong>
          to confirm.
        </p>
        <input
          v-model="typedName"
          type="text"
          :placeholder="projectName"
          class="w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background focus:outline-none focus:ring-2 focus:ring-ring"
          autofocus
          @keyup.enter="typedName === projectName && onConfirm()"
          @keyup.escape="emit('cancel')"
        />
      </CardContent>

      <CardFooter class="flex justify-end gap-2">
        <Button variant="outline" @click="emit('cancel')">Cancel</Button>
        <Button
          variant="destructive"
          :disabled="typedName !== projectName"
          @click="onConfirm"
        >
          Run {{ commandLabel }}
        </Button>
      </CardFooter>
    </Card>
  </div>
</template>
