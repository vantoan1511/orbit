<script setup lang="ts">
import { ref } from 'vue'
import Button from 'primevue/button'
import InputNumber from 'primevue/inputnumber'
import InputText from 'primevue/inputtext'
import Popover from 'primevue/popover'
import { useToast } from 'primevue/usetoast'
import type { ContainerImageInfo } from '@/types/kubernetes'
import { kubernetesService } from '@/services/kubernetesService'
import { resolveEditableContainers } from '@/utils/workloadMetrics'

const props = defineProps<{
  kind: string
}>()

const toast = useToast()

interface ReplicasTarget {
  name: string
  namespace: string
  replicas: {
    desired: number
    current?: number
  }
}

interface ImagesTarget {
  name: string
  namespace: string
  containers?: ContainerImageInfo[]
  images?: string[]
}

// Inline Replicas Edit State
const replicasPopover = ref<InstanceType<typeof Popover> | null>(null)
const editingReplicasWorkload = ref<ReplicasTarget | null>(null)
const editReplicasValue = ref<number>(1)
const isSavingReplicas = ref<boolean>(false)

const openReplicasEdit = (event: Event, workload: ReplicasTarget) => {
  editingReplicasWorkload.value = workload
  editReplicasValue.value = workload.replicas.desired
  replicasPopover.value?.toggle(event)
}

const saveReplicas = async () => {
  const target = editingReplicasWorkload.value
  if (!target) return
  isSavingReplicas.value = true
  try {
    await kubernetesService.scaleResource({
      namespace: target.namespace,
      kind: props.kind,
      name: target.name,
      replicas: editReplicasValue.value
    })
    replicasPopover.value?.hide()
  } catch (e) {
    console.error(`Error scaling ${props.kind}`, e)
    toast.add({
      severity: 'error',
      summary: 'Error Scaling Workload',
      detail: e instanceof Error ? e.message : `Failed to scale ${props.kind}`,
      life: 5000
    })
  } finally {
    isSavingReplicas.value = false
  }
}

// Inline Images Edit State
const imagePopover = ref<InstanceType<typeof Popover> | null>(null)
const editingImagesWorkload = ref<ImagesTarget | null>(null)
const editContainers = ref<ContainerImageInfo[]>([])
const isSavingImages = ref<boolean>(false)

const openImagesEdit = (event: Event, workload: ImagesTarget) => {
  editingImagesWorkload.value = workload
  editContainers.value = resolveEditableContainers(workload)
  imagePopover.value?.toggle(event)
}

const saveImages = async () => {
  const target = editingImagesWorkload.value
  if (!target) return
  isSavingImages.value = true
  try {
    await kubernetesService.updateResourceImages({
      namespace: target.namespace,
      kind: props.kind,
      name: target.name,
      containers: editContainers.value
    })
    imagePopover.value?.hide()
  } catch (e) {
    console.error(`Error updating ${props.kind} images`, e)
    toast.add({
      severity: 'error',
      summary: 'Error Updating Images',
      detail: e instanceof Error ? e.message : `Failed to update ${props.kind} images`,
      life: 5000
    })
  } finally {
    isSavingImages.value = false
  }
}

defineExpose({
  openReplicasEdit,
  openImagesEdit
})
</script>

<template>
  <!-- Inline Replicas Edit Popover -->
  <Popover ref="replicasPopover" data-component="workload-inline-edit-popovers">
    <div class="flex flex-col gap-3 p-1 min-w-56" @click.stop>
      <div class="flex flex-col gap-0.5">
        <span class="text-xs font-semibold tracking-wider text-muted-color uppercase">
          Scale Replicas
        </span>
        <span class="text-xs text-muted-color font-mono truncate">
          {{ editingReplicasWorkload?.name }}
        </span>
      </div>
      <div class="flex items-center gap-2">
        <InputNumber
          v-model="editReplicasValue"
          :min="0"
          :max="1000"
          showButtons
          buttonLayout="horizontal"
          size="small"
          class="w-full"
        />
      </div>
      <div class="flex items-center justify-end gap-2 pt-2 border-t border-(--border)">
        <Button
          label="Cancel"
          severity="secondary"
          variant="text"
          size="small"
          @click="replicasPopover?.hide()"
        />
        <Button label="Save" size="small" :loading="isSavingReplicas" @click="saveReplicas" />
      </div>
    </div>
  </Popover>

  <!-- Inline Images Edit Popover -->
  <Popover ref="imagePopover" data-component="workload-inline-edit-popovers">
    <div class="flex flex-col gap-3 p-1 min-w-80" @click.stop>
      <div class="flex flex-col gap-0.5">
        <span class="text-xs font-semibold tracking-wider text-muted-color uppercase">
          Update Images
        </span>
        <span class="text-xs text-muted-color font-mono truncate">
          {{ editingImagesWorkload?.name }}
        </span>
      </div>
      <div class="flex flex-col gap-2 max-h-60 overflow-y-auto">
        <div
          v-for="c in editContainers"
          :key="c.name"
          class="flex flex-col gap-1 p-2 bg-(--bg-hover)/30 rounded-[var(--ui-radius-control)] border border-(--border)"
        >
          <span class="text-xs font-semibold font-mono text-primary">{{ c.name }}</span>
          <InputText v-model="c.image" size="small" class="w-full font-mono text-xs" />
        </div>
      </div>
      <div class="flex items-center justify-end gap-2 pt-2 border-t border-(--border)">
        <Button
          label="Cancel"
          severity="secondary"
          variant="text"
          size="small"
          @click="imagePopover?.hide()"
        />
        <Button
          label="Save"
          size="small"
          :loading="isSavingImages"
          :disabled="editContainers.length === 0"
          @click="saveImages"
        />
      </div>
    </div>
  </Popover>
</template>
