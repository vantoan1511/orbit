<script setup lang="ts">
import KeyValueEditor, { type KeyValuePair } from '@/components/shared/KeyValueEditor.vue'
import { useCreateResourceDialog } from '@/composables/useCreateResourceDialog'
import { KUBERNETES_RESOURCE_KIND } from '@/constants/kubernetes'
import { kubernetesService } from '@/services/kubernetesService'
import { useKubernetesStore } from '@/stores/kubernetesStore'
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
import { computed, inject, ref, type Ref } from 'vue'

const dialogRef = inject<
  | Ref<{
      data?: { initialNamespace?: string }
      close: () => void
    }>
  | undefined
>('dialogRef')

const k8sStore = useKubernetesStore()

const { namespace, namespaceOptions, isCreating, submittedName } = useCreateResourceDialog({
  onSuccess: () => void k8sStore.fetchPersistentVolumeClaims()
})

const name = ref('')
const storageRequest = ref('10Gi')
const accessModes = ref<string[]>(['ReadWriteOnce'])
const volumeMode = ref('Filesystem')
const storageClassName = ref('')
const labels = ref<KeyValuePair[]>([])

const accessModeOptions = ['ReadWriteOnce', 'ReadOnlyMany', 'ReadWriteMany']
const volumeModeOptions = ['Filesystem', 'Block']

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
  } catch (err) {
    console.error('Failed to create PersistentVolumeClaim:', err)
    isCreating.value = false
    submittedName.value = ''
  }
}
</script>

<template>
  <form
    data-component="create-pvc-dialog"
    class="flex flex-col gap-4 p-1"
    @submit.prevent="handleCreate"
  >
    <!-- Claim Name & Namespace -->
    <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
      <div class="flex flex-col gap-1.5">
        <label for="pvc-name" class="text-xs font-medium text-muted-color">Claim Name *</label>
        <InputText
          id="pvc-name"
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
        <label for="pvc-namespace" class="text-xs font-medium text-muted-color">Namespace *</label>
        <Select id="pvc-namespace" v-model="namespace" :options="namespaceOptions" fluid />
      </div>
    </div>

    <!-- Storage Request & StorageClass -->
    <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
      <div class="flex flex-col gap-1.5">
        <label for="pvc-storage-request" class="text-xs font-medium text-muted-color"
          >Storage Request *</label
        >
        <InputText
          id="pvc-storage-request"
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
        <label for="pvc-storage-class" class="text-xs font-medium text-muted-color"
          >Storage Class</label
        >
        <Select
          id="pvc-storage-class"
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
        <label for="pvc-access-modes" class="text-xs font-medium text-muted-color"
          >Access Modes *</label
        >
        <MultiSelect
          id="pvc-access-modes"
          v-model="accessModes"
          :options="accessModeOptions"
          placeholder="Select modes"
          :invalid="Boolean(validationResult.errors.accessModes)"
          fluid
        />
        <span v-if="validationResult.errors.accessModes" class="text-[11px] text-(--danger)">
          {{ validationResult.errors.accessModes }}
        </span>
      </div>

      <div class="flex flex-col gap-1.5">
        <label for="pvc-volume-mode" class="text-xs font-medium text-muted-color"
          >Volume Mode</label
        >
        <Select id="pvc-volume-mode" v-model="volumeMode" :options="volumeModeOptions" fluid />
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
        :disabled="isCreating"
        @click="handleCancel"
      />
      <Button
        type="submit"
        :loading="isCreating"
        :disabled="!isFormValid || isCreating"
        class="flex items-center gap-1.5"
      >
        <Plus class="w-3.5 h-3.5" />
        <span>{{ isCreating ? 'Creating...' : 'Create Claim' }}</span>
      </Button>
    </div>
  </form>
</template>
