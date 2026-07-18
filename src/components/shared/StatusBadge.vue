<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{
  status: string
}>()

const statusColor = computed(() => {
  switch (props.status) {
    case 'Running':
    case 'Completed':
    case 'Active':
      return 'emerald'
    case 'Pending':
    case 'Progressing':
      return 'amber'
    case 'Failed':
    case 'CrashLoopBackOff':
    case 'Terminating':
      return 'rose'
    default:
      return 'gray'
  }
})

const dotClass = computed(() => {
  switch (statusColor.value) {
    case 'emerald':
      return 'bg-emerald-500'
    case 'amber':
      return 'bg-amber-500'
    case 'rose':
      return 'bg-rose-500'
    default:
      return 'bg-gray-400'
  }
})

const textClass = computed(() => {
  switch (statusColor.value) {
    case 'emerald':
      return 'text-emerald-500'
    case 'amber':
      return 'text-amber-500'
    case 'rose':
      return 'text-rose-500'
    default:
      return 'text-gray-400'
  }
})
</script>

<template>
  <div class="flex items-center gap-1.5">
    <span class="w-1.5 h-1.5 rounded-full" :class="dotClass"></span>
    <span class="font-medium" :class="textClass">
      {{ props.status }}
    </span>
  </div>
</template>
