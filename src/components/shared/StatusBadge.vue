<script setup lang="ts">
import {
  KUBERNETES_EVENT_TYPE,
  KUBERNETES_NAMESPACE_STATUS,
  KUBERNETES_POD_STATUS,
  KUBERNETES_VOLUME_STATUS
} from '@/constants/kubernetes'
import { computed } from 'vue'

const props = defineProps<{
  status: string
}>()

const statusColor = computed(() => {
  switch (props.status) {
    case KUBERNETES_POD_STATUS.Running:
    case KUBERNETES_POD_STATUS.Completed:
    case KUBERNETES_NAMESPACE_STATUS.Active:
    case 'Ready':
    case KUBERNETES_VOLUME_STATUS.Bound:
    case KUBERNETES_VOLUME_STATUS.Available:
    case KUBERNETES_POD_STATUS.Succeeded:
      return 'emerald'
    case KUBERNETES_POD_STATUS.Pending:
    case 'Progressing':
    case KUBERNETES_VOLUME_STATUS.Released:
    case KUBERNETES_POD_STATUS.ContainerCreating:
    case KUBERNETES_EVENT_TYPE.Warning:
      return 'amber'
    case KUBERNETES_POD_STATUS.Failed:
    case KUBERNETES_POD_STATUS.CrashLoopBackOff:
    case KUBERNETES_POD_STATUS.Terminating:
    case KUBERNETES_POD_STATUS.Error:
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
