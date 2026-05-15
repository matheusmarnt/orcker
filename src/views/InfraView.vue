<script setup lang="ts">
import ImageList from '@/components/infra/ImageList.vue'
import VolumeList from '@/components/infra/VolumeList.vue'
import TemplateMarketplace from '@/components/infra/TemplateMarketplace.vue'
</script>

<template>
  <div class="space-y-8 p-6">
    <div>
      <h2 class="mb-1 text-xl font-bold">Infrastructure</h2>
      <p class="text-sm text-muted-foreground">Manage Docker volumes, images, and templates</p>
    </div>

    <section>
      <VolumeList />
    </section>

    <hr class="border-border" />

    <section>
      <ImageList />
    </section>

    <hr class="border-border" />

    <section>
      <TemplateMarketplace />
    </section>
  </div>
</template>
