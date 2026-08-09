<script setup lang="ts">
import { computed, ref } from 'vue'
import Column from 'primevue/column'
import Select from 'primevue/select'
import Button from 'primevue/button'
import { MoreVertical } from '@lucide/vue'
import { useKubernetesStore } from '@/stores/kubernetesStore'
import ResourceActionMenu from '@/components/shared/ResourceActionMenu.vue'
import ResourceDataTable from '@/components/shared/ResourceDataTable.vue'
import NamespaceFilter from '@/components/shared/NamespaceFilter.vue'
import NamespaceBadge from '@/components/shared/NamespaceBadge.vue'
import { useWorkloadActions } from '@/composables/useWorkloadActions'
import { useResourceFilters } from '@/composables/useResourceFilters'
import { useTableColumns } from '@/composables/useTableColumns'
import type { PersistentVolumeClaimInfo } from '@/types/kubernetes'

const k8sStore = useKubernetesStore()

const { searchQuery, selectedNamespace, filteredResources } = useResourceFilters(
  computed(() => k8sStore.persistentVolumeClaims),
  ['name', 'volume', 'storageClass']
)

const { tableColumns, visibleCols } = useTableColumns([
  { field: 'name', header: 'Name', visible: true },
  { field: 'namespace', header: 'Namespace', visible: true },
  { field: 'volume', header: 'Volume', visible: true },
  { field: 'capacity', header: 'Request', visible: true },
  { field: 'accessMode', header: 'Access Mode', visible: true },
  { field: 'storageClass', header: 'Storage Class', visible: true },
  { field: 'status', header: 'Status', visible: true },
  { field: 'age', header: 'Age', visible: true }
])

const selectedStatus = ref('All Statuses')
const namespaces = computed(() => k8sStore.namespaces)
const statuses = ['All Statuses', 'Bound', 'Pending', 'Lost']

const filteredPVCs = computed(() => {
  return filteredResources.value.filter((pvc) => {
    if (selectedStatus.value !== 'All Statuses' && pvc.status !== selectedStatus.value) {
      return false
    }
    return true
  })
})

const handleRefresh = async () => {
  try {
    const ns = selectedNamespace.value.length === 1 ? selectedNamespace.value[0] : undefined
    await k8sStore.fetchPersistentVolumeClaims(ns)
  } catch (error) {
    console.error('Error fetching PVCs:', error)
  }
}

const actionMenu = ref<InstanceType<typeof ResourceActionMenu> | null>(null)
const selectedActionRow = ref<PersistentVolumeClaimInfo | null>(null)

const toggleActionMenu = (event: Event, data: PersistentVolumeClaimInfo) => {
  event.stopPropagation()
  selectedActionRow.value = data
  actionMenu.value?.toggle(event)
}

const { actionMenuItems } = useWorkloadActions(selectedActionRow, {
  kind: 'PersistentVolumeClaim'
})
</script>

<template>
  <ResourceDataTable
    :data="filteredPVCs"
    v-model:searchQuery="searchQuery"
    v-model:columns="tableColumns"
    searchPlaceholder="Search Claims..."
    emptyMessage="No PVCs found matching the filter criteria."
    reportTemplate="Showing {first} to {last} of {totalRecords} claims"
    :loading="k8sStore.persistentVolumeClaimsLoading"
    @refresh="handleRefresh"
  >
    <!-- Filters -->
    <template #filters>
      <NamespaceFilter v-model="selectedNamespace" :namespaces="namespaces" />
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
        <span
          class="font-semibold hover:text-violet-400 transition-colors font-mono truncate max-w-48 block"
          :title="data.name"
        >
          {{ data.name }}
        </span>
      </template>
    </Column>

    <!-- Namespace Column -->
    <Column
      v-if="visibleCols['namespace']"
      field="namespace"
      header="Namespace"
      sortable
      class="p-3"
    >
      <template #body="{ data }">
        <NamespaceBadge :namespace="data.namespace" />
      </template>
    </Column>

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
              'bg-amber-500': data.status === 'Pending',
              'bg-rose-500': data.status === 'Lost'
            }"
          ></span>
          <span
            class="font-medium"
            :class="{
              'text-emerald-500': data.status === 'Bound',
              'text-amber-500': data.status === 'Pending',
              'text-rose-500': data.status === 'Lost'
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
