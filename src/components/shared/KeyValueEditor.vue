<script setup lang="ts">
import Button from 'primevue/button'
import InputText from 'primevue/inputtext'

export interface KeyValuePair {
  key: string
  value: string
}

const props = withDefaults(
  defineProps<{
    modelValue: KeyValuePair[]
    title?: string
    keyPlaceholder?: string
    valuePlaceholder?: string
    addLabel?: string
    disabled?: boolean
    readonlyKeys?: boolean
  }>(),
  {
    title: '',
    keyPlaceholder: 'Key',
    valuePlaceholder: 'Value',
    addLabel: 'Add',
    disabled: false,
    readonlyKeys: false
  }
)

const emit = defineEmits<{
  (e: 'update:modelValue', value: KeyValuePair[]): void
}>()

const addItem = () => {
  if (props.disabled) return
  const newList = [...props.modelValue, { key: '', value: '' }]
  emit('update:modelValue', newList)
}

const removeItem = (index: number) => {
  if (props.disabled) return
  const newList = props.modelValue.filter((_, idx) => idx !== index)
  emit('update:modelValue', newList)
}

const updateKey = (index: number, newKey: string) => {
  if (props.disabled || props.readonlyKeys) return
  const newList = props.modelValue.map((item, idx) =>
    idx === index ? { ...item, key: newKey } : item
  )
  emit('update:modelValue', newList)
}

const updateValue = (index: number, newValue: string) => {
  if (props.disabled) return
  const newList = props.modelValue.map((item, idx) =>
    idx === index ? { ...item, value: newValue } : item
  )
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
        class="text-xs"
        @click="addItem"
      />
    </div>

    <div v-if="modelValue.length === 0" class="text-xs text-(--text-muted) italic py-1">
      No items defined.
    </div>

    <div v-for="(item, idx) in modelValue" :key="'kv-' + idx" class="flex items-center gap-2">
      <InputText
        :model-value="item.key"
        :placeholder="keyPlaceholder"
        :disabled="disabled || readonlyKeys"
        class="w-1/2 px-2 py-1.5 bg-(--bg-primary) border border-(--border) rounded-md text-xs text-(--text-primary)"
        @update:model-value="(val) => updateKey(idx, val ?? '')"
      />
      <span class="text-(--text-secondary)">=</span>
      <InputText
        :model-value="item.value"
        :placeholder="valuePlaceholder"
        :disabled="disabled"
        class="w-1/2 px-2 py-1.5 bg-(--bg-primary) border border-(--border) rounded-md text-xs text-(--text-primary)"
        @update:model-value="(val) => updateValue(idx, val ?? '')"
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
