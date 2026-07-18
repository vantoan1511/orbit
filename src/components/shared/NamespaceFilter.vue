<script setup lang="ts">
import Select from 'primevue/select'

const modelValue = defineModel<string>({ required: true })
const props = defineProps<{
  namespaces: string[]
}>()
</script>

<template>
  <Select
    v-model="modelValue"
    :options="props.namespaces"
    class="text-xs min-w-44 bg-(--bg-hover)/30 border-(--border)"
  />
</template>
