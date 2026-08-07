<script setup lang="ts">
defineProps<{
  title?: string
}>()
</script>

<template>
  <div class="flex flex-col gap-6 h-full">
    <!-- Header/Title -->
    <div v-if="title || $slots.actions" class="flex items-center justify-between shrink-0">
      <h2 v-if="title" class="text-xl font-bold tracking-tight text-primary">{{ title }}</h2>
      <div v-if="$slots.actions" class="flex items-center gap-3">
        <slot name="actions" />
      </div>
    </div>

    <!-- Content -->
    <div class="flex-1 min-h-0 flex flex-col gap-6">
      <slot />
    </div>
  </div>
</template>
