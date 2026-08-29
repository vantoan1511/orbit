<script setup lang="ts">
import ResourceYamlTab from '@/components/shared/ResourceYamlTab.vue'

defineProps<{
  displayedYaml: string
  isYamlLoading: boolean
  copied: boolean
}>()

const emit = defineEmits<{
  (e: 'copy-yaml'): void
}>()
</script>

<template>
  <ResourceYamlTab
    :displayed-yaml="displayedYaml"
    :is-yaml-loading="isYamlLoading"
    :copied="copied"
    @copy-yaml="emit('copy-yaml')"
  />
</template>
