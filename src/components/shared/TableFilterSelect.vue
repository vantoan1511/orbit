<script setup lang="ts">
/* eslint-disable @typescript-eslint/no-explicit-any */
import Select from 'primevue/select'

const modelValue = defineModel<any>({ required: true })

const props = withDefaults(
  defineProps<{
    options: any[]
    placeholder?: string
    variant?: 'filled' | 'outlined'
    size?: 'small' | 'large'
  }>(),
  {
    variant: 'filled',
    size: 'small'
  }
)
</script>

<template>
  <Select
    v-model="modelValue"
    :options="props.options"
    :placeholder="props.placeholder"
    :variant="props.variant"
    :size="props.size"
    class="min-w-40"
  />
</template>
