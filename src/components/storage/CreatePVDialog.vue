<script setup lang="ts">
import KeyValueEditor, { type KeyValuePair } from '@/components/shared/KeyValueEditor.vue'
import { useCreateResourceDialog } from '@/composables/useCreateResourceDialog'
import { KUBERNETES_RESOURCE_KIND } from '@/constants/kubernetes'
import { kubernetesService } from '@/services/kubernetesService'
import { useKubernetesStore } from '@/stores/kubernetesStore'
import { buildCreatePVManifest, validateCreatePV, type CreatePVParams } from '@/utils/storageForms'
import { Plus } from '@lucide/vue'
import Button from 'primevue/button'
import InputText from 'primevue/inputtext'
import MultiSelect from 'primevue/multiselect'
import Select from 'primevue/select'
import { computed, inject, ref, type Ref } from 'vue'

const dialogRef = inject<
  | Ref<{
      close: () => void
    }>
  | undefined
>('dialogRef')

const k8sStore = useKubernetesStore()

const { isCreating, submittedName } = useCreateResourceDialog({
  onSuccess: () => void k8sStore.fetchPersistentVolumes()
})

const name = ref('')
const capacity = ref('10Gi')
const accessModes = ref<string[]>(['ReadWriteOnce'])
const reclaimPolicy = ref('Retain')
const volumeMode = ref('Filesystem')
const storageClassName = ref('')
const sourceType = ref<'hostPath' | 'nfs' | 'csi'>('hostPath')
const hostPath = ref('/mnt/data')
const nfsServer = ref('')
const nfsPath = ref('')
const csiDriver = ref('')
const csiVolumeHandle = ref('')
const labels = ref<KeyValuePair[]>([])

const accessModeOptions = ['ReadWriteOnce', 'ReadOnlyMany', 'ReadWriteMany']
const reclaimPolicyOptions = ['Retain', 'Delete']
const volumeModeOptions = ['Filesystem', 'Block']
const sourceTypeOptions = [
  { label: 'HostPath (Local Node)', value: 'hostPath' },
  { label: 'NFS (Network File System)', value: 'nfs' },
  { label: 'CSI (Container Storage Interface)', value: 'csi' }
]

const storageClassOptions = computed(() => {
  return k8sStore.storageClasses.map((sc) => sc.name)
})

const validationResult = computed(() => {
  const params: CreatePVParams = {
    name: name.value,
    capacity: capacity.value,
    accessModes: accessModes.value,
    reclaimPolicy: reclaimPolicy.value,
    volumeMode: volumeMode.value,
    storageClassName: storageClassName.value,
    sourceType: sourceType.value,
    hostPath: hostPath.value,
    nfsServer: nfsServer.value,
    nfsPath: nfsPath.value,
    csiDriver: csiDriver.value,
    csiVolumeHandle: csiVolumeHandle.value,
    labels: labels.value
  }
  return validateCreatePV(params)
})

const isFormValid = computed(() => validationResult.value.valid)

const handleCancel = () => {
  dialogRef?.value?.close()
}

const handleCreate = async () => {
  if (!isFormValid.value || isCreating.value) return

  const params: CreatePVParams = {
    name: name.value,
    capacity: capacity.value,
    accessModes: accessModes.value,
    reclaimPolicy: reclaimPolicy.value,
    volumeMode: volumeMode.value,
    storageClassName: storageClassName.value,
    sourceType: sourceType.value,
    hostPath: hostPath.value,
    nfsServer: nfsServer.value,
    nfsPath: nfsPath.value,
    csiDriver: csiDriver.value,
    csiVolumeHandle: csiVolumeHandle.value,
    labels: labels.value
  }

  const manifest = buildCreatePVManifest(params)
  isCreating.value = true
  submittedName.value = name.value.trim()

  try {
    await kubernetesService.createResource({
      namespace: '',
      kind: KUBERNETES_RESOURCE_KIND.PersistentVolume,
      name: submittedName.value,
      data: manifest
    })
  } catch (err) {
    console.error('Failed to create PersistentVolume:', err)
    isCreating.value = false
    submittedName.value = ''
  }
}
</script>

<template>
  <form
    data-component="create-pv-dialog"
    class="flex flex-col gap-4 p-1"
    @submit.prevent="handleCreate"
  >
    <!-- Volume Name & Capacity -->
    <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
      <div class="flex flex-col gap-1.5">
        <label for="pv-name" class="text-xs font-medium text-muted-color">Volume Name *</label>
        <InputText
          id="pv-name"
          v-model="name"
          placeholder="e.g. pv-data-01"
          :invalid="Boolean(name && validationResult.errors.name)"
          fluid
        />
        <span v-if="name && validationResult.errors.name" class="text-[11px] text-(--danger)">
          {{ validationResult.errors.name }}
        </span>
      </div>

      <div class="flex flex-col gap-1.5">
        <label for="pv-capacity" class="text-xs font-medium text-muted-color">Capacity *</label>
        <InputText
          id="pv-capacity"
          v-model="capacity"
          placeholder="e.g. 10Gi, 500Mi"
          :invalid="Boolean(capacity && validationResult.errors.capacity)"
          fluid
        />
        <span
          v-if="capacity && validationResult.errors.capacity"
          class="text-[11px] text-(--danger)"
        >
          {{ validationResult.errors.capacity }}
        </span>
      </div>
    </div>

    <!-- Access Modes & Volume Mode -->
    <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
      <div class="flex flex-col gap-1.5">
        <label for="pv-access-modes" class="text-xs font-medium text-muted-color"
          >Access Modes *</label
        >
        <MultiSelect
          id="pv-access-modes"
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
        <label for="pv-volume-mode" class="text-xs font-medium text-muted-color">Volume Mode</label>
        <Select id="pv-volume-mode" v-model="volumeMode" :options="volumeModeOptions" fluid />
      </div>
    </div>

    <!-- Reclaim Policy & StorageClass -->
    <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
      <div class="flex flex-col gap-1.5">
        <label for="pv-reclaim-policy" class="text-xs font-medium text-muted-color"
          >Reclaim Policy</label
        >
        <Select
          id="pv-reclaim-policy"
          v-model="reclaimPolicy"
          :options="reclaimPolicyOptions"
          fluid
        />
      </div>

      <div class="flex flex-col gap-1.5">
        <label for="pv-storage-class" class="text-xs font-medium text-muted-color"
          >Storage Class</label
        >
        <Select
          id="pv-storage-class"
          v-model="storageClassName"
          :options="storageClassOptions"
          editable
          placeholder="Select or enter SC"
          fluid
        />
      </div>
    </div>

    <!-- Volume Source Type & Conditional Inputs -->
    <div class="flex flex-col gap-1.5">
      <label for="pv-source-type" class="text-xs font-medium text-muted-color"
        >Volume Source Plugin *</label
      >
      <Select
        id="pv-source-type"
        v-model="sourceType"
        :options="sourceTypeOptions"
        option-label="label"
        option-value="value"
        fluid
      />
    </div>

    <!-- HostPath Details -->
    <div
      v-if="sourceType === 'hostPath'"
      class="p-3 rounded-[var(--ui-radius-card)] bg-(--bg-hover)/30 flex flex-col gap-2"
    >
      <label for="pv-host-path" class="text-xs font-medium text-muted-color">Host Path *</label>
      <InputText
        id="pv-host-path"
        v-model="hostPath"
        placeholder="/mnt/data"
        :invalid="Boolean(hostPath && validationResult.errors.hostPath)"
        fluid
      />
      <span v-if="validationResult.errors.hostPath" class="text-[11px] text-(--danger)">
        {{ validationResult.errors.hostPath }}
      </span>
    </div>

    <!-- NFS Details -->
    <div
      v-if="sourceType === 'nfs'"
      class="p-3 rounded-[var(--ui-radius-card)] bg-(--bg-hover)/30 flex flex-col gap-3"
    >
      <div class="flex flex-col gap-1.5">
        <label for="pv-nfs-server" class="text-xs font-medium text-muted-color">NFS Server *</label>
        <InputText
          id="pv-nfs-server"
          v-model="nfsServer"
          placeholder="nfs.example.com or 192.168.1.100"
          :invalid="Boolean(validationResult.errors.nfsServer)"
          fluid
        />
        <span v-if="validationResult.errors.nfsServer" class="text-[11px] text-(--danger)">
          {{ validationResult.errors.nfsServer }}
        </span>
      </div>
      <div class="flex flex-col gap-1.5">
        <label for="pv-nfs-path" class="text-xs font-medium text-muted-color">NFS Path *</label>
        <InputText
          id="pv-nfs-path"
          v-model="nfsPath"
          placeholder="/exports/data"
          :invalid="Boolean(validationResult.errors.nfsPath)"
          fluid
        />
        <span v-if="validationResult.errors.nfsPath" class="text-[11px] text-(--danger)">
          {{ validationResult.errors.nfsPath }}
        </span>
      </div>
    </div>

    <!-- CSI Details -->
    <div
      v-if="sourceType === 'csi'"
      class="p-3 rounded-[var(--ui-radius-card)] bg-(--bg-hover)/30 flex flex-col gap-3"
    >
      <div class="flex flex-col gap-1.5">
        <label for="pv-csi-driver" class="text-xs font-medium text-muted-color">CSI Driver *</label>
        <InputText
          id="pv-csi-driver"
          v-model="csiDriver"
          placeholder="hostpath.csi.k8s.io"
          :invalid="Boolean(validationResult.errors.csiDriver)"
          fluid
        />
        <span v-if="validationResult.errors.csiDriver" class="text-[11px] text-(--danger)">
          {{ validationResult.errors.csiDriver }}
        </span>
      </div>
      <div class="flex flex-col gap-1.5">
        <label for="pv-csi-volume-handle" class="text-xs font-medium text-muted-color"
          >Volume Handle *</label
        >
        <InputText
          id="pv-csi-volume-handle"
          v-model="csiVolumeHandle"
          placeholder="csi-vol-12345"
          :invalid="Boolean(validationResult.errors.csiVolumeHandle)"
          fluid
        />
        <span v-if="validationResult.errors.csiVolumeHandle" class="text-[11px] text-(--danger)">
          {{ validationResult.errors.csiVolumeHandle }}
        </span>
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
        <span>{{ isCreating ? 'Creating...' : 'Create Volume' }}</span>
      </Button>
    </div>
  </form>
</template>
