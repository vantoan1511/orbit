<script setup lang="ts">
import KeyValueEditor, { type KeyValuePair } from '@/components/shared/KeyValueEditor.vue'
import type { KubernetesResource } from '@/types/kubernetes'
import {
  extractPVCFormValues,
  applyPVCFormToResource,
  validatePVCStorageExpansion,
  type PVCFormValues
} from '@/utils/storageForms'
import { isValidStorageQuantity } from '@/utils/storageQuantity'
import InputText from 'primevue/inputtext'
import Tag from 'primevue/tag'
import { computed, ref, toRaw, watch } from 'vue'

const props = defineProps<{
  rawData: KubernetesResource | Record<string, unknown> | null
}>()

const emit = defineEmits<{
  (e: 'update:rawData', value: Record<string, unknown>): void
  (e: 'update:isValid', value: boolean): void
}>()

const name = ref('')
const namespace = ref('default')
const storageRequest = ref('10Gi')
const initialStorageRequest = ref('10Gi')
const accessModes = ref<string[]>(['ReadWriteOnce'])
const storageClassName = ref('')
const volumeName = ref('')
const volumeMode = ref('Filesystem')
const labels = ref<KeyValuePair[]>([])
const annotations = ref<KeyValuePair[]>([])

let isEmitting = false

const isStorageFormatValid = computed(() => isValidStorageQuantity(storageRequest.value))

const isStorageExpansionValid = computed(() => {
  if (!isStorageFormatValid.value) return false
  if (!initialStorageRequest.value) return true
  return validatePVCStorageExpansion(initialStorageRequest.value, storageRequest.value)
})

const isFormValid = computed(() => {
  return isStorageFormatValid.value && isStorageExpansionValid.value
})

watch(
  isFormValid,
  (val) => {
    emit('update:isValid', val)
  },
  { immediate: true }
)

const syncFromRawData = (data: KubernetesResource | Record<string, unknown> | null) => {
  if (!data || isEmitting) return
  const form = extractPVCFormValues(data)
  name.value = form.name
  namespace.value = form.namespace
  storageRequest.value = form.storageRequest
  initialStorageRequest.value = form.storageRequest
  accessModes.value = [...form.accessModes]
  storageClassName.value = form.storageClassName
  volumeName.value = form.volumeName
  volumeMode.value = form.volumeMode
  labels.value = form.labels.map((l) => ({ key: l.key, value: l.value }))
  annotations.value = form.annotations.map((a) => ({ key: a.key, value: a.value }))
}

watch(
  () => props.rawData,
  (newData) => {
    syncFromRawData(newData)
  },
  { immediate: true, deep: true }
)

const handleFieldChange = () => {
  if (!props.rawData) return
  isEmitting = true

  const rawCopy = JSON.parse(JSON.stringify(toRaw(props.rawData))) as Record<string, unknown>
  const formVals: PVCFormValues = {
    name: name.value,
    namespace: namespace.value,
    storageRequest: storageRequest.value,
    accessModes: accessModes.value,
    storageClassName: storageClassName.value,
    volumeName: volumeName.value,
    volumeMode: volumeMode.value,
    labels: labels.value.map((l) => ({ key: l.key, value: l.value })),
    annotations: annotations.value.map((a) => ({ key: a.key, value: a.value }))
  }

  const updated = applyPVCFormToResource(rawCopy, formVals)
  emit('update:rawData', updated)

  setTimeout(() => {
    isEmitting = false
  }, 0)
}
</script>

<template>
  <div
    data-component="persistent-volume-claim-edit-form"
    class="w-full h-full overflow-y-auto pt-2 flex flex-col gap-8 pb-12"
  >
    <!-- Section 1: Resource Identity & Spec (Immutable) -->
    <div class="grid grid-cols-1 md:grid-cols-12 gap-6 max-w-7xl">
      <div class="md:col-span-4 flex flex-col gap-1">
        <span class="text-xs font-semibold tracking-wider text-muted-color uppercase">
          Identity & Binding
        </span>
        <p class="text-xs text-muted-color leading-relaxed">
          PVC claim identity and bound volume mapping. These fields are immutable once bound.
        </p>
      </div>
      <div class="md:col-span-8 flex flex-col gap-3">
        <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
          <div class="flex flex-col gap-1.5">
            <label class="text-xs font-medium text-muted-color">Claim Name</label>
            <InputText disabled :model-value="name" fluid />
          </div>
          <div class="flex flex-col gap-1.5">
            <label class="text-xs font-medium text-muted-color">Namespace</label>
            <InputText disabled :model-value="namespace" fluid />
          </div>
        </div>

        <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
          <div class="flex flex-col gap-1.5">
            <label class="text-xs font-medium text-muted-color">Bound Volume</label>
            <InputText disabled :model-value="volumeName || '-'" fluid />
          </div>
          <div class="flex flex-col gap-1.5">
            <label class="text-xs font-medium text-muted-color">Storage Class</label>
            <InputText disabled :model-value="storageClassName || '-'" fluid />
          </div>
        </div>

        <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
          <div class="flex flex-col gap-1.5">
            <label class="text-xs font-medium text-muted-color">Volume Mode</label>
            <InputText disabled :model-value="volumeMode" fluid />
          </div>
          <div class="flex flex-col gap-1.5">
            <label class="text-xs font-medium text-muted-color">Access Modes</label>
            <div class="flex flex-wrap gap-1 mt-1">
              <Tag
                v-for="mode in accessModes"
                :key="mode"
                severity="secondary"
                class="font-mono text-[11px]"
                :value="mode"
              />
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Section 2: Storage Capacity Expansion -->
    <div class="grid grid-cols-1 md:grid-cols-12 gap-6 max-w-7xl">
      <div class="md:col-span-4 flex flex-col gap-1">
        <span class="text-xs font-semibold tracking-wider text-muted-color uppercase">
          Capacity Expansion
        </span>
        <p class="text-xs text-muted-color leading-relaxed">
          Expand volume capacity for resizable StorageClasses. Storage requests can only be
          increased in Kubernetes.
        </p>
      </div>
      <div class="md:col-span-8 flex flex-col gap-3">
        <div class="flex flex-col gap-1.5">
          <label class="text-xs font-medium text-muted-color">Requested Storage</label>
          <InputText
            v-model="storageRequest"
            placeholder="e.g. 20Gi"
            :invalid="!isFormValid"
            fluid
            @update:model-value="handleFieldChange"
          />
          <span v-if="!isStorageFormatValid" class="text-[11px] text-(--danger)">
            Valid storage quantity required (e.g. 20Gi, 500Mi).
          </span>
          <span v-else-if="!isStorageExpansionValid" class="text-[11px] text-(--danger)">
            Storage request can only be expanded, not reduced.
          </span>
          <span v-else class="text-[11px] text-muted-color">
            Modifying this field triggers Kubernetes volume expansion if supported by the
            provisioner.
          </span>
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
          Key-value pairs for organizational indexing and Kubernetes annotations.
        </p>
      </div>
      <div class="md:col-span-8 flex flex-col gap-6">
        <div class="p-4 rounded-[var(--ui-radius-card)] bg-(--bg-hover)/30 flex flex-col gap-3">
          <KeyValueEditor
            v-model="labels"
            title="Labels"
            add-label="Add Label"
            @update:model-value="handleFieldChange"
          />
        </div>
        <div class="p-4 rounded-[var(--ui-radius-card)] bg-(--bg-hover)/30 flex flex-col gap-3">
          <KeyValueEditor
            v-model="annotations"
            title="Annotations"
            add-label="Add Annotation"
            @update:model-value="handleFieldChange"
          />
        </div>
      </div>
    </div>
  </div>
</template>
