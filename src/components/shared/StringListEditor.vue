<script setup lang="ts">
import { Plus, X } from '@lucide/vue'
import Button from 'primevue/button'
import InputText from 'primevue/inputtext'

const props = withDefaults(
  defineProps<{
    modelValue: string[]
    title?: string
    placeholder?: string
    addLabel?: string
    disabled?: boolean
    itemValidator?: (val: string, index: number, allItems: string[]) => boolean | string
    errorMessage?: string
  }>(),
  {
    title: '',
    placeholder: 'Value',
    addLabel: 'Add',
    disabled: false,
    errorMessage: 'Invalid value'
  }
)

const emit = defineEmits<{
  (e: 'update:modelValue', value: string[]): void
}>()

const getItemError = (item: string, idx: number): string | null => {
  if (!item || !props.itemValidator) return null
  const res = props.itemValidator(item, idx, props.modelValue)
  if (res === false) return props.errorMessage
  if (typeof res === 'string') return res
  return null
}

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
      <label v-if="title" class="text-xs font-medium text-muted-color">{{ title }}</label>
      <Button
        v-if="!disabled"
        size="small"
        variant="text"
        :label="addLabel"
        class="text-xs"
        @click="addItem"
      >
        <template #icon>
          <Plus class="w-3.5 h-3.5 mr-1" />
        </template>
      </Button>
    </div>

    <div v-if="modelValue.length === 0" class="text-xs text-muted-color italic py-1">
      No items defined.
    </div>

    <div v-for="(item, idx) in modelValue" :key="'str-' + idx" class="flex flex-col gap-1">
      <div class="flex items-center gap-2">
        <InputText
          :model-value="item"
          :placeholder="placeholder"
          :disabled="disabled"
          :invalid="Boolean(getItemError(item, idx))"
          size="small"
          class="flex-1 text-xs"
          @update:model-value="(val) => updateItem(idx, val ?? '')"
        />
        <Button
          v-if="!disabled"
          variant="text"
          severity="danger"
          size="small"
          class="p-1! text-muted-color hover:text-rose-500 cursor-pointer"
          @click="removeItem(idx)"
        >
          <template #icon>
            <X class="w-4 h-4" />
          </template>
        </Button>
      </div>
      <small v-if="getItemError(item, idx)" class="text-(--danger) text-[11px] leading-tight">
        {{ getItemError(item, idx) }}
      </small>
    </div>
  </div>
</template>
