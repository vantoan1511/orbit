<script setup lang="ts">
import { computed } from 'vue'
import { getNamespaceSeverity } from '@/utils/badgeStyles'

const props = defineProps<{
  namespace: string
}>()

const severity = computed(() => getNamespaceSeverity(props.namespace))
</script>

<template>
  <Tag class="font-mono" :severity="severity" :value="props.namespace" />
</template>
