<script setup lang="ts">
import { Plus, X } from '@lucide/vue'
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
        label="Add Port"
        class="text-xs"
        @click="addPort"
      >
        <template #icon>
          <Plus class="w-3.5 h-3.5 mr-1" />
        </template>
      </Button>
    </div>

    <div v-if="modelValue.length === 0" class="text-xs text-muted-color italic py-1">
      No ports configured.
    </div>

    <div v-for="(port, idx) in modelValue" :key="'port-' + idx" class="flex items-center gap-2">
      <InputText
        :model-value="port.name"
        placeholder="Port Name (e.g. http)"
        :disabled="disabled"
        size="small"
        class="w-1/3"
        @update:model-value="(val) => updatePortField(idx, 'name', val ?? '')"
      />
      <InputNumber
        :model-value="port.containerPort"
        placeholder="Port"
        :disabled="disabled"
        :min="1"
        :max="65535"
        size="small"
        class="w-1/3"
        fluid
        @update:model-value="(val) => updatePortField(idx, 'containerPort', Number(val ?? 80))"
      />
      <Select
        :model-value="port.protocol"
        :options="protocolOptions"
        :disabled="disabled"
        size="small"
        class="w-1/4"
        @update:model-value="(val) => updatePortField(idx, 'protocol', val ?? 'TCP')"
      />
      <Button
        v-if="!disabled"
        variant="text"
        severity="danger"
        size="small"
        class="p-1! text-muted-color hover:text-rose-500 cursor-pointer"
        @click="removePort(idx)"
      >
        <template #icon>
          <X class="w-4 h-4" />
        </template>
      </Button>
    </div>
  </div>
</template>
