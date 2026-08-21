<script setup lang="ts">
import GenericResourceTable from '@/components/shared/GenericResourceTable.vue'
import { kubernetesService } from '@/services/kubernetesService'
import { useKubernetesStore } from '@/stores/kubernetesStore'
import type { DeploymentInfo } from '@/types/kubernetes'
import Column from 'primevue/column'
import { computed, onMounted, ref, watch } from 'vue'
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
  { field: 'replicas', header: 'Replicas', visible: true },
  { field: 'available', header: 'Available', visible: true },
  { field: 'upToDate', header: 'Up-To-Date', visible: true },
  { field: 'restarts', header: 'Restarts', visible: true },
  { field: 'age', header: 'Age', visible: true },
  { field: 'images', header: 'Images', visible: true }
]

const statuses = ['All Statuses', 'Running', 'Progressing', 'Failed']

const fetchDeployments = async (namespace?: string) => {
  loading.value = true
  try {
    await kubernetesService.getDeployments(namespace)
  } catch (e) {
    console.error('Error fetching deployments', e)
  } finally {
    loading.value = false
  }
}

onMounted(() => {
  if (k8sStore.deployments.length === 0 && !k8sStore.deploymentsLoading) {
    fetchDeployments()
  }
})

// Refetch on cluster change
watch(
  () => k8sStore.activeClusterId,
  () => {
    fetchDeployments()
  }
)
</script>

<template>
  <GenericResourceTable
    :data="deployments"
    :initialColumns="columns"
    :statuses="statuses"
    :searchFields="['name', 'images']"
    kind="Deployment"
    searchPlaceholder="Search deployments or images..."
    emptyMessage="No deployments found matching the filter criteria."
    reportTemplate="Showing {first} to {last} of {totalRecords} deployments"
    :loading="loading || k8sStore.deploymentsLoading"
    :selectable="true"
    @refresh="fetchDeployments"
    @namespace-change="fetchDeployments"
  >
    <template #default="{ visibleCols }">
      <!-- Replicas Column -->
      <Column v-if="visibleCols['replicas']" header="Replicas" class="p-3">
        <template #body="{ data }">
          <div class="flex items-center gap-2 font-mono text-muted-color">
            <span class="font-bold">{{ data.replicas.current }}</span>
            <span class="text-muted-color">/</span>
            <span>{{ data.replicas.desired }}</span>
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
          <div class="flex flex-wrap gap-1">
            <span
              v-for="img in data.images"
              :key="img"
              class="px-1.5 py-0.5 rounded bg-(--bg-hover) text-muted-color text-xs font-mono truncate max-w-full"
              :title="img"
            >
              {{ img.split('/').pop() }}
            </span>
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
</template>
