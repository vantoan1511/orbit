<script setup lang="ts">
import { Plus, X } from '@lucide/vue'
import Button from 'primevue/button'
import InputText from 'primevue/inputtext'
import Select from 'primevue/select'
import { computed } from 'vue'

import { useKubernetesStore } from '@/stores/kubernetesStore'

export type EnvSourceType =
  | 'Literal'
  | 'ConfigMapKey'
  | 'SecretKey'
  | 'FieldRef'
  | 'ResourceFieldRef'
  | 'ConfigMapEnvFrom'
  | 'SecretEnvFrom'

export interface ContainerEnvItem {
  id?: string
  type: EnvSourceType
  name?: string
  value?: string
  refName?: string
  refKey?: string
  fieldPath?: string
  prefix?: string
}

const props = withDefaults(
  defineProps<{
    modelValue: ContainerEnvItem[]
    namespace?: string
    disabled?: boolean
  }>(),
  {
    namespace: '',
    disabled: false
  }
)

const emit = defineEmits<{
  (e: 'update:modelValue', value: ContainerEnvItem[]): void
}>()

const k8sStore = useKubernetesStore()

const sourceOptions = [
  { label: 'Literal', value: 'Literal' },
  { label: 'ConfigMap Key', value: 'ConfigMapKey' },
  { label: 'Secret Key', value: 'SecretKey' },
  { label: 'Field Ref', value: 'FieldRef' },
  { label: 'Resource Ref', value: 'ResourceFieldRef' },
  { label: 'ConfigMap (All)', value: 'ConfigMapEnvFrom' },
  { label: 'Secret (All)', value: 'SecretEnvFrom' }
]

const fieldRefOptions = [
  'metadata.name',
  'metadata.namespace',
  'metadata.uid',
  'metadata.labels',
  'metadata.annotations',
  'spec.nodeName',
  'spec.serviceAccountName',
  'status.hostIP',
  'status.podIP',
  'status.podIPs'
]

const resourceFieldRefOptions = [
  'limits.cpu',
  'limits.memory',
  'limits.ephemeral-storage',
  'requests.cpu',
  'requests.memory',
  'requests.ephemeral-storage'
]

const configMapOptions = computed(() => {
  const list = props.namespace
    ? k8sStore.configMaps.filter((cm) => !cm.namespace || cm.namespace === props.namespace)
    : k8sStore.configMaps
  const names = list.map((cm) => cm.name).filter(Boolean)
  return Array.from(new Set(names))
})

const secretOptions = computed(() => {
  const list = props.namespace
    ? k8sStore.secrets.filter((s) => !s.namespace || s.namespace === props.namespace)
    : k8sStore.secrets
  const names = list.map((s) => s.name).filter(Boolean)
  return Array.from(new Set(names))
})

const getKeysFor = (item: ContainerEnvItem): string[] => {
  if (!item.refName) return []
  if (item.type === 'ConfigMapKey') {
    const cm = k8sStore.configMaps.find(
      (c) => c.name === item.refName && (!props.namespace || c.namespace === props.namespace)
    )
    return cm?.data ? Object.keys(cm.data) : []
  }
  if (item.type === 'SecretKey') {
    const sec = k8sStore.secrets.find(
      (s) => s.name === item.refName && (!props.namespace || s.namespace === props.namespace)
    )
    return sec?.data ? Object.keys(sec.data) : []
  }
  return []
}

const isEnvFrom = (type: EnvSourceType) => {
  return type === 'ConfigMapEnvFrom' || type === 'SecretEnvFrom'
}

const addEnv = () => {
  if (props.disabled) return
  const newList = [...props.modelValue, { type: 'Literal' as EnvSourceType, name: '', value: '' }]
  emit('update:modelValue', newList)
}

const removeEnv = (index: number) => {
  if (props.disabled) return
  const newList = props.modelValue.filter((_, idx) => idx !== index)
  emit('update:modelValue', newList)
}

const updateEnvField = <K extends keyof ContainerEnvItem>(
  index: number,
  field: K,
  value: ContainerEnvItem[K]
) => {
  if (props.disabled) return
  const newList = props.modelValue.map((item, idx) =>
    idx === index ? { ...item, [field]: value } : item
  )

  // Clean up fields if type changes
  if (field === 'type') {
    const updatedItem = newList[index]
    if (updatedItem) {
      const newType = value as EnvSourceType

      // Reset irrelevant fields when switching types to prevent stale data
      if (newType === 'Literal') {
        delete updatedItem.refName
        delete updatedItem.refKey
        delete updatedItem.fieldPath
        delete updatedItem.prefix
      } else if (isEnvFrom(newType)) {
        delete updatedItem.name
        delete updatedItem.value
        delete updatedItem.refKey
        delete updatedItem.fieldPath
        updatedItem.prefix = updatedItem.prefix || ''
        updatedItem.refName = updatedItem.refName || ''
      } else if (newType === 'FieldRef' || newType === 'ResourceFieldRef') {
        delete updatedItem.value
        delete updatedItem.refName
        delete updatedItem.refKey
        delete updatedItem.prefix
        updatedItem.fieldPath = updatedItem.fieldPath || ''
      } else if (newType === 'ConfigMapKey' || newType === 'SecretKey') {
        delete updatedItem.value
        delete updatedItem.fieldPath
        delete updatedItem.prefix
        updatedItem.refName = updatedItem.refName || ''
        updatedItem.refKey = updatedItem.refKey || ''
      }
    }
  }

  emit('update:modelValue', newList)
}
</script>

<template>
  <div class="flex flex-col gap-2">
    <div class="flex items-center justify-between">
      <label class="text-xs font-medium text-muted-color">Environment Variables</label>
      <Button
        v-if="!disabled"
        size="small"
        variant="text"
        label="Add Env"
        class="text-xs"
        @click="addEnv"
      >
        <template #icon>
          <Plus class="w-3.5 h-3.5 mr-1" />
        </template>
      </Button>
    </div>

    <div v-if="modelValue.length === 0" class="text-xs text-muted-color italic py-1">
      No environment variables configured.
    </div>

    <div
      v-for="(item, idx) in modelValue"
      :key="'env-' + idx"
      class="flex items-center gap-1.5 w-full"
    >
      <!-- Source Type Selector -->
      <Select
        :model-value="item.type"
        :options="sourceOptions"
        optionLabel="label"
        optionValue="value"
        :disabled="disabled"
        size="small"
        class="w-40 shrink-0 text-xs"
        @update:model-value="(val) => updateEnvField(idx, 'type', val as EnvSourceType)"
      />

      <!-- Fields for envFrom (All from ConfigMap / Secret) -->
      <template v-if="isEnvFrom(item.type)">
        <Select
          :model-value="item.refName || ''"
          :options="item.type === 'ConfigMapEnvFrom' ? configMapOptions : secretOptions"
          editable
          :placeholder="item.type === 'ConfigMapEnvFrom' ? 'ConfigMap Name' : 'Secret Name'"
          :disabled="disabled"
          size="small"
          class="flex-1 min-w-36 font-mono text-xs"
          @update:model-value="(val) => updateEnvField(idx, 'refName', val ?? '')"
        />
        <InputText
          :model-value="item.prefix || ''"
          placeholder="Prefix (optional)"
          :disabled="disabled"
          size="small"
          class="w-36 shrink-0 font-mono text-xs"
          @update:model-value="(val) => updateEnvField(idx, 'prefix', val ?? '')"
        />
      </template>

      <!-- Fields for individual variables -->
      <template v-else>
        <InputText
          :model-value="item.name || ''"
          placeholder="NAME"
          :disabled="disabled"
          size="small"
          class="w-44 shrink-0 font-mono text-xs"
          @update:model-value="(val) => updateEnvField(idx, 'name', val ?? '')"
        />
        <span class="text-muted-color text-xs shrink-0">=</span>

        <!-- Literal -->
        <template v-if="item.type === 'Literal'">
          <InputText
            :model-value="item.value || ''"
            placeholder="VALUE"
            :disabled="disabled"
            size="small"
            class="flex-1 min-w-36 font-mono text-xs"
            @update:model-value="(val) => updateEnvField(idx, 'value', val ?? '')"
          />
        </template>

        <!-- ConfigMap / Secret Key Reference -->
        <template v-else-if="item.type === 'ConfigMapKey' || item.type === 'SecretKey'">
          <Select
            :model-value="item.refName || ''"
            :options="item.type === 'ConfigMapKey' ? configMapOptions : secretOptions"
            editable
            :placeholder="item.type === 'ConfigMapKey' ? 'ConfigMap Name' : 'Secret Name'"
            :disabled="disabled"
            size="small"
            class="flex-1 min-w-36 font-mono text-xs"
            @update:model-value="(val) => updateEnvField(idx, 'refName', val ?? '')"
          />
          <Select
            :model-value="item.refKey || ''"
            :options="getKeysFor(item)"
            editable
            placeholder="Key"
            :disabled="disabled"
            size="small"
            class="w-36 shrink-0 font-mono text-xs"
            @update:model-value="(val) => updateEnvField(idx, 'refKey', val ?? '')"
          />
        </template>

        <!-- Field References -->
        <template v-else-if="item.type === 'FieldRef'">
          <Select
            :model-value="item.fieldPath || ''"
            :options="fieldRefOptions"
            editable
            placeholder="Field Path (e.g. metadata.name)"
            :disabled="disabled"
            size="small"
            class="flex-1 min-w-36 font-mono text-xs"
            @update:model-value="(val) => updateEnvField(idx, 'fieldPath', val ?? '')"
          />
        </template>

        <!-- Resource Field References -->
        <template v-else-if="item.type === 'ResourceFieldRef'">
          <Select
            :model-value="item.fieldPath || ''"
            :options="resourceFieldRefOptions"
            editable
            placeholder="Resource (e.g. limits.cpu)"
            :disabled="disabled"
            size="small"
            class="flex-1 min-w-36 font-mono text-xs"
            @update:model-value="(val) => updateEnvField(idx, 'fieldPath', val ?? '')"
          />
        </template>
      </template>

      <!-- Delete Action -->
      <Button
        v-if="!disabled"
        variant="text"
        severity="danger"
        size="small"
        class="p-1! shrink-0 text-muted-color hover:text-rose-500 cursor-pointer"
        @click="removeEnv(idx)"
      >
        <template #icon>
          <X class="w-4 h-4" />
        </template>
      </Button>
    </div>
  </div>
</template>
