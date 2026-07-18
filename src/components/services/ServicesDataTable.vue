<script setup lang="ts">
import ResourceDataTable, { type TableColumn } from '@/components/shared/ResourceDataTable.vue'
import { kubernetesService } from '@/services/kubernetesService'
import { useKubernetesStore } from '@/stores/kubernetesStore'
import type { ServiceInfo } from '@/types/kubernetes'
import { ExternalLink, MoreVertical } from '@lucide/vue'
import Button from 'primevue/button'
import Column from 'primevue/column'
import Select from 'primevue/select'
import ToggleSwitch from 'primevue/toggleswitch'
import { useToast } from 'primevue/usetoast'
import { computed, ref } from 'vue'
import ServiceDetailsDrawer from './ServiceDetailsDrawer.vue'

const toast = useToast()
const k8sStore = useKubernetesStore()

const tableColumns = ref<TableColumn[]>([
  { field: 'namespace', header: 'Namespace', visible: true },
  { field: 'type', header: 'Type', visible: true },
  { field: 'clusterIP', header: 'Cluster IP', visible: true },
  { field: 'externalIP', header: 'External IP', visible: true },
  { field: 'ports', header: 'Ports', visible: true },
  { field: 'endpoints', header: 'Endpoints', visible: true },
  { field: 'age', header: 'Age', visible: true }
])

const visibleCols = computed(() => {
  return tableColumns.value.reduce(
    (acc, col) => {
      acc[col.field] = col.visible
      return acc
    },
    {} as Record<string, boolean>
  )
})

const services = computed(() => k8sStore.services)
const searchQuery = ref('')
const selectedNamespace = ref('All Namespaces')
const selectedType = ref('All Types')
const showSystemNamespaces = ref(false)

// Drawer state
const drawerVisible = ref(false)
const selectedService = ref<ServiceInfo | null>(null)

const handleRefresh = async () => {
  await kubernetesService.getServices()
}

const types = ['All Types', 'ClusterIP', 'NodePort', 'LoadBalancer', 'ExternalName']

const filteredServices = computed(() => {
  return services.value.filter((s) => {
    // Search Query filter
    if (searchQuery.value) {
      const query = searchQuery.value.toLowerCase()
      if (!s.name.toLowerCase().includes(query)) return false
    }

    // Namespace filter
    if (selectedNamespace.value !== 'All Namespaces' && s.namespace !== selectedNamespace.value) {
      return false
    }

    // System Namespaces filter
    const isSystem = ['kube-system', 'monitoring', 'logging'].includes(s.namespace)
    if (!showSystemNamespaces.value && isSystem && selectedNamespace.value === 'All Namespaces') {
      return false
    }

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

const handleActionClick = (event: Event, action: string, serviceName: string) => {
  event.stopPropagation()
  toast.add({
    severity: 'info',
    summary: action,
    detail: `Action triggered for service: ${serviceName}`,
    life: 3000
  })
}
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
      <Select
        v-model="selectedNamespace"
        :options="k8sStore.namespaces"
        class="text-xs min-w-44 bg-(--bg-hover)/30 border-(--border)"
      />

      <!-- Type Select -->
      <Select
        v-model="selectedType"
        :options="types"
        class="text-xs min-w-40 bg-(--bg-hover)/30 border-(--border)"
      />
    </template>

    <!-- Actions Left -->
    <template #actions-left>
      <div class="flex items-center gap-2">
        <ToggleSwitch v-model="showSystemNamespaces" inputId="system-ns-toggle" />
        <label
          for="system-ns-toggle"
          class="text-xs font-semibold text-(--text-secondary) cursor-pointer select-none"
        >
          Show system namespaces
        </label>
      </div>
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
        <span class="font-mono text-(--text-muted)">{{ data.namespace }}</span>
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
    <Column class="p-3 text-center">
      <template #body="{ data }">
        <Button
          severity="secondary"
          variant="text"
          size="small"
          class="p-1"
          title="Actions"
          @click="handleActionClick($event, 'Show Actions Menu', data.name)"
        >
          <MoreVertical class="w-4 h-4 text-(--text-muted)" />
        </Button>
      </template>
    </Column>

    <!-- Drawer -->
    <template #drawer>
      <ServiceDetailsDrawer v-model:visible="drawerVisible" :service="selectedService" />
    </template>
  </ResourceDataTable>
</template>
