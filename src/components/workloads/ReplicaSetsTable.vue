<script setup lang="ts">
import GenericResourceTable from '@/components/shared/GenericResourceTable.vue'
import { KUBERNETES_RESOURCE_KIND, KUBERNETES_WORKLOAD_STATUS } from '@/constants/kubernetes'
import { kubernetesService } from '@/services/kubernetesService'
import { useKubernetesStore } from '@/stores/kubernetesStore'
import Column from 'primevue/column'
import { ref } from 'vue'
import WorkloadDetailsDrawer from './WorkloadDetailsDrawer.vue'

const k8sStore = useKubernetesStore()
const loading = ref(false)

const columns = [
  { field: 'namespace', header: 'Namespace', visible: true },
  { field: 'status', header: 'Status', visible: true },
  { field: 'replicas', header: 'Replicas', visible: true },
  { field: 'age', header: 'Age', visible: true },
  { field: 'images', header: 'Images', visible: true }
]

const statuses = [
  'All Statuses',
  KUBERNETES_WORKLOAD_STATUS.Running,
  KUBERNETES_WORKLOAD_STATUS.Progressing
]

const fetchReplicaSets = async () => {
  loading.value = true
  try {
    await kubernetesService.getReplicaSets()
  } catch (e) {
    console.error('Error fetching replicasets', e)
  } finally {
    loading.value = false
  }
}
</script>

<template>
  <GenericResourceTable
    :data="k8sStore.replicaSets"
    :initialColumns="columns"
    :statuses="statuses"
    :searchFields="['name', 'images']"
    :kind="KUBERNETES_RESOURCE_KIND.ReplicaSet"
    searchPlaceholder="Search replicasets or images..."
    emptyMessage="No replicasets found matching the filter criteria."
    reportTemplate="Showing {first} to {last} of {totalRecords} replicasets"
    :loading="loading || k8sStore.replicaSetsLoading"
    @refresh="fetchReplicaSets"
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

      <!-- Images Column -->
      <Column v-if="visibleCols['images']" header="Images" class="p-3 max-w-48">
        <template #body="{ data }">
          <div class="flex flex-wrap gap-1">
            <Tag
              v-for="img in data.images"
              :key="img"
              severity="secondary"
              class="font-mono truncate max-w-full"
              :title="img"
              :value="img.split('/').pop()"
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
</template>
