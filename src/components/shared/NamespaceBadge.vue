<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{
  namespace: string
}>()

const severity = computed(() => {
  switch (props.namespace) {
    case 'kube-system':
      return 'secondary'
    case 'monitoring':
      return 'warn'
    case 'logging':
      return 'info'
    default:
      return 'secondary'
  }
})
</script>

<template>
  <Tag class="font-mono" :severity="severity" :value="props.namespace" />
</template>
