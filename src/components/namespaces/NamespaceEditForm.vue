<script setup lang="ts">
import KeyValueEditor, { type KeyValuePair } from '@/components/shared/KeyValueEditor.vue'
import StringListEditor from '@/components/shared/StringListEditor.vue'
import type { Namespace } from 'kubernetes-types/core/v1'
import InputText from 'primevue/inputtext'
import { computed, ref, toRaw, watch } from 'vue'

const props = defineProps<{
  rawData: Namespace | null
}>()

const emit = defineEmits<{
  (e: 'update:rawData', value: Namespace): void
  (e: 'update:isValid', value: boolean): void
}>()

const name = ref('')
const statusPhase = ref('')
const finalizers = ref<string[]>([])
const labels = ref<KeyValuePair[]>([])
const annotations = ref<KeyValuePair[]>([])

let isEmitting = false

const isFormValid = computed(() => {
  return true
})

watch(
  isFormValid,
  (val) => {
    emit('update:isValid', val)
  },
  { immediate: true }
)

const kvObjectToArray = (obj: Record<string, string> | undefined): KeyValuePair[] => {
  if (!obj || typeof obj !== 'object') return []
  return Object.entries(obj).map(([key, value]) => ({ key, value: String(value ?? '') }))
}

const kvArrayToObject = (arr: KeyValuePair[]): Record<string, string> => {
  const res: Record<string, string> = {}
  for (const item of arr) {
    if (item.key.trim()) {
      res[item.key.trim()] = item.value
    }
  }
  return res
}

const syncFromRawData = (data: Namespace | null) => {
  if (!data || isEmitting) return

  // Resource Identity
  name.value = data.metadata?.name || ''
  statusPhase.value = data.status?.phase || 'Active'

  // Spec
  finalizers.value = Array.isArray(data.spec?.finalizers) ? [...data.spec.finalizers] : []

  // Metadata
  labels.value = kvObjectToArray(data.metadata?.labels)
  annotations.value = kvObjectToArray(data.metadata?.annotations)
}

watch(
  () => props.rawData,
  (newData) => {
    syncFromRawData(newData)
  },
  { immediate: true, deep: true }
)

const emitUpdate = () => {
  if (!props.rawData) return
  isEmitting = true

  const rawObj = JSON.parse(JSON.stringify(toRaw(props.rawData))) as Namespace
  if (!rawObj.metadata) rawObj.metadata = {}

  // 1. Metadata
  rawObj.metadata.labels = kvArrayToObject(labels.value)
  rawObj.metadata.annotations = kvArrayToObject(annotations.value)

  // 2. Spec
  const cleanFinalizers = finalizers.value.map((f) => f.trim()).filter(Boolean)
  if (cleanFinalizers.length > 0) {
    if (!rawObj.spec) rawObj.spec = {}
    rawObj.spec.finalizers = cleanFinalizers
  } else if (rawObj.spec) {
    delete rawObj.spec.finalizers
    if (Object.keys(rawObj.spec).length === 0) {
      delete rawObj.spec
    }
  }

  emit('update:rawData', rawObj)

  setTimeout(() => {
    isEmitting = false
  }, 50)
}

const handleFieldChange = () => {
  if (!isEmitting) {
    emitUpdate()
  }
}
</script>

<template>
  <div class="w-full h-full overflow-y-auto pt-2 px-0 flex flex-col gap-10">
    <!-- Section 1: Resource Identity -->
    <div class="grid grid-cols-1 md:grid-cols-12 gap-6 max-w-7xl">
      <div class="md:col-span-4 flex flex-col gap-1">
        <span class="text-xs font-semibold tracking-wider text-muted-color uppercase">
          Resource Identity
        </span>
        <p class="text-xs text-muted-color leading-relaxed">
          Core identity and lifecycle state defined in metadata and status. These fields are
          immutable.
        </p>
      </div>
      <div class="md:col-span-8 flex flex-col gap-3">
        <div class="flex flex-col gap-1.5">
          <label class="text-xs font-medium text-muted-color">Namespace Name</label>
          <InputText disabled v-model="name" size="small" fluid />
        </div>
        <div class="flex flex-col gap-1.5">
          <label class="text-xs font-medium text-muted-color">Status Phase</label>
          <InputText disabled v-model="statusPhase" size="small" fluid />
        </div>
      </div>
    </div>

    <!-- Section 2: Configuration (Spec) -->
    <div class="grid grid-cols-1 md:grid-cols-12 gap-6 max-w-7xl">
      <div class="md:col-span-4 flex flex-col gap-1">
        <span class="text-xs font-semibold tracking-wider text-muted-color uppercase">
          Configuration
        </span>
        <p class="text-xs text-muted-color leading-relaxed">
          Namespace spec configuration including finalizers required before resource deletion.
        </p>
      </div>
      <div class="md:col-span-8 flex flex-col gap-6">
        <div class="p-4 rounded-lg bg-(--bg-hover)/30 flex flex-col gap-3">
          <StringListEditor
            v-model="finalizers"
            title="Finalizers"
            placeholder="e.g. kubernetes"
            add-label="Add Finalizer"
            @update:model-value="handleFieldChange"
          />
        </div>
      </div>
    </div>

    <!-- Section 3: Metadata & Tags -->
    <div class="grid grid-cols-1 md:grid-cols-12 gap-6 max-w-7xl">
      <div class="md:col-span-4 flex flex-col gap-1">
        <span class="text-xs font-semibold tracking-wider text-muted-color uppercase">
          Metadata & Tags
        </span>
        <p class="text-xs text-muted-color leading-relaxed">
          Key-value pairs for organizational indexing, selectors, and Kubernetes annotations.
        </p>
      </div>
      <div class="md:col-span-8 flex flex-col gap-6">
        <div class="p-4 rounded-lg bg-(--bg-hover)/30 flex flex-col gap-3">
          <KeyValueEditor
            v-model="labels"
            title="Namespace Labels"
            add-label="Add Label"
            @update:model-value="handleFieldChange"
          />
        </div>
        <div class="p-4 rounded-lg bg-(--bg-hover)/30 flex flex-col gap-3">
          <KeyValueEditor
            v-model="annotations"
            title="Namespace Annotations"
            add-label="Add Annotation"
            @update:model-value="handleFieldChange"
          />
        </div>
      </div>
    </div>
  </div>
</template>
