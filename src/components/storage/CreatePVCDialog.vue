<script setup lang="ts">
import KeyValueEditor, { type KeyValuePair } from '@/components/shared/KeyValueEditor.vue'
import { KUBERNETES_RESOURCE_KIND } from '@/constants/kubernetes'
import { kubernetesService } from '@/services/kubernetesService'
import { events } from '@/services/nativeService'
import { useKubernetesStore } from '@/stores/kubernetesStore'
import { OrbitEvents } from '@/types/events'
import {
  buildCreatePVCManifest,
  validateCreatePVC,
  type CreatePVCParams
} from '@/utils/storageForms'
import { Plus } from '@lucide/vue'
import Button from 'primevue/button'
import InputText from 'primevue/inputtext'
import MultiSelect from 'primevue/multiselect'
import Select from 'primevue/select'
import { computed, inject, onMounted, onUnmounted, ref, type Ref } from 'vue'

const dialogRef = inject<
  | Ref<{
      data?: { initialNamespace?: string }
      close: () => void
    }>
  | undefined
>('dialogRef')

const k8sStore = useKubernetesStore()

const name = ref('')
const namespace = ref('default')
const storageRequest = ref('10Gi')
const accessModes = ref<string[]>(['ReadWriteOnce'])
const volumeMode = ref('Filesystem')
const storageClassName = ref('')
const labels = ref<KeyValuePair[]>([])

const isCreating = ref(false)
const submittedName = ref('')

const accessModeOptions = ['ReadWriteOnce', 'ReadOnlyMany', 'ReadWriteMany']
const volumeModeOptions = ['Filesystem', 'Block']

const namespaceOptions = computed(() => {
  const list = k8sStore.namespaceList.map((ns) => ns.name)
  return list.length > 0 ? list : ['default']
})

const storageClassOptions = computed(() => {
  return k8sStore.storageClasses.map((sc) => sc.name)
})

const validationResult = computed(() => {
  const params: CreatePVCParams = {
    name: name.value,
    namespace: namespace.value,
    storageRequest: storageRequest.value,
    accessModes: accessModes.value,
    volumeMode: volumeMode.value,
    storageClassName: storageClassName.value,
    labels: labels.value
  }
  return validateCreatePVC(params)
})

const isFormValid = computed(() => validationResult.value.valid)

const handleCommandSucceeded = (payload: { message: string }) => {
  if (isCreating.value && submittedName.value && payload.message.includes(submittedName.value)) {
    isCreating.value = false
    submittedName.value = ''
    void k8sStore.fetchPersistentVolumeClaims()
    dialogRef?.value?.close()
  }
}

const handleErrorOccurred = () => {
  if (isCreating.value) {
    isCreating.value = false
    submittedName.value = ''
  }
}

onMounted(() => {
  events.on(OrbitEvents.CommandSucceeded, handleCommandSucceeded)
  events.on(OrbitEvents.ErrorOccurred, handleErrorOccurred)

  if (dialogRef?.value?.data?.initialNamespace) {
    namespace.value = dialogRef.value.data.initialNamespace
  } else if (namespaceOptions.value.length > 0) {
    namespace.value = namespaceOptions.value.includes('default')
      ? 'default'
      : (namespaceOptions.value[0] ?? 'default')
  }
})

onUnmounted(() => {
  events.off(OrbitEvents.CommandSucceeded, handleCommandSucceeded)
  events.off(OrbitEvents.ErrorOccurred, handleErrorOccurred)
})

const handleCancel = () => {
  dialogRef?.value?.close()
}

const handleCreate = async () => {
  if (!isFormValid.value || isCreating.value) return

  const params: CreatePVCParams = {
    name: name.value,
    namespace: namespace.value,
    storageRequest: storageRequest.value,
    accessModes: accessModes.value,
    volumeMode: volumeMode.value,
    storageClassName: storageClassName.value,
    labels: labels.value
  }

  const manifest = buildCreatePVCManifest(params)
  isCreating.value = true
  submittedName.value = name.value.trim()

  try {
    await kubernetesService.createResource({
      namespace: namespace.value.trim(),
      kind: KUBERNETES_RESOURCE_KIND.PersistentVolumeClaim,
      name: submittedName.value,
      data: manifest
    })
    await k8sStore.fetchPersistentVolumeClaims()
    dialogRef?.value?.close()
  } catch (err) {
    console.error('Failed to create PersistentVolumeClaim:', err)
    isCreating.value = false
    submittedName.value = ''
  }
}
</script>

<template>
  <div data-component="create-pvc-dialog" class="flex flex-col gap-4 p-1">
    <!-- Claim Name & Namespace -->
    <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
      <div class="flex flex-col gap-1.5">
        <label class="text-xs font-medium text-muted-color">Claim Name *</label>
        <InputText
          v-model="name"
          placeholder="e.g. pvc-app-data"
          :invalid="Boolean(name && validationResult.errors.name)"
          fluid
        />
        <span v-if="name && validationResult.errors.name" class="text-[11px] text-(--danger)">
          {{ validationResult.errors.name }}
        </span>
      </div>

      <div class="flex flex-col gap-1.5">
        <label class="text-xs font-medium text-muted-color">Namespace *</label>
        <Select v-model="namespace" :options="namespaceOptions" fluid />
      </div>
    </div>

    <!-- Storage Request & StorageClass -->
    <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
      <div class="flex flex-col gap-1.5">
        <label class="text-xs font-medium text-muted-color">Storage Request *</label>
        <InputText
          v-model="storageRequest"
          placeholder="e.g. 10Gi, 500Mi"
          :invalid="Boolean(storageRequest && validationResult.errors.storageRequest)"
          fluid
        />
        <span
          v-if="storageRequest && validationResult.errors.storageRequest"
          class="text-[11px] text-(--danger)"
        >
          {{ validationResult.errors.storageRequest }}
        </span>
      </div>

      <div class="flex flex-col gap-1.5">
        <label class="text-xs font-medium text-muted-color">Storage Class</label>
        <Select
          v-model="storageClassName"
          :options="storageClassOptions"
          editable
          placeholder="Select or enter SC"
          fluid
        />
      </div>
    </div>

    <!-- Access Modes & Volume Mode -->
    <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
      <div class="flex flex-col gap-1.5">
        <label class="text-xs font-medium text-muted-color">Access Modes *</label>
        <MultiSelect
          v-model="accessModes"
          :options="accessModeOptions"
          placeholder="Select modes"
          fluid
        />
      </div>

      <div class="flex flex-col gap-1.5">
        <label class="text-xs font-medium text-muted-color">Volume Mode</label>
        <Select v-model="volumeMode" :options="volumeModeOptions" fluid />
      </div>
    </div>

    <!-- Labels -->
    <div class="p-3 rounded-[var(--ui-radius-card)] bg-(--bg-hover)/30 flex flex-col gap-2">
      <KeyValueEditor v-model="labels" title="Labels" add-label="Add Label" />
    </div>

    <!-- Modal Actions -->
    <div class="flex items-center justify-end gap-2 pt-2">
      <Button
        label="Cancel"
        severity="secondary"
        variant="text"
        @click="handleCancel"
      />
      <Button
        :loading="isCreating"
        :disabled="!isFormValid || isCreating"
        class="flex items-center gap-1.5"
        @click="handleCreate"
      >
        <Plus class="w-3.5 h-3.5" />
        <span>{{ isCreating ? 'Creating...' : 'Create Claim' }}</span>
      </Button>
    </div>
  </div>
</template>
