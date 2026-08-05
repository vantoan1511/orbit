<script setup lang="ts">
import Button from 'primevue/button'
import InputNumber from 'primevue/inputnumber'
import InputText from 'primevue/inputtext'
import Select from 'primevue/select'

export interface ContainerPortItem {
  name: string
  containerPort: number
  protocol: string
}

const props = withDefaults(
  defineProps<{
    modelValue: ContainerPortItem[]
    disabled?: boolean
  }>(),
  {
    disabled: false
  }
)

const emit = defineEmits<{
  (e: 'update:modelValue', value: ContainerPortItem[]): void
}>()

const protocolOptions = ['TCP', 'UDP', 'SCTP']

const addPort = () => {
  if (props.disabled) return
  const newList = [...props.modelValue, { name: '', containerPort: 80, protocol: 'TCP' }]
  emit('update:modelValue', newList)
}

const removePort = (index: number) => {
  if (props.disabled) return
  const newList = props.modelValue.filter((_, idx) => idx !== index)
  emit('update:modelValue', newList)
}

const updatePortField = <K extends keyof ContainerPortItem>(
  index: number,
  field: K,
  value: ContainerPortItem[K]
) => {
  if (props.disabled) return
  const newList = props.modelValue.map((item, idx) =>
    idx === index ? { ...item, [field]: value } : item
  )
  emit('update:modelValue', newList)
}
</script>

<template>
  <div class="flex flex-col gap-2">
    <div class="flex items-center justify-between">
      <label class="text-xs font-medium text-muted-color">Container Ports</label>
      <Button
        v-if="!disabled"
        size="small"
        variant="text"
        icon="pi pi-plus"
        label="Add Port"
        class="text-xs"
        @click="addPort"
      />
    </div>

    <div v-if="modelValue.length === 0" class="text-xs text-muted-color italic py-1">
      No ports configured.
    </div>

    <div v-for="(port, idx) in modelValue" :key="'port-' + idx" class="flex items-center gap-2">
      <InputText
        :model-value="port.name"
        placeholder="Port Name (e.g. http)"
        :disabled="disabled"
        class="w-1/3 px-2.5 py-1.5 bg-(--bg-primary) border border-(--border) rounded-md text-xs text-primary"
        @update:model-value="(val) => updatePortField(idx, 'name', val ?? '')"
      />
      <InputNumber
        :model-value="port.containerPort"
        placeholder="Port"
        :disabled="disabled"
        :min="1"
        :max="65535"
        class="w-1/3 text-xs"
        input-class="w-full px-2.5 py-1.5 bg-(--bg-primary) border border-(--border) rounded-md text-xs text-primary"
        @update:model-value="(val) => updatePortField(idx, 'containerPort', Number(val ?? 80))"
      />
      <Select
        :model-value="port.protocol"
        :options="protocolOptions"
        :disabled="disabled"
        class="w-1/4 bg-(--bg-primary) border border-(--border) text-xs"
        @update:model-value="(val) => updatePortField(idx, 'protocol', val ?? 'TCP')"
      />
      <Button
        v-if="!disabled"
        icon="pi pi-times"
        variant="text"
        severity="danger"
        size="small"
        class="p-1! text-red-400 hover:text-red-300 cursor-pointer"
        @click="removePort(idx)"
      />
    </div>
  </div>
</template>
