<script setup lang="ts">
import { computed, ref } from 'vue'
import Column from 'primevue/column'
import Select from 'primevue/select'
import Button from 'primevue/button'
import { AlertCircle, MoreVertical } from '@lucide/vue'
import { useKubernetesStore } from '@/stores/kubernetesStore'
import ResourceActionMenu from '@/components/shared/ResourceActionMenu.vue'
import ResourceDataTable from '@/components/shared/ResourceDataTable.vue'
import { useWorkloadActions } from '@/composables/useWorkloadActions'
import { useResourceFilters } from '@/composables/useResourceFilters'
import { useTableColumns } from '@/composables/useTableColumns'
import type { PersistentVolumeInfo } from '@/types/kubernetes'

const k8sStore = useKubernetesStore()

const pvsWithNamespace = computed(() =>
  k8sStore.persistentVolumes.map((pv) => ({ ...pv, namespace: '' }))
)

const { searchQuery, filteredResources } = useResourceFilters(pvsWithNamespace, [
  'name',
  'storageClass'
])

const { tableColumns, visibleCols } = useTableColumns([
  { field: 'name', header: 'Name', visible: true },
  { field: 'capacity', header: 'Capacity', visible: true },
  { field: 'accessMode', header: 'Access Mode', visible: true },
  { field: 'reclaimPolicy', header: 'Reclaim', visible: true },
  { field: 'storageClass', header: 'Storage Class', visible: true },
  { field: 'status', header: 'Status', visible: true },
  { field: 'age', header: 'Age', visible: true }
])

const selectedStorageClass = ref('All Storage Classes')
const selectedStatus = ref('All Statuses')

const storageClasses = computed(() => {
  const classes = new Set(k8sStore.persistentVolumes.map((pv) => pv.storageClass))
  return ['All Storage Classes', ...Array.from(classes)]
})

const statuses = ['All Statuses', 'Bound', 'Available', 'Released', 'Failed']

const filteredPVs = computed(() => {
  return filteredResources.value.filter((pv) => {
    if (
      selectedStorageClass.value !== 'All Storage Classes' &&
      pv.storageClass !== selectedStorageClass.value
    ) {
      return false
    }

    if (selectedStatus.value !== 'All Statuses' && pv.status !== selectedStatus.value) {
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

const actionMenu = ref<InstanceType<typeof ResourceActionMenu> | null>(null)
const selectedActionRow = ref<PersistentVolumeInfo | null>(null)

const toggleActionMenu = (event: Event, data: PersistentVolumeInfo) => {
  event.stopPropagation()
  selectedActionRow.value = data
  actionMenu.value?.toggle(event)
}

const { actionMenuItems } = useWorkloadActions(selectedActionRow, {
  kind: 'PersistentVolume'
})
</script>

<template>
  <ResourceDataTable
    :data="filteredPVs"
    v-model:searchQuery="searchQuery"
    v-model:columns="tableColumns"
    searchPlaceholder="Search PVs..."
    emptyMessage="No Persistent Volumes found matching the filter criteria."
    reportTemplate="Showing {first} to {last} of {totalRecords} volumes"
    :loading="k8sStore.persistentVolumesLoading"
    @refresh="handleRefresh"
  >
    <!-- Filters -->
    <template #filters>
      <Select v-model="selectedStorageClass" :options="storageClasses" class="text-xs min-w-40" />
      <Select v-model="selectedStatus" :options="statuses" class="text-xs min-w-36" />
    </template>

    <!-- Name Column -->
    <Column
      v-if="visibleCols['name']"
      field="name"
      header="Name"
      sortable
      class="p-3"
      bodyClass="font-medium text-primary"
    >
      <template #body="{ data }">
        <div class="flex items-center gap-1.5">
          <span
            class="font-semibold hover:text-violet-400 transition-colors font-mono truncate max-w-56"
            :title="data.name"
          >
            {{ data.name }}
          </span>
          <AlertCircle
            v-if="data.status === 'Failed'"
            class="w-3.5 h-3.5 text-rose-400"
            :title="data.reason"
          />
        </div>
      </template>
    </Column>

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
        <span class="font-mono text-violet-400 font-semibold">{{ data.storageClass }}</span>
      </template>
    </Column>

    <!-- Status Column -->
    <Column v-if="visibleCols['status']" field="status" header="Status" sortable class="p-3">
      <template #body="{ data }">
        <div class="flex items-center gap-1.5">
          <span
            class="w-1.5 h-1.5 rounded-full"
            :class="{
              'bg-emerald-500': data.status === 'Bound',
              'bg-blue-500': data.status === 'Available',
              'bg-amber-500': data.status === 'Released',
              'bg-rose-500': data.status === 'Failed'
            }"
          ></span>
          <span
            class="font-medium"
            :class="{
              'text-emerald-500': data.status === 'Bound',
              'text-blue-500': data.status === 'Available',
              'text-amber-500': data.status === 'Released',
              'text-rose-500': data.status === 'Failed'
            }"
          >
            {{ data.status }}
          </span>
        </div>
      </template>
    </Column>

    <!-- Age Column -->
    <Column
      v-if="visibleCols['age']"
      field="age"
      header="Age"
      sortable
      class="p-3"
      bodyClass="text-muted-color font-mono"
    ></Column>

    <!-- Actions Column -->
    <Column class="p-3 text-center w-12 shrink-0">
      <template #body="{ data }">
        <Button
          severity="secondary"
          variant="text"
          size="small"
          class="p-1"
          title="Actions"
          @click="toggleActionMenu($event, data)"
        >
          <MoreVertical class="w-4 h-4 text-muted-color" />
        </Button>
      </template>
    </Column>

    <!-- Drawer -->
    <template #drawer>
      <ResourceActionMenu ref="actionMenu" :items="actionMenuItems" />
    </template>
  </ResourceDataTable>
</template>
