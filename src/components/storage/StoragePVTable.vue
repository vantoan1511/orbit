<script setup lang="ts">
import GenericResourceTable from '@/components/shared/GenericResourceTable.vue'
import TableFilterSelect from '@/components/shared/TableFilterSelect.vue'
import { useKubernetesStore } from '@/stores/kubernetesStore'
import { useTableFilterStore } from '@/stores/tableFilterStore'
import { AlertCircle } from '@lucide/vue'
import Column from 'primevue/column'
import { computed } from 'vue'

const k8sStore = useKubernetesStore()
const filterStore = useTableFilterStore()
const STORE_KEY = 'persistentvolume'

const columns = [
  { field: 'name', header: 'Name', visible: true },
  { field: 'capacity', header: 'Capacity', visible: true },
  { field: 'accessMode', header: 'Access Mode', visible: true },
  { field: 'reclaimPolicy', header: 'Reclaim', visible: true },
  { field: 'storageClass', header: 'Storage Class', visible: true },
  { field: 'status', header: 'Status', visible: true },
  { field: 'age', header: 'Age', visible: true }
]

const selectedStorageClass = computed({
  get: () => filterStore.getExtraFilter(STORE_KEY, 'storageClass', 'All Storage Classes'),
  set: (val: string) => filterStore.setExtraFilter(STORE_KEY, 'storageClass', val)
})
const statuses = ['All Statuses', 'Bound', 'Available', 'Released', 'Failed']

const storageClasses = computed(() => {
  const classes = new Set(k8sStore.persistentVolumes.map((pv) => pv.storageClass))
  return ['All Storage Classes', ...Array.from(classes)]
})

const filteredPVs = computed(() => {
  return k8sStore.persistentVolumes.filter((pv) => {
    if (
      selectedStorageClass.value !== 'All Storage Classes' &&
      pv.storageClass !== selectedStorageClass.value
    ) {
      return false
    }
    return true
  })
})

const handleRefresh = async () => {
  try {
    await k8sStore.fetchPersistentVolumes()
  } catch (error) {
    console.error('Error fetching PVs:', error)
  }
}
</script>

<template>
  <GenericResourceTable
    :data="filteredPVs"
    :initialColumns="columns"
    :statuses="statuses"
    :searchFields="['name', 'storageClass']"
    :hideNamespaceFilter="true"
    :hideNamespaceColumn="true"
    :hideSystemNamespaceToggle="true"
    kind="PersistentVolume"
    searchPlaceholder="Search PVs..."
    emptyMessage="No Persistent Volumes found matching the filter criteria."
    reportTemplate="Showing {first} to {last} of {totalRecords} volumes"
    :loading="k8sStore.persistentVolumesLoading"
    @refresh="handleRefresh"
  >
    <!-- Extra Filter -->
    <template #filters>
      <TableFilterSelect v-model="selectedStorageClass" :options="storageClasses" />
    </template>

    <!-- Custom Name with Alert -->
    <template #name="{ data }">
      <div class="flex items-center gap-1.5">
        <span class="font-semibold font-mono truncate max-w-56" :title="data.name">
          {{ data.name }}
        </span>
        <AlertCircle
          v-if="data.status === 'Failed'"
          class="w-3.5 h-3.5 text-rose-400"
          :title="data.reason"
        />
      </div>
    </template>

    <template #default="{ visibleCols }">
      <!-- Capacity Column -->
      <Column
        v-if="visibleCols['capacity']"
        field="capacity"
        header="Capacity"
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

      <!-- Reclaim Policy Column -->
      <Column
        v-if="visibleCols['reclaimPolicy']"
        field="reclaimPolicy"
        header="Reclaim"
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
