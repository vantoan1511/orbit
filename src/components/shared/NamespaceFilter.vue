<script setup lang="ts">
import MultiSelect from 'primevue/multiselect'
import { computed } from 'vue'

const modelValue = defineModel<string[]>({ required: true })
const props = defineProps<{
  namespaces: string[]
}>()

const namespaceOptions = computed(() => props.namespaces.filter((n) => n !== 'All Namespaces'))
</script>

<template>
  <MultiSelect
    v-model="modelValue"
    :options="namespaceOptions"
    placeholder="All Namespaces"
    :maxSelectedLabels="3"
    :filter="true"
    size="small"
    variant="filled"
    class="min-w-44"
  />
</template>
