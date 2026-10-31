<script setup lang="ts">
import KeyValueEditor, { type KeyValuePair } from '@/components/shared/KeyValueEditor.vue'
import StringListEditor from '@/components/shared/StringListEditor.vue'
import { useCreateResourceDialog } from '@/composables/useCreateResourceDialog'
import { KUBERNETES_RESOURCE_KIND } from '@/constants/kubernetes'
import { kubernetesService } from '@/services/kubernetesService'
import { useKubernetesStore } from '@/stores/kubernetesStore'
import {
  buildCreateStorageClassManifest,
  validateCreateStorageClass,
  type CreateStorageClassParams
} from '@/utils/storageForms'
import { Plus } from '@lucide/vue'
import Button from 'primevue/button'
import InputText from 'primevue/inputtext'
import Select from 'primevue/select'
import ToggleSwitch from 'primevue/toggleswitch'
import { computed, inject, ref, type Ref } from 'vue'

const dialogRef = inject<
  | Ref<{
      close: () => void
    }>
  | undefined
>('dialogRef')

const k8sStore = useKubernetesStore()

const { isCreating, submittedName } = useCreateResourceDialog({
  onSuccess: () => void k8sStore.fetchStorageClasses()
})

const name = ref('')
const provisioner = ref('rancher.io/local-path')
const reclaimPolicy = ref('Delete')
const volumeBindingMode = ref('Immediate')
const allowVolumeExpansion = ref(true)
const isDefaultClass = ref(false)
const mountOptions = ref<string[]>([])
const parameters = ref<KeyValuePair[]>([])
const labels = ref<KeyValuePair[]>([])

const reclaimPolicyOptions = ['Delete', 'Retain']
const volumeBindingModeOptions = ['Immediate', 'WaitForFirstConsumer']

const validationResult = computed(() => {
  const params: CreateStorageClassParams = {
    name: name.value,
    provisioner: provisioner.value,
    reclaimPolicy: reclaimPolicy.value,
    volumeBindingMode: volumeBindingMode.value,
    allowVolumeExpansion: allowVolumeExpansion.value,
    isDefaultClass: isDefaultClass.value,
    parameters: parameters.value,
    mountOptions: mountOptions.value.map((m) => m.trim()).filter(Boolean),
    labels: labels.value
  }
  return validateCreateStorageClass(params)
})

const isFormValid = computed(() => validationResult.value.valid)

const handleCancel = () => {
  dialogRef?.value?.close()
}

const handleCreate = async () => {
  if (!isFormValid.value || isCreating.value) return

  const params: CreateStorageClassParams = {
    name: name.value,
    provisioner: provisioner.value,
    reclaimPolicy: reclaimPolicy.value,
    volumeBindingMode: volumeBindingMode.value,
    allowVolumeExpansion: allowVolumeExpansion.value,
    isDefaultClass: isDefaultClass.value,
    parameters: parameters.value,
    mountOptions: mountOptions.value.map((m) => m.trim()).filter(Boolean),
    labels: labels.value
  }

  const manifest = buildCreateStorageClassManifest(params)
  isCreating.value = true
  submittedName.value = name.value.trim()

  try {
    await kubernetesService.createResource({
      namespace: '',
      kind: KUBERNETES_RESOURCE_KIND.StorageClass,
      name: submittedName.value,
      data: manifest
    })
  } catch (err) {
    console.error('Failed to create StorageClass:', err)
    isCreating.value = false
    submittedName.value = ''
  }
}
</script>

<template>
  <form
    data-component="create-storage-class-dialog"
    class="flex flex-col gap-4 p-1"
    @submit.prevent="handleCreate"
  >
    <!-- StorageClass Name & Provisioner -->
    <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
      <div class="flex flex-col gap-1.5">
        <label for="sc-name" class="text-xs font-medium text-muted-color">Class Name *</label>
        <InputText
          id="sc-name"
          v-model="name"
          placeholder="e.g. fast-ssd"
          :invalid="Boolean(name && validationResult.errors.name)"
          fluid
        />
        <span v-if="name && validationResult.errors.name" class="text-[11px] text-(--danger)">
          {{ validationResult.errors.name }}
        </span>
      </div>

      <div class="flex flex-col gap-1.5">
        <label for="sc-provisioner" class="text-xs font-medium text-muted-color"
          >Provisioner *</label
        >
        <InputText
          id="sc-provisioner"
          v-model="provisioner"
          placeholder="e.g. rancher.io/local-path"
          :invalid="Boolean(provisioner && validationResult.errors.provisioner)"
          fluid
        />
        <span
          v-if="provisioner && validationResult.errors.provisioner"
          class="text-[11px] text-(--danger)"
        >
          {{ validationResult.errors.provisioner }}
        </span>
      </div>
    </div>

    <!-- Reclaim Policy & Volume Binding Mode -->
    <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
      <div class="flex flex-col gap-1.5">
        <label for="sc-reclaim-policy" class="text-xs font-medium text-muted-color"
          >Reclaim Policy</label
        >
        <Select
          id="sc-reclaim-policy"
          v-model="reclaimPolicy"
          :options="reclaimPolicyOptions"
          fluid
        />
      </div>

      <div class="flex flex-col gap-1.5">
        <label for="sc-volume-binding-mode" class="text-xs font-medium text-muted-color"
          >Volume Binding Mode</label
        >
        <Select
          id="sc-volume-binding-mode"
          v-model="volumeBindingMode"
          :options="volumeBindingModeOptions"
          fluid
        />
      </div>
    </div>

    <!-- Toggles: Default Class & Volume Expansion -->
    <div class="flex flex-col gap-3 p-3 rounded-[var(--ui-radius-card)] bg-(--bg-hover)/30">
      <div class="flex items-center justify-between">
        <div class="flex flex-col">
          <span class="text-xs font-semibold text-primary">Default Storage Class</span>
          <span class="text-[11px] text-muted-color">
            Annotates class as cluster default for unbound claims.
          </span>
        </div>
        <ToggleSwitch v-model="isDefaultClass" />
      </div>

      <div class="flex items-center justify-between">
        <div class="flex flex-col">
          <span class="text-xs font-semibold text-primary">Allow Volume Expansion</span>
          <span class="text-[11px] text-muted-color">
            Allows resizing persistent volume claims dynamically.
          </span>
        </div>
        <ToggleSwitch v-model="allowVolumeExpansion" />
      </div>
    </div>

    <!-- Parameters (Key-Value) -->
    <div class="p-3 rounded-[var(--ui-radius-card)] bg-(--bg-hover)/30 flex flex-col gap-2">
      <KeyValueEditor v-model="parameters" title="Parameters" add-label="Add Parameter" />
    </div>

    <!-- Mount Options -->
    <div class="p-3 rounded-[var(--ui-radius-card)] bg-(--bg-hover)/30 flex flex-col gap-2">
      <StringListEditor
        v-model="mountOptions"
        title="Mount Options"
        placeholder="e.g. noatime"
        add-label="Add Mount Option"
      />
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
        <span>{{ isCreating ? 'Creating...' : 'Create Class' }}</span>
      </Button>
    </div>
  </form>
</template>
