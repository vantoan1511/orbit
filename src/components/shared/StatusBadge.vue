<script setup lang="ts">
// Consumes canonical status tokens via badgeStyles: bg-(--text-muted), text-(--text-muted)
import { computed } from 'vue'
import { getStatusVisualStyle } from '@/utils/badgeStyles'

const props = defineProps<{
  status: string
}>()

const visualStyle = computed(() => getStatusVisualStyle(props.status))
</script>

<template>
  <div data-component="status-badge" class="flex items-center gap-1.5">
    <span class="w-1.5 h-1.5 rounded-full" :class="visualStyle.dotClass"></span>
    <span class="font-medium" :class="visualStyle.textClass">
      {{ props.status }}
    </span>
  </div>
</template>
