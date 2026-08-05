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
    class="min-w-44 text-sm"
  />
</template>
