<script setup lang="ts">
import Button from 'primevue/button'
import InputText from 'primevue/inputtext'

const props = withDefaults(
  defineProps<{
    modelValue: string[]
    title?: string
    placeholder?: string
    addLabel?: string
    disabled?: boolean
  }>(),
  {
    title: '',
    placeholder: 'Value',
    addLabel: 'Add',
    disabled: false
  }
)

const emit = defineEmits<{
  (e: 'update:modelValue', value: string[]): void
}>()

const addItem = () => {
  if (props.disabled) return
  const newList = [...props.modelValue, '']
  emit('update:modelValue', newList)
}

const removeItem = (index: number) => {
  if (props.disabled) return
  const newList = props.modelValue.filter((_, idx) => idx !== index)
  emit('update:modelValue', newList)
}

const updateItem = (index: number, val: string) => {
  if (props.disabled) return
  const newList = props.modelValue.map((item, idx) => (idx === index ? val : item))
  emit('update:modelValue', newList)
}
</script>

<template>
  <div class="flex flex-col gap-2">
    <div class="flex items-center justify-between">
      <label v-if="title" class="text-xs font-medium text-(--text-secondary)">{{ title }}</label>
      <Button
        v-if="!disabled"
        size="small"
        variant="text"
        icon="pi pi-plus"
        :label="addLabel"
        @click="addItem"
      />
    </div>

    <div v-if="modelValue.length === 0" class="text-xs text-(--text-muted) italic py-1">
      No items defined.
    </div>

    <div v-for="(item, idx) in modelValue" :key="'str-' + idx" class="flex items-center gap-2">
      <InputText
        :model-value="item"
        :placeholder="placeholder"
        :disabled="disabled"
        class="flex-1 px-2.5 py-1.5 bg-(--bg-primary) border border-(--border) rounded-md text-xs text-(--text-primary)"
        @update:model-value="(val) => updateItem(idx, val ?? '')"
      />
      <Button
        v-if="!disabled"
        icon="pi pi-times"
        variant="text"
        severity="danger"
        size="small"
        class="p-1! text-red-400 hover:text-red-300 cursor-pointer"
        @click="removeItem(idx)"
      />
    </div>
  </div>
</template>
