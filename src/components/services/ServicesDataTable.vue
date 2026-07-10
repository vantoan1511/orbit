<script setup lang="ts">
import { ref, computed } from 'vue'
import DataTable from 'primevue/datatable'
import Column from 'primevue/column'
import Select from 'primevue/select'
import ToggleSwitch from 'primevue/toggleswitch'
import InputText from 'primevue/inputtext'
import Button from 'primevue/button'
import { Search, Info, RefreshCw, Settings2, ExternalLink, MoreVertical } from '@lucide/vue'
import { mockServices } from './mockServices'
import type { ServiceInfo } from './mockServices'
import ServiceDetailsDrawer from './ServiceDetailsDrawer.vue'

const services = ref<ServiceInfo[]>(mockServices)
const searchQuery = ref('')
const selectedNamespace = ref('All Namespaces')
const selectedType = ref('All Types')
const showSystemNamespaces = ref(false)

// Drawer state
const drawerVisible = ref(false)
const selectedService = ref<ServiceInfo | null>(null)

const namespaces = computed(() => {
  const list = new Set(services.value.map((s) => s.namespace))
  return ['All Namespaces', ...Array.from(list)]
})

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
  alert(`${action} triggered for service: ${serviceName}`)
}
</script>

<template>
  <div class="bg-(--bg-card) border border-(--border) rounded-xl p-6 shadow-sm flex flex-col gap-6">
    <!-- Filter Toolbar -->
    <div class="flex flex-col md:flex-row md:items-center justify-between gap-4">
      <div class="flex items-center gap-3 flex-wrap">
        <!-- Search -->
        <div class="relative min-w-64">
          <Search class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-(--text-muted)" />
          <InputText
            v-model="searchQuery"
            placeholder="Search services..."
            class="pl-9 pr-4 py-2 w-full text-xs bg-(--bg-hover)/30 border-(--border) text-(--text-primary) rounded-lg focus:border-violet-500 focus:ring-1 focus:ring-violet-500"
          />
        </div>

        <!-- Namespace Select -->
        <Select
          v-model="selectedNamespace"
          :options="namespaces"
          class="text-xs min-w-44 bg-(--bg-hover)/30 border-(--border)"
        />

        <!-- Type Select -->
        <Select
          v-model="selectedType"
          :options="types"
          class="text-xs min-w-40 bg-(--bg-hover)/30 border-(--border)"
        />
      </div>

      <!-- Toggles and Actions -->
      <div class="flex items-center gap-4 self-end md:self-auto">
        <div class="flex items-center gap-2">
          <ToggleSwitch v-model="showSystemNamespaces" inputId="system-ns-toggle" />
          <label
            for="system-ns-toggle"
            class="text-xs font-semibold text-(--text-secondary) cursor-pointer select-none"
          >
            Show system namespaces
          </label>
        </div>

        <div class="flex items-center gap-1">
          <Button severity="secondary" variant="text" size="small" class="p-1">
            <RefreshCw class="w-4 h-4 text-(--text-secondary)" />
          </Button>
          <Button severity="secondary" variant="text" size="small" class="p-1">
            <Settings2 class="w-4 h-4 text-(--text-secondary)" />
          </Button>
        </div>
      </div>
    </div>

    <!-- Data Table -->
    <DataTable
      :value="filteredServices"
      paginator
      :rows="12"
      class="p-datatable-sm border border-(--border) rounded-lg overflow-hidden cursor-pointer"
      tableClass="w-full text-left text-xs border-collapse"
      paginatorTemplate="FirstPageLink PrevPageLink PageLinks NextPageLink LastPageLink RowsPerPageDropdown CurrentPageReport"
      currentPageReportTemplate="Showing {first} to {last} of {totalRecords} services"
      @row-click="onRowClick"
    >
      <template #empty>
        <div class="text-center py-10 text-(--text-muted) flex flex-col items-center gap-2">
          <Info class="w-8 h-8 text-(--text-muted)/50" />
          <span>No services found matching the filter criteria.</span>
        </div>
      </template>

      <!-- Name Column -->
      <Column field="name" header="Name" sortable class="font-medium p-3 text-(--text-primary)">
        <template #body="{ data }">
          <div class="flex items-center gap-2">
            <span class="w-2 h-2 rounded-full bg-emerald-500 flex-shrink-0"></span>
            <span class="font-semibold text-violet-400 hover:text-violet-300 transition-colors">{{
              data.name
            }}</span>
          </div>
        </template>
      </Column>

      <!-- Namespace Column -->
      <Column field="namespace" header="Namespace" sortable class="p-3">
        <template #body="{ data }">
          <span class="font-mono text-(--text-muted)">{{ data.namespace }}</span>
        </template>
      </Column>

      <!-- Type Column -->
      <Column field="type" header="Type" sortable class="p-3">
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
      <Column field="clusterIP" header="Cluster IP" sortable class="p-3">
        <template #body="{ data }">
          <span class="font-mono text-(--text-secondary)">{{ data.clusterIP }}</span>
        </template>
      </Column>

      <!-- External IP Column -->
      <Column field="externalIP" header="External IP" sortable class="p-3">
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
      <Column field="ports" header="Ports" sortable class="p-3">
        <template #body="{ data }">
          <span class="font-mono text-(--text-muted) whitespace-pre-line">{{ data.ports }}</span>
        </template>
      </Column>

      <!-- Endpoints Column -->
      <Column field="endpoints" header="Endpoints" sortable class="p-3">
        <template #body="{ data }">
          <span class="font-mono font-medium text-emerald-400">{{ data.endpoints }}</span>
        </template>
      </Column>

      <!-- Age Column -->
      <Column field="age" header="Age" sortable class="p-3 text-(--text-muted) font-mono"></Column>

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
    </DataTable>

    <!-- Details Slideout Drawer -->
    <ServiceDetailsDrawer v-model:visible="drawerVisible" :service="selectedService" />
  </div>
</template>

<style scoped>
:deep(.p-datatable-thead > tr > th) {
  background: var(--bg-hover);
  color: var(--text-secondary);
  border-bottom: 1px solid var(--border);
  font-weight: 600;
  font-size: 0.75rem;
  text-transform: uppercase;
}

:deep(.p-datatable-tbody > tr:hover) {
  background: var(--bg-hover) / 20;
}
</style>
