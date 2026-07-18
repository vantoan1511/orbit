<script setup lang="ts">
import MultiSelect from 'primevue/multiselect'

const modelValue = defineModel<string[]>({ required: true })
const props = defineProps<{
  namespaces: string[]
}>()
</script>

<template>
  <MultiSelect
    v-model="modelValue"
    :options="props.namespaces.filter((n) => n !== 'All Namespaces')"
    placeholder="All Namespaces"
    :maxSelectedLabels="3"
    class="text-xs min-w-44 bg-(--bg-hover)/30 border-(--border)"
  />
</template>
