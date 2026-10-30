<script setup lang="ts">
import KeyValueEditor, { type KeyValuePair } from '@/components/shared/KeyValueEditor.vue'
import { KUBERNETES_RESOURCE_KIND } from '@/constants/kubernetes'
import { kubernetesService } from '@/services/kubernetesService'
import { events } from '@/services/nativeService'
import { useKubernetesStore } from '@/stores/kubernetesStore'
import { OrbitEvents } from '@/types/events'
import { buildCreatePVManifest, validateCreatePV, type CreatePVParams } from '@/utils/storageForms'
import { Plus } from '@lucide/vue'
import Button from 'primevue/button'
import InputText from 'primevue/inputtext'
import MultiSelect from 'primevue/multiselect'
import Select from 'primevue/select'
import { computed, inject, onMounted, onUnmounted, ref, type Ref } from 'vue'

const dialogRef = inject<
  | Ref<{
      close: () => void
    }>
  | undefined
>('dialogRef')

const k8sStore = useKubernetesStore()

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

const isCreating = ref(false)
const submittedName = ref('')

const accessModeOptions = ['ReadWriteOnce', 'ReadOnlyMany', 'ReadWriteMany']
const reclaimPolicyOptions = ['Retain', 'Delete', 'Recycle']
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

const handleCommandSucceeded = (payload: { message: string }) => {
  if (isCreating.value && submittedName.value && payload.message.includes(submittedName.value)) {
    isCreating.value = false
    submittedName.value = ''
    void k8sStore.fetchPersistentVolumes()
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
    await k8sStore.fetchPersistentVolumes()
    dialogRef?.value?.close()
  } catch (err) {
    console.error('Failed to create PersistentVolume:', err)
    isCreating.value = false
    submittedName.value = ''
  }
}
</script>

<template>
  <div data-component="create-pv-dialog" class="flex flex-col gap-4 p-1">
    <!-- Volume Name & Capacity -->
    <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
      <div class="flex flex-col gap-1.5">
        <label class="text-xs font-medium text-muted-color">Volume Name *</label>
        <InputText
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
        <label class="text-xs font-medium text-muted-color">Capacity *</label>
        <InputText
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

    <!-- Reclaim Policy & StorageClass -->
    <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
      <div class="flex flex-col gap-1.5">
        <label class="text-xs font-medium text-muted-color">Reclaim Policy</label>
        <Select v-model="reclaimPolicy" :options="reclaimPolicyOptions" fluid />
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

    <!-- Volume Source Type & Conditional Inputs -->
    <div class="flex flex-col gap-1.5">
      <label class="text-xs font-medium text-muted-color">Volume Source Plugin *</label>
      <Select
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
      <label class="text-xs font-medium text-muted-color">Host Path *</label>
      <InputText
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
        <label class="text-xs font-medium text-muted-color">NFS Server *</label>
        <InputText v-model="nfsServer" placeholder="nfs.example.com or 192.168.1.100" fluid />
      </div>
      <div class="flex flex-col gap-1.5">
        <label class="text-xs font-medium text-muted-color">NFS Path *</label>
        <InputText v-model="nfsPath" placeholder="/exports/data" fluid />
      </div>
    </div>

    <!-- CSI Details -->
    <div
      v-if="sourceType === 'csi'"
      class="p-3 rounded-[var(--ui-radius-card)] bg-(--bg-hover)/30 flex flex-col gap-3"
    >
      <div class="flex flex-col gap-1.5">
        <label class="text-xs font-medium text-muted-color">CSI Driver *</label>
        <InputText v-model="csiDriver" placeholder="hostpath.csi.k8s.io" fluid />
      </div>
      <div class="flex flex-col gap-1.5">
        <label class="text-xs font-medium text-muted-color">Volume Handle *</label>
        <InputText v-model="csiVolumeHandle" placeholder="csi-vol-12345" fluid />
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
        <span>{{ isCreating ? 'Creating...' : 'Create Volume' }}</span>
      </Button>
    </div>
  </div>
</template>
