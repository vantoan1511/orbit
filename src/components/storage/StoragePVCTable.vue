<script setup lang="ts">
import GenericResourceTable from '@/components/shared/GenericResourceTable.vue'
import { useKubernetesStore } from '@/stores/kubernetesStore'
import Column from 'primevue/column'

const k8sStore = useKubernetesStore()

const columns = [
  { field: 'name', header: 'Name', visible: true },
  { field: 'namespace', header: 'Namespace', visible: true },
  { field: 'volume', header: 'Volume', visible: true },
  { field: 'capacity', header: 'Request', visible: true },
  { field: 'accessMode', header: 'Access Mode', visible: true },
  { field: 'storageClass', header: 'Storage Class', visible: true },
  { field: 'status', header: 'Status', visible: true },
  { field: 'age', header: 'Age', visible: true }
]

const statuses = ['All Statuses', 'Bound', 'Pending', 'Lost']

const handleRefresh = async (namespace?: string) => {
  try {
    await k8sStore.fetchPersistentVolumeClaims(namespace)
  } catch (error) {
    console.error('Error fetching PVCs:', error)
  }
}
</script>

<template>
  <GenericResourceTable
    :data="k8sStore.persistentVolumeClaims"
    :initialColumns="columns"
    :statuses="statuses"
    :searchFields="['name', 'volume', 'storageClass']"
    kind="PersistentVolumeClaim"
    searchPlaceholder="Search Claims..."
    emptyMessage="No PVCs found matching the filter criteria."
    reportTemplate="Showing {first} to {last} of {totalRecords} claims"
    :loading="k8sStore.persistentVolumeClaimsLoading"
    @refresh="handleRefresh"
    @namespace-change="handleRefresh"
  >
    <template #default="{ visibleCols }">
      <!-- Volume Column -->
      <Column v-if="visibleCols['volume']" field="volume" header="Volume" sortable class="p-3">
        <template #body="{ data }">
          <span
            v-if="data.volume"
            class="font-mono text-muted-color truncate max-w-44 block"
            :title="data.volume"
          >
            {{ data.volume }}
          </span>
          <span v-else class="text-muted-color italic font-mono">-</span>
        </template>
      </Column>

      <!-- Capacity Column -->
      <Column
        v-if="visibleCols['capacity']"
        field="capacity"
        header="Request"
        sortable
        class="p-3"
        bodyClass="font-mono text-primary"
      ></Column>

      <!-- Access Mode Column -->
      <Column
        v-if="visibleCols['accessMode']"
        field="accessMode"
        header="Access Mode"
        sortable
        class="p-3"
        bodyClass="text-muted-color"
      ></Column>

      <!-- Storage Class Column -->
      <Column
        v-if="visibleCols['storageClass']"
        field="storageClass"
        header="Storage Class"
        sortable
        class="p-3"
      >
        <template #body="{ data }">
          <span class="font-mono text-muted-color font-semibold">{{ data.storageClass }}</span>
        </template>
      </Column>
    </template>
  </GenericResourceTable>
</template>
