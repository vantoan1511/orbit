<script setup lang="ts">
import GenericResourceTable from '@/components/shared/GenericResourceTable.vue'
import { KUBERNETES_RESOURCE_KIND, KUBERNETES_WORKLOAD_STATUS } from '@/constants/kubernetes'
import { kubernetesService } from '@/services/kubernetesService'
import { useKubernetesStore } from '@/stores/kubernetesStore'
import type { ContainerImageInfo, DeploymentInfo } from '@/types/kubernetes'
import type { DeploymentCondition } from 'kubernetes-types/apps/v1'
import { Pencil } from '@lucide/vue'
import Button from 'primevue/button'
import Column from 'primevue/column'
import InputNumber from 'primevue/inputnumber'
import InputText from 'primevue/inputtext'
import Popover from 'primevue/popover'
import { computed, ref } from 'vue'
import WorkloadDetailsDrawer from './WorkloadDetailsDrawer.vue'

const k8sStore = useKubernetesStore()
const loading = ref(false)

const getRestarts = (deployment: DeploymentInfo) => {
  const deploymentPods = k8sStore.pods.filter((p) => {
    return (
      p.namespace === deployment.namespace &&
      p.controlledBy?.startsWith(`ReplicaSet/${deployment.name}-`)
    )
  })
  return deploymentPods.reduce((sum, pod) => sum + (pod.restarts ?? 0), 0)
}

const deployments = computed(() => {
  return k8sStore.deployments.map((d) => ({
    ...d,
    restarts: getRestarts(d)
  }))
})

const columns = [
  { field: 'namespace', header: 'Namespace', visible: true },
  { field: 'status', header: 'Status', visible: true },
  { field: 'conditions', header: 'Conditions', visible: true },
  { field: 'replicas', header: 'Replicas', visible: true },
  { field: 'available', header: 'Available', visible: true },
  { field: 'upToDate', header: 'Up-To-Date', visible: true },
  { field: 'restarts', header: 'Restarts', visible: true },
  { field: 'age', header: 'Age', visible: true },
  { field: 'images', header: 'Images', visible: true }
]

const statuses = [
  'All Statuses',
  KUBERNETES_WORKLOAD_STATUS.Running,
  KUBERNETES_WORKLOAD_STATUS.Progressing,
  KUBERNETES_WORKLOAD_STATUS.Failed
]

const CONDITION_ORDER: Record<string, number> = {
  Available: 1,
  Progressing: 2,
  ReplicaFailure: 3
}

const getOrderedConditions = (conditions?: DeploymentCondition[]) => {
  if (!conditions || conditions.length === 0) return []
  return [...conditions]
    .filter((c) => c.status === 'True')
    .sort((a, b) => {
      const orderA = CONDITION_ORDER[a.type] ?? 99
      const orderB = CONDITION_ORDER[b.type] ?? 99
      if (orderA !== orderB) return orderA - orderB
      return a.type.localeCompare(b.type)
    })
}

const getConditionSeverity = (type: string) => {
  switch (type) {
    case 'ReplicaFailure':
      return 'danger'
    case 'Available':
      return 'success'
    case 'Progressing':
      return 'info'
    default:
      return 'secondary'
  }
}

const fetchDeployments = async () => {
  loading.value = true
  try {
    await kubernetesService.getDeployments()
  } catch (e) {
    console.error('Error fetching deployments', e)
  } finally {
    loading.value = false
  }
}

// Inline Replicas Edit State
const replicasPopover = ref<InstanceType<typeof Popover> | null>(null)
const editingDeployment = ref<DeploymentInfo | null>(null)
const editReplicasValue = ref<number>(1)
const isSavingReplicas = ref<boolean>(false)

const openReplicasEdit = (event: Event, deployment: DeploymentInfo) => {
  editingDeployment.value = deployment
  editReplicasValue.value = deployment.replicas.desired
  replicasPopover.value?.toggle(event)
}

const saveReplicas = async () => {
  if (!editingDeployment.value) return
  isSavingReplicas.value = true
  try {
    await kubernetesService.scaleResource({
      namespace: editingDeployment.value.namespace,
      kind: KUBERNETES_RESOURCE_KIND.Deployment,
      name: editingDeployment.value.name,
      replicas: editReplicasValue.value
    })
    replicasPopover.value?.hide()
  } catch (e) {
    console.error('Error scaling deployment', e)
  } finally {
    isSavingReplicas.value = false
  }
}

// Inline Images Edit State
const imagePopover = ref<InstanceType<typeof Popover> | null>(null)
const editContainers = ref<ContainerImageInfo[]>([])
const isSavingImages = ref<boolean>(false)

const openImagesEdit = (event: Event, deployment: DeploymentInfo) => {
  editingDeployment.value = deployment
  if (deployment.containers && deployment.containers.length > 0) {
    editContainers.value = deployment.containers.map((c) => ({ ...c }))
  } else {
    editContainers.value = deployment.images.map((img, idx) => ({
      name: `container-${idx}`,
      image: img
    }))
  }
  imagePopover.value?.toggle(event)
}

const saveImages = async () => {
  if (!editingDeployment.value) return
  isSavingImages.value = true
  try {
    await kubernetesService.updateResourceImages({
      namespace: editingDeployment.value.namespace,
      kind: KUBERNETES_RESOURCE_KIND.Deployment,
      name: editingDeployment.value.name,
      containers: editContainers.value
    })
    imagePopover.value?.hide()
  } catch (e) {
    console.error('Error updating deployment images', e)
  } finally {
    isSavingImages.value = false
  }
}
</script>

<template>
  <GenericResourceTable
    :data="deployments"
    :initialColumns="columns"
    :statuses="statuses"
    :searchFields="['name', 'images']"
    :kind="KUBERNETES_RESOURCE_KIND.Deployment"
    searchPlaceholder="Search deployments or images..."
    emptyMessage="No deployments found matching the filter criteria."
    reportTemplate="Showing {first} to {last} of {totalRecords} deployments"
    :loading="loading || k8sStore.deploymentsLoading"
    :selectable="true"
    @refresh="fetchDeployments"
  >
    <template #default="{ visibleCols }">
      <!-- Conditions Column -->
      <Column v-if="visibleCols['conditions']" header="Conditions" class="p-3">
        <template #body="{ data }">
          <div class="flex flex-wrap gap-1">
            <template v-if="getOrderedConditions(data.conditions).length > 0">
              <Tag
                v-for="cond in getOrderedConditions(data.conditions)"
                :key="cond.type"
                :value="cond.type"
                :severity="getConditionSeverity(cond.type)"
                class="font-mono text-xs"
                v-tooltip.top="cond.message || cond.reason || cond.type"
              />
            </template>
            <span v-else class="text-muted-color">-</span>
          </div>
        </template>
      </Column>

      <!-- Replicas Column -->
      <Column v-if="visibleCols['replicas']" header="Replicas" class="p-3">
        <template #body="{ data }">
          <div
            class="group inline-flex items-center gap-1.5 font-mono text-muted-color px-1.5 py-0.5 rounded hover:bg-(--bg-hover) hover:text-primary transition-colors cursor-pointer"
            title="Click to edit replicas"
            @click.stop="openReplicasEdit($event, data)"
          >
            <span class="font-bold">{{ data.replicas.current }}</span>
            <span class="text-muted-color">/</span>
            <span>{{ data.replicas.desired }}</span>
            <Pencil
              class="w-3 h-3 opacity-0 group-hover:opacity-60 transition-opacity text-muted-color"
            />
          </div>
        </template>
      </Column>

      <!-- Available Column -->
      <Column
        v-if="visibleCols['available']"
        field="available"
        header="Available"
        sortable
        class="p-3"
      >
        <template #body="{ data }">
          <span
            class="font-mono"
            :class="
              data.available === data.replicas.desired ? 'text-emerald-500' : 'text-amber-500'
            "
          >
            {{ data.available }}
          </span>
        </template>
      </Column>

      <!-- Up to Date Column -->
      <Column
        v-if="visibleCols['upToDate']"
        field="upToDate"
        header="Up-To-Date"
        sortable
        class="p-3"
      >
        <template #body="{ data }">
          <span class="font-mono text-muted-color">{{ data.upToDate }}</span>
        </template>
      </Column>

      <!-- Restarts Column -->
      <Column
        v-if="visibleCols['restarts']"
        field="restarts"
        header="Restarts"
        sortable
        class="p-3"
      >
        <template #body="{ data }">
          <span class="font-mono text-muted-color">{{ data.restarts ?? 0 }}</span>
        </template>
      </Column>

      <!-- Images Column -->
      <Column v-if="visibleCols['images']" header="Images" class="p-3 max-w-48">
        <template #body="{ data }">
          <div
            class="group flex items-center gap-1 max-w-full min-w-0 cursor-pointer"
            title="Click to edit container images"
            @click.stop="openImagesEdit($event, data)"
          >
            <div class="flex flex-wrap gap-1 max-w-full min-w-0">
              <Tag
                v-for="img in data.images"
                :key="img"
                severity="secondary"
                class="group-hover:text-primary font-mono truncate max-w-full transition-colors inline-block"
                :title="img"
                :value="img.split('/').pop()"
              />
            </div>
            <Pencil
              class="w-3 h-3 shrink-0 opacity-0 group-hover:opacity-60 transition-opacity text-muted-color ml-0.5"
            />
          </div>
        </template>
      </Column>
    </template>

    <!-- Drawer -->
    <template #drawer="{ selectedItem, visible, close }">
      <WorkloadDetailsDrawer
        :visible="visible"
        :workload="selectedItem"
        @update:visible="!$event && close()"
      />
    </template>
  </GenericResourceTable>

  <!-- Inline Replicas Edit Popover -->
  <Popover ref="replicasPopover" class="shadow-lg">
    <div class="flex flex-col gap-3 p-1 min-w-56" @click.stop>
      <div class="flex flex-col gap-0.5">
        <span class="text-xs font-semibold tracking-wider text-muted-color uppercase">
          Scale Replicas
        </span>
        <span class="text-xs text-muted-color font-mono truncate">
          {{ editingDeployment?.name }}
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
          @keyup.enter="saveReplicas"
        />
        <Button label="Scale" size="small" :loading="isSavingReplicas" @click="saveReplicas" />
      </div>
    </div>
  </Popover>

  <!-- Inline Images Edit Popover -->
  <Popover ref="imagePopover" class="shadow-lg">
    <div class="flex flex-col gap-3 p-1 min-w-80" @click.stop>
      <div class="flex flex-col gap-0.5">
        <span class="text-xs font-semibold tracking-wider text-muted-color uppercase">
          Update Image
        </span>
        <span class="text-xs text-muted-color font-mono truncate">
          {{ editingDeployment?.name }}
        </span>
      </div>

      <div class="flex flex-col gap-2.5 max-h-60 overflow-y-auto">
        <div v-for="c in editContainers" :key="c.name" class="flex flex-col gap-1">
          <label class="text-xs text-muted-color font-medium">
            Container: <span class="font-mono text-primary">{{ c.name }}</span>
          </label>
          <InputText
            v-model="c.image"
            size="small"
            fluid
            class="text-xs font-mono"
            placeholder="e.g. nginx:latest"
            @keyup.enter="saveImages"
          />
        </div>
      </div>

      <div class="flex justify-end gap-2 pt-1 border-t border-(--border)">
        <Button label="Update" size="small" :loading="isSavingImages" @click="saveImages" />
      </div>
    </div>
  </Popover>
</template>
