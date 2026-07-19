<script setup lang="ts">
import NamespaceBadge from '@/components/shared/NamespaceBadge.vue'
import NamespaceFilter from '@/components/shared/NamespaceFilter.vue'
import ResourceDataTable from '@/components/shared/ResourceDataTable.vue'
import SystemNamespaceToggle from '@/components/shared/SystemNamespaceToggle.vue'
import { useResourceFilters } from '@/composables/useResourceFilters'
import { useTableColumns } from '@/composables/useTableColumns'
import { kubernetesService } from '@/services/kubernetesService'
import { useKubernetesStore } from '@/stores/kubernetesStore'
import type { ServiceInfo } from '@/types/kubernetes'
import { ExternalLink, MoreVertical } from '@lucide/vue'
import Button from 'primevue/button'
import Column from 'primevue/column'
import Select from 'primevue/select'
import { computed, ref } from 'vue'
import ServiceDetailsDrawer from './ServiceDetailsDrawer.vue'
import ResourceActionMenu from '@/components/shared/ResourceActionMenu.vue'
import { useWorkloadActions } from '@/composables/useWorkloadActions'

const k8sStore = useKubernetesStore()

const { tableColumns, visibleCols } = useTableColumns([
  { field: 'namespace', header: 'Namespace', visible: true },
  { field: 'type', header: 'Type', visible: true },
  { field: 'clusterIP', header: 'Cluster IP', visible: true },
  { field: 'externalIP', header: 'External IP', visible: true },
  { field: 'ports', header: 'Ports', visible: true },
  { field: 'endpoints', header: 'Endpoints', visible: true },
  { field: 'age', header: 'Age', visible: true }
])

const { searchQuery, selectedNamespace, showSystemNamespaces, filteredResources } =
  useResourceFilters(computed(() => k8sStore.services))

const selectedType = ref('All Types')

// Drawer state
const drawerVisible = ref(false)
const selectedService = ref<ServiceInfo | null>(null)

const handleRefresh = async () => {
  await kubernetesService.getServices()
}

const types = ['All Types', 'ClusterIP', 'NodePort', 'LoadBalancer', 'ExternalName']

const filteredServices = computed(() => {
  return filteredResources.value.filter((s) => {
    // Type filter
    if (selectedType.value !== 'All Types' && s.type !== selectedType.value) {
      return false
    }

    return true
  })
})

const onRowClick = (event: { data: ServiceInfo }) => {
  selectedService.value = event.data
  drawerVisible.value = true
}

const getTypeBadgeClass = (type: string) => {
  switch (type) {
    case 'LoadBalancer':
      return 'bg-purple-500/10 text-purple-400 border border-purple-500/20'
    case 'ClusterIP':
      return 'bg-emerald-500/10 text-emerald-400 border border-emerald-500/20'
    case 'NodePort':
      return 'bg-blue-500/10 text-blue-400 border border-blue-500/20'
    case 'ExternalName':
      return 'bg-amber-500/10 text-amber-400 border border-amber-500/20'
    default:
      return 'bg-gray-500/10 text-gray-400 border border-gray-500/20'
  }
}

const actionMenu = ref<InstanceType<typeof ResourceActionMenu> | null>(null)
const selectedActionRow = ref<ServiceInfo | null>(null)

const toggleActionMenu = (event: Event, data: ServiceInfo) => {
  event.stopPropagation()
  selectedActionRow.value = data
  actionMenu.value?.toggle(event)
}

const { actionMenuItems } = useWorkloadActions(selectedActionRow, {
  kind: 'Service',
  onViewDetails: (row) => {
    selectedService.value = row
    drawerVisible.value = true
  }
})
</script>

<template>
  <ResourceDataTable
    :data="filteredServices"
    v-model:searchQuery="searchQuery"
    v-model:columns="tableColumns"
    searchPlaceholder="Search services..."
    emptyMessage="No services found matching the filter criteria."
    reportTemplate="Showing {first} to {last} of {totalRecords} services"
    :rows="12"
    @refresh="handleRefresh"
    @row-click="onRowClick"
  >
    <!-- Filters -->
    <template #filters>
      <!-- Namespace Select -->
      <NamespaceFilter v-model="selectedNamespace" :namespaces="k8sStore.namespaces" />

      <!-- Type Select -->
      <Select
        v-model="selectedType"
        :options="types"
        class="text-xs min-w-40 bg-(--bg-hover)/30 border-(--border)"
      />
    </template>

    <!-- Actions Left -->
    <template #actions-left>
      <SystemNamespaceToggle v-model="showSystemNamespaces" />
    </template>

    <!-- Columns -->
    <!-- Name Column -->
    <Column field="name" header="Name" sortable class="font-medium p-3 text-(--text-primary)">
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

    <!-- Type Column -->
    <Column v-if="visibleCols['type']" field="type" header="Type" sortable class="p-3">
      <template #body="{ data }">
        <span
          class="px-2 py-0.5 rounded text-[10px] font-semibold tracking-wider font-ui border"
          :class="getTypeBadgeClass(data.type)"
        >
          {{ data.type }}
        </span>
      </template>
    </Column>

    <!-- Cluster IP Column -->
    <Column
      v-if="visibleCols['clusterIP']"
      field="clusterIP"
      header="Cluster IP"
      sortable
      class="p-3"
    >
      <template #body="{ data }">
        <span class="font-mono text-(--text-secondary)">{{ data.clusterIP }}</span>
      </template>
    </Column>

    <!-- External IP Column -->
    <Column
      v-if="visibleCols['externalIP']"
      field="externalIP"
      header="External IP"
      sortable
      class="p-3"
    >
      <template #body="{ data }">
        <div class="flex items-center gap-1">
          <span class="font-mono text-(--text-secondary)">{{ data.externalIP }}</span>
          <ExternalLink
            v-if="data.externalIP !== '-'"
            class="w-3 h-3 text-violet-400 hover:text-violet-300 cursor-pointer"
          />
        </div>
      </template>
    </Column>

    <!-- Ports Column -->
    <Column v-if="visibleCols['ports']" field="ports" header="Ports" sortable class="p-3">
      <template #body="{ data }">
        <span class="font-mono text-(--text-muted) whitespace-pre-line">{{ data.ports }}</span>
      </template>
    </Column>

    <!-- Endpoints Column -->
    <Column
      v-if="visibleCols['endpoints']"
      field="endpoints"
      header="Endpoints"
      sortable
      class="p-3"
    >
      <template #body="{ data }">
        <span class="font-mono font-medium text-emerald-400">{{ data.endpoints }}</span>
      </template>
    </Column>

    <!-- Age Column -->
    <Column
      v-if="visibleCols['age']"
      field="age"
      header="Age"
      sortable
      class="p-3 text-(--text-muted) font-mono"
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
          <MoreVertical class="w-4 h-4 text-(--text-muted)" />
        </Button>
      </template>
    </Column>

    <!-- Drawer -->
    <template #drawer>
      <ServiceDetailsDrawer v-model:visible="drawerVisible" :service="selectedService" />
      <ResourceActionMenu ref="actionMenu" :items="actionMenuItems" />
    </template>
  </ResourceDataTable>
</template>
