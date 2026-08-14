<script setup lang="ts">
import NamespaceBadge from '@/components/shared/NamespaceBadge.vue'
import NamespaceFilter from '@/components/shared/NamespaceFilter.vue'
import ResourceActionMenu from '@/components/shared/ResourceActionMenu.vue'
import ResourceDataTable from '@/components/shared/ResourceDataTable.vue'
import SystemNamespaceToggle from '@/components/shared/SystemNamespaceToggle.vue'
import { useResourceActionMenu } from '@/composables/useResourceActionMenu'
import { useResourceFilters } from '@/composables/useResourceFilters'
import { useTableColumns } from '@/composables/useTableColumns'
import { useWorkloadActions } from '@/composables/useWorkloadActions'
import { kubernetesService } from '@/services/kubernetesService'
import { useKubernetesStore } from '@/stores/kubernetesStore'
import type { IngressInfo } from '@/types/kubernetes'
import { MoreVertical } from '@lucide/vue'
import Button from 'primevue/button'
import Column from 'primevue/column'
import { computed, ref } from 'vue'
import IngressDetailsDrawer from './IngressDetailsDrawer.vue'

const k8sStore = useKubernetesStore()

const { tableColumns, visibleCols } = useTableColumns([
  { field: 'namespace', header: 'Namespace', visible: true },
  { field: 'className', header: 'Class', visible: true },
  { field: 'hosts', header: 'Hosts', visible: true },
  { field: 'address', header: 'Address', visible: true },
  { field: 'ports', header: 'Ports', visible: true },
  { field: 'age', header: 'Age', visible: true }
])

const { searchQuery, selectedNamespace, showSystemNamespaces, filteredResources } =
  useResourceFilters(computed(() => k8sStore.ingresses))

// Drawer state
const drawerVisible = ref(false)
const selectedIngress = ref<IngressInfo | null>(null)

const handleRefresh = async () => {
  await kubernetesService.getIngresses()
}

const onRowClick = (event: { data: IngressInfo }) => {
  selectedIngress.value = event.data
  drawerVisible.value = true
}

const { actionMenu, selectedActionRow, toggleActionMenu, onRowContextMenu } =
  useResourceActionMenu<IngressInfo>()

const { actionMenuItems } = useWorkloadActions(selectedActionRow, {
  kind: 'Ingress',
  onViewDetails: (row) => {
    selectedIngress.value = row
    drawerVisible.value = true
  }
})
</script>

<template>
  <ResourceDataTable
    :data="filteredResources"
    v-model:searchQuery="searchQuery"
    v-model:columns="tableColumns"
    searchPlaceholder="Search ingresses..."
    emptyMessage="No ingresses found matching the filter criteria."
    reportTemplate="Showing {first} to {last} of {totalRecords} ingresses"
    :loading="k8sStore.ingressesLoading"
    @refresh="handleRefresh"
    @row-click="onRowClick"
    @row-contextmenu="onRowContextMenu"
  >
    <!-- Filters -->
    <template #filters>
      <!-- Namespace Select -->
      <NamespaceFilter v-model="selectedNamespace" :namespaces="k8sStore.namespaces" />
    </template>

    <!-- Actions Left -->
    <template #actions-left>
      <SystemNamespaceToggle v-model="showSystemNamespaces" />
    </template>

    <!-- Columns -->
    <!-- Name Column -->
    <Column field="name" header="Name" sortable class="p-3" bodyClass="font-medium text-primary">
      <template #body="{ data }">
        <div class="flex items-center gap-2">
          <span class="w-2 h-2 rounded-full bg-emerald-500 shrink-0"></span>
          <span class="font-semibold text-violet-400 hover:text-violet-300 transition-colors">{{
            data.name
          }}</span>
        </div>
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

    <!-- Class Column -->
    <Column v-if="visibleCols['className']" field="className" header="Class" sortable class="p-3">
      <template #body="{ data }">
        <span
          class="font-mono text-xs px-2 py-0.5 rounded bg-surface-800 text-surface-200 border border-surface-700"
        >
          {{ data.className || '-' }}
        </span>
      </template>
    </Column>

    <!-- Hosts Column -->
    <Column v-if="visibleCols['hosts']" field="hosts" header="Hosts" sortable class="p-3">
      <template #body="{ data }">
        <span class="font-mono text-xs text-violet-400 whitespace-pre-line">{{ data.hosts }}</span>
      </template>
    </Column>

    <!-- Address Column -->
    <Column v-if="visibleCols['address']" field="address" header="Address" sortable class="p-3">
      <template #body="{ data }">
        <span class="font-mono text-xs text-muted-color">{{ data.address }}</span>
      </template>
    </Column>

    <!-- Ports Column -->
    <Column v-if="visibleCols['ports']" field="ports" header="Ports" sortable class="p-3">
      <template #body="{ data }">
        <span class="font-mono text-xs text-muted-color">{{ data.ports }}</span>
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
      <IngressDetailsDrawer v-model:visible="drawerVisible" :ingress="selectedIngress" />
      <ResourceActionMenu ref="actionMenu" :items="actionMenuItems" />
    </template>
  </ResourceDataTable>
</template>
