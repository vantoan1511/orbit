<script setup lang="ts">
import Button from 'primevue/button'

defineProps<{
  label: string
  tooltip: string
  severity: 'danger' | 'warn'
  loading?: boolean
}>()

defineEmits<{
  click: []
}>()
</script>

<template>
  <Button
    size="small"
    :severity="severity"
    :loading="loading"
    @click="$emit('click')"
    v-tooltip.top="tooltip"
    class="h-5 px-2 text-[10px] flex items-center gap-1"
  >
    <slot name="icon" />
    <span>{{ label }}</span>
  </Button>
</template>
