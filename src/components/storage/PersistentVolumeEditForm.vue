<script setup lang="ts">
import KeyValueEditor, { type KeyValuePair } from '@/components/shared/KeyValueEditor.vue'
import StringListEditor from '@/components/shared/StringListEditor.vue'
import { extractPVFormValues, applyPVFormToResource, type PVFormValues } from '@/utils/storageForms'
import { isValidStorageQuantity } from '@/utils/storageQuantity'
import InputText from 'primevue/inputtext'
import Select from 'primevue/select'
import { computed, ref, toRaw, watch } from 'vue'

const props = defineProps<{
  rawData: Record<string, unknown> | null
}>()

const emit = defineEmits<{
  (e: 'update:rawData', value: Record<string, unknown>): void
  (e: 'update:isValid', value: boolean): void
}>()

const name = ref('')
const capacity = ref('10Gi')
const accessModes = ref<string[]>(['ReadWriteOnce'])
const reclaimPolicy = ref('Retain')
const storageClassName = ref('')
const volumeMode = ref('Filesystem')
const mountOptions = ref<string[]>([])
const sourceSummary = ref('')
const claimRefSummary = ref('-')
const labels = ref<KeyValuePair[]>([])
const annotations = ref<KeyValuePair[]>([])

let isEmitting = false

const reclaimPolicyOptions = ['Retain', 'Delete', 'Recycle']

const isCapacityValid = computed(() => isValidStorageQuantity(capacity.value))

const isFormValid = computed(() => {
  return isCapacityValid.value
})

watch(
  isFormValid,
  (val) => {
    emit('update:isValid', val)
  },
  { immediate: true }
)

const syncFromRawData = (data: Record<string, unknown> | null) => {
  if (!data || isEmitting) return
  const form = extractPVFormValues(data)
  name.value = form.name
  capacity.value = form.capacity
  accessModes.value = [...form.accessModes]
  reclaimPolicy.value = form.reclaimPolicy
  storageClassName.value = form.storageClassName
  volumeMode.value = form.volumeMode
  mountOptions.value = [...form.mountOptions]
  sourceSummary.value = form.sourceSummary
  claimRefSummary.value = form.claimRefSummary
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
  const formVals: PVFormValues = {
    name: name.value,
    capacity: capacity.value,
    accessModes: accessModes.value,
    reclaimPolicy: reclaimPolicy.value,
    storageClassName: storageClassName.value,
    volumeMode: volumeMode.value,
    mountOptions: mountOptions.value.map((m) => m.trim()).filter(Boolean),
    sourceSummary: sourceSummary.value,
    claimRefSummary: claimRefSummary.value,
    finalizers: ((rawCopy.metadata as Record<string, unknown>)?.finalizers as string[]) || [],
    labels: labels.value.map((l) => ({ key: l.key, value: l.value })),
    annotations: annotations.value.map((a) => ({ key: a.key, value: a.value }))
  }

  const updated = applyPVFormToResource(rawCopy, formVals)
  emit('update:rawData', updated)

  setTimeout(() => {
    isEmitting = false
  }, 0)
}
</script>

<template>
  <div data-component="persistent-volume-edit-form" class="flex flex-col gap-8 pb-12">
    <!-- Section 1: Resource Identity & Status (Read-Only) -->
    <div class="grid grid-cols-1 md:grid-cols-12 gap-6 max-w-7xl">
      <div class="md:col-span-4 flex flex-col gap-1">
        <span class="text-xs font-semibold tracking-wider text-muted-color uppercase">
          Identity & Source
        </span>
        <p class="text-xs text-muted-color leading-relaxed">
          PersistentVolume identity, underlying storage source plugin, and bound PVC claim. These
          fields are immutable.
        </p>
      </div>
      <div class="md:col-span-8 flex flex-col gap-3">
        <div class="flex flex-col gap-1.5">
          <label class="text-xs font-medium text-muted-color">Volume Name</label>
          <InputText disabled :model-value="name" fluid />
        </div>
        <div class="flex flex-col gap-1.5">
          <label class="text-xs font-medium text-muted-color">Volume Source</label>
          <InputText disabled :model-value="sourceSummary" fluid />
        </div>
        <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
          <div class="flex flex-col gap-1.5">
            <label class="text-xs font-medium text-muted-color">Volume Mode</label>
            <InputText disabled :model-value="volumeMode" fluid />
          </div>
          <div class="flex flex-col gap-1.5">
            <label class="text-xs font-medium text-muted-color">Claim Reference</label>
            <InputText disabled :model-value="claimRefSummary" fluid />
          </div>
        </div>
      </div>
    </div>

    <!-- Section 2: Storage Configuration -->
    <div class="grid grid-cols-1 md:grid-cols-12 gap-6 max-w-7xl">
      <div class="md:col-span-4 flex flex-col gap-1">
        <span class="text-xs font-semibold tracking-wider text-muted-color uppercase">
          Configuration
        </span>
        <p class="text-xs text-muted-color leading-relaxed">
          Storage capacity, reclaim policy, storage class, and filesystem mount options.
        </p>
      </div>
      <div class="md:col-span-8 flex flex-col gap-4">
        <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
          <div class="flex flex-col gap-1.5">
            <label class="text-xs font-medium text-muted-color">Capacity</label>
            <InputText
              v-model="capacity"
              placeholder="e.g. 10Gi, 500Mi"
              :invalid="!isCapacityValid"
              fluid
              @update:model-value="handleFieldChange"
            />
            <span v-if="!isCapacityValid" class="text-[11px] text-(--danger)">
              Valid quantity required (e.g. 20Gi, 500Mi).
            </span>
          </div>

          <div class="flex flex-col gap-1.5">
            <label class="text-xs font-medium text-muted-color">Reclaim Policy</label>
            <Select
              v-model="reclaimPolicy"
              :options="reclaimPolicyOptions"
              fluid
              @change="handleFieldChange"
            />
          </div>
        </div>

        <div class="flex flex-col gap-1.5">
          <label class="text-xs font-medium text-muted-color">Storage Class Name</label>
          <InputText
            v-model="storageClassName"
            placeholder="e.g. standard, manual"
            fluid
            @update:model-value="handleFieldChange"
          />
        </div>

        <div class="p-4 rounded-[var(--ui-radius-card)] bg-(--bg-hover)/30 flex flex-col gap-3">
          <StringListEditor
            v-model="mountOptions"
            title="Mount Options"
            placeholder="e.g. ro, noatime"
            add-label="Add Mount Option"
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
