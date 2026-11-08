<script setup lang="ts">
import GenericResourceTable from '@/components/shared/GenericResourceTable.vue'
import { KUBERNETES_RESOURCE_KIND, KUBERNETES_WORKLOAD_STATUS } from '@/constants/kubernetes'
import { kubernetesService } from '@/services/kubernetesService'
import { useKubernetesStore } from '@/stores/kubernetesStore'
import { Pencil } from '@lucide/vue'
import Column from 'primevue/column'
import Tag from 'primevue/tag'
import { computed, ref, shallowRef, watch } from 'vue'
import WorkloadDetailsDrawer from './WorkloadDetailsDrawer.vue'
import WorkloadInlineEditPopovers from './WorkloadInlineEditPopovers.vue'
import {
  computeDeploymentRestarts,
  createDeploymentRowDecorator,
  restartFingerprint
} from '@/utils/workloadMetrics'

const k8sStore = useKubernetesStore()
const loading = ref(false)
const inlineEdit = ref<InstanceType<typeof WorkloadInlineEditPopovers> | null>(null)

const deploymentDecorator = createDeploymentRowDecorator()
const deploymentRestartsMap = shallowRef(new Map<string, number>())

watch(
  () => restartFingerprint(k8sStore.pods),
  () => {
    deploymentRestartsMap.value = computeDeploymentRestarts(k8sStore.pods)
  },
  { immediate: true }
)

// Memoised decorator preserves row reference and computes replicasSortKey, orderedConditions, imageTags
const deployments = computed(() => {
  return deploymentDecorator(k8sStore.deployments, deploymentRestartsMap.value)
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
</script>

<template>
  <GenericResourceTable
    data-component="deployments-table"
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
            <template v-if="data.orderedConditions && data.orderedConditions.length > 0">
              <Tag
                v-for="cond in data.orderedConditions"
                :key="cond.type"
                :value="cond.type"
                :severity="cond.severity"
                class="font-mono text-xs"
                v-tooltip.top="cond.tooltip"
              />
            </template>
            <span v-else class="text-muted-color">-</span>
          </div>
        </template>
      </Column>

      <!-- Replicas Column -->
      <Column
        v-if="visibleCols['replicas']"
        field="replicas"
        sortField="replicasSortKey"
        header="Replicas"
        sortable
        class="p-3"
      >
        <template #body="{ data }">
          <div
            aria-label="Click to edit replicas"
            class="group inline-flex items-center gap-1.5 font-mono text-muted-color px-1.5 py-0.5 rounded-[var(--ui-radius-control)] hover:bg-(--bg-hover) hover:text-primary transition-colors cursor-pointer"
            @click.stop="inlineEdit?.openReplicasEdit($event, data)"
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
              data.available === data.replicas.desired ? 'text-(--success)' : 'text-(--warning)'
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
            aria-label="Click to edit container images"
            class="group flex items-center gap-1 max-w-full min-w-0 cursor-pointer"
            @click.stop="inlineEdit?.openImagesEdit($event, data)"
          >
            <div class="flex flex-wrap gap-1 max-w-full min-w-0">
              <Tag
                v-for="img in data.imageTags"
                :key="img.full"
                severity="secondary"
                class="group-hover:text-primary font-mono truncate max-w-full transition-colors inline-block"
                v-tooltip.top="img.full"
                :value="img.short"
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

  <WorkloadInlineEditPopovers ref="inlineEdit" :kind="KUBERNETES_RESOURCE_KIND.Deployment" />
</template>
