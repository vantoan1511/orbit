<script setup lang="ts">
import { computed, ref } from 'vue'
import Column from 'primevue/column'
import Button from 'primevue/button'
import { MoreVertical } from '@lucide/vue'
import { useKubernetesStore } from '@/stores/kubernetesStore'
import ResourceActionMenu from '@/components/shared/ResourceActionMenu.vue'
import { useResourceActionMenu } from '@/composables/useResourceActionMenu'
import ResourceDataTable from '@/components/shared/ResourceDataTable.vue'
import { useWorkloadActions } from '@/composables/useWorkloadActions'
import { useResourceFilters } from '@/composables/useResourceFilters'
import { useTableColumns } from '@/composables/useTableColumns'
import type { StorageClassInfo } from '@/types/kubernetes'

const k8sStore = useKubernetesStore()

const storageClassesWithNamespace = computed(() =>
  k8sStore.storageClasses.map((sc) => ({ ...sc, namespace: '' }))
)

const { searchQuery, filteredResources } = useResourceFilters(storageClassesWithNamespace, [
  'name',
  'provisioner'
])

const { tableColumns, visibleCols } = useTableColumns([
  { field: 'name', header: 'Name', visible: true },
  { field: 'provisioner', header: 'Provisioner', visible: true },
  { field: 'reclaimPolicy', header: 'Reclaim Policy', visible: true },
  { field: 'volumeBindingMode', header: 'Volume Binding Mode', visible: true },
  { field: 'allowVolumeExpansion', header: 'Allow Volume Expansion', visible: true },
  { field: 'age', header: 'Age', visible: true }
])

const handleRefresh = async () => {
  try {
    await k8sStore.fetchStorageClasses()
  } catch (error) {
    console.error('Error fetching Storage Classes:', error)
  }
}

const { actionMenu, selectedActionRow, toggleActionMenu, onRowContextMenu } =
  useResourceActionMenu<StorageClassInfo & { namespace: string }>()

const { actionMenuItems } = useWorkloadActions(selectedActionRow, {
  kind: 'StorageClass'
})
</script>

<template>
  <ResourceDataTable
    :data="filteredResources"
    v-model:searchQuery="searchQuery"
    v-model:columns="tableColumns"
    searchPlaceholder="Search Storage Classes..."
    emptyMessage="No Storage Classes found matching the filter criteria."
    reportTemplate="Showing {first} to {last} of {totalRecords} classes"
    :loading="k8sStore.storageClassesLoading"
    @refresh="handleRefresh"
    @row-contextmenu="onRowContextMenu"
  >
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
        <span class="font-semibold font-mono">{{ data.name }}</span>
      </template>
    </Column>

    <!-- Provisioner Column -->
    <Column
      v-if="visibleCols['provisioner']"
      field="provisioner"
      header="Provisioner"
      sortable
      class="p-3"
      bodyClass="font-mono text-muted-color"
    ></Column>

    <!-- Reclaim Policy Column -->
    <Column
      v-if="visibleCols['reclaimPolicy']"
      field="reclaimPolicy"
      header="Reclaim Policy"
      sortable
      class="p-3"
      bodyClass="text-muted-color"
    ></Column>

    <!-- Volume Binding Mode Column -->
    <Column
      v-if="visibleCols['volumeBindingMode']"
      field="volumeBindingMode"
      header="Volume Binding Mode"
      sortable
      class="p-3"
      bodyClass="text-muted-color"
    ></Column>

    <!-- Allow Volume Expansion Column -->
    <Column
      v-if="visibleCols['allowVolumeExpansion']"
      field="allowVolumeExpansion"
      header="Allow Volume Expansion"
      sortable
      class="p-3"
    >
      <template #body="{ data }">
        <span
          class="px-2 py-0.5 rounded text-[10px] font-bold uppercase tracking-wider"
          :class="
            data.allowVolumeExpansion
              ? 'bg-emerald-500/10 text-emerald-400 border border-emerald-500/20'
              : 'bg-zinc-500/10 text-zinc-400 border border-zinc-500/20'
          "
        >
          {{ data.allowVolumeExpansion ? 'True' : 'False' }}
        </span>
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
