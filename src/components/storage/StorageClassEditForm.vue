<script setup lang="ts">
import KeyValueEditor, { type KeyValuePair } from '@/components/shared/KeyValueEditor.vue'
import StringListEditor from '@/components/shared/StringListEditor.vue'
import {
  extractStorageClassFormValues,
  applyStorageClassFormToResource,
  type StorageClassFormValues
} from '@/utils/storageForms'
import InputText from 'primevue/inputtext'
import ToggleSwitch from 'primevue/toggleswitch'
import { computed, ref, toRaw, watch } from 'vue'

const props = defineProps<{
  rawData: Record<string, unknown> | null
}>()

const emit = defineEmits<{
  (e: 'update:rawData', value: Record<string, unknown>): void
  (e: 'update:isValid', value: boolean): void
}>()

const name = ref('')
const provisioner = ref('')
const reclaimPolicy = ref('Delete')
const volumeBindingMode = ref('Immediate')
const allowVolumeExpansion = ref(false)
const isDefaultClass = ref(false)
const mountOptions = ref<string[]>([])
const parameters = ref<KeyValuePair[]>([])
const labels = ref<KeyValuePair[]>([])
const annotations = ref<KeyValuePair[]>([])

let isEmitting = false

const isFormValid = computed(() => true)

watch(
  isFormValid,
  (val) => {
    emit('update:isValid', val)
  },
  { immediate: true }
)

const syncFromRawData = (data: Record<string, unknown> | null) => {
  if (!data || isEmitting) return
  const form = extractStorageClassFormValues(data)
  name.value = form.name
  provisioner.value = form.provisioner
  reclaimPolicy.value = form.reclaimPolicy
  volumeBindingMode.value = form.volumeBindingMode
  allowVolumeExpansion.value = form.allowVolumeExpansion
  isDefaultClass.value = form.isDefaultClass
  mountOptions.value = [...form.mountOptions]
  parameters.value = form.parameters.map((p) => ({ key: p.key, value: p.value }))
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
  const formVals: StorageClassFormValues = {
    name: name.value,
    provisioner: provisioner.value,
    reclaimPolicy: reclaimPolicy.value,
    volumeBindingMode: volumeBindingMode.value,
    allowVolumeExpansion: allowVolumeExpansion.value,
    isDefaultClass: isDefaultClass.value,
    mountOptions: mountOptions.value.map((m) => m.trim()).filter(Boolean),
    parameters: parameters.value.map((p) => ({ key: p.key, value: p.value })),
    labels: labels.value.map((l) => ({ key: l.key, value: l.value })),
    annotations: annotations.value.map((a) => ({ key: a.key, value: a.value }))
  }

  const updated = applyStorageClassFormToResource(rawCopy, formVals)
  emit('update:rawData', updated)

  setTimeout(() => {
    isEmitting = false
  }, 0)
}
</script>

<template>
  <div data-component="storage-class-edit-form" class="flex flex-col gap-8 pb-12">
    <!-- Section 1: Provisioner & Binding (Immutable) -->
    <div class="grid grid-cols-1 md:grid-cols-12 gap-6 max-w-7xl">
      <div class="md:col-span-4 flex flex-col gap-1">
        <span class="text-xs font-semibold tracking-wider text-muted-color uppercase">
          Identity & Provisioner
        </span>
        <p class="text-xs text-muted-color leading-relaxed">
          StorageClass name, provisioner plugin, and volume allocation settings. Immutable in
          Kubernetes.
        </p>
      </div>
      <div class="md:col-span-8 flex flex-col gap-3">
        <div class="flex flex-col gap-1.5">
          <label class="text-xs font-medium text-muted-color">StorageClass Name</label>
          <InputText disabled :model-value="name" fluid />
        </div>
        <div class="flex flex-col gap-1.5">
          <label class="text-xs font-medium text-muted-color">Provisioner Plugin</label>
          <InputText disabled :model-value="provisioner" fluid />
        </div>
        <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
          <div class="flex flex-col gap-1.5">
            <label class="text-xs font-medium text-muted-color">Reclaim Policy</label>
            <InputText disabled :model-value="reclaimPolicy" fluid />
          </div>
          <div class="flex flex-col gap-1.5">
            <label class="text-xs font-medium text-muted-color">Volume Binding Mode</label>
            <InputText disabled :model-value="volumeBindingMode" fluid />
          </div>
        </div>
      </div>
    </div>

    <!-- Section 2: Mutable Behaviors -->
    <div class="grid grid-cols-1 md:grid-cols-12 gap-6 max-w-7xl">
      <div class="md:col-span-4 flex flex-col gap-1">
        <span class="text-xs font-semibold tracking-wider text-muted-color uppercase">
          Class Options
        </span>
        <p class="text-xs text-muted-color leading-relaxed">
          Default class annotation, expansion support, and mount options.
        </p>
      </div>
      <div class="md:col-span-8 flex flex-col gap-6">
        <div class="flex flex-col gap-4 p-4 rounded-[var(--ui-radius-card)] bg-(--bg-hover)/30">
          <div class="flex items-center justify-between">
            <div class="flex flex-col">
              <span class="text-xs font-semibold text-primary">Default Storage Class</span>
              <span class="text-[11px] text-muted-color">
                Automatically provisions storage for PVCs without an explicit storageClassName.
              </span>
            </div>
            <ToggleSwitch v-model="isDefaultClass" @update:model-value="handleFieldChange" />
          </div>

          <div class="flex items-center justify-between">
            <div class="flex flex-col">
              <span class="text-xs font-semibold text-primary">Allow Volume Expansion</span>
              <span class="text-[11px] text-muted-color">
                Permits users to resize PVCs backed by this StorageClass.
              </span>
            </div>
            <ToggleSwitch v-model="allowVolumeExpansion" @update:model-value="handleFieldChange" />
          </div>
        </div>

        <div class="p-4 rounded-[var(--ui-radius-card)] bg-(--bg-hover)/30 flex flex-col gap-3">
          <StringListEditor
            v-model="mountOptions"
            title="Mount Options"
            placeholder="e.g. debug, noatime"
            add-label="Add Mount Option"
            @update:model-value="handleFieldChange"
          />
        </div>
      </div>
    </div>

    <!-- Section 3: Provisioner Parameters (Read-Only) -->
    <div v-if="parameters.length > 0" class="grid grid-cols-1 md:grid-cols-12 gap-6 max-w-7xl">
      <div class="md:col-span-4 flex flex-col gap-1">
        <span class="text-xs font-semibold tracking-wider text-muted-color uppercase">
          Parameters
        </span>
        <p class="text-xs text-muted-color leading-relaxed">
          Provisioner-specific parameters defined during StorageClass creation.
        </p>
      </div>
      <div class="md:col-span-8 flex flex-col gap-3">
        <div class="p-4 rounded-[var(--ui-radius-card)] bg-(--bg-hover)/30 flex flex-col gap-2">
          <div
            v-for="param in parameters"
            :key="param.key"
            class="flex items-center justify-between text-xs font-mono py-1 border-b border-(--border) last:border-0"
          >
            <span class="text-muted-color font-semibold">{{ param.key }}</span>
            <span class="text-primary">{{ param.value }}</span>
          </div>
        </div>
      </div>
    </div>

    <!-- Section 4: Metadata & Tags -->
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
