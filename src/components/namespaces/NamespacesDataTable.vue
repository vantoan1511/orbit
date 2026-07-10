<script setup lang="ts">
import { Info, MoreHorizontal, RefreshCw, Search, Settings2 } from '@lucide/vue'
import Button from 'primevue/button'
import Chart from 'primevue/chart'
import Column from 'primevue/column'
import DataTable from 'primevue/datatable'
import InputText from 'primevue/inputtext'
import Select from 'primevue/select'
import ToggleSwitch from 'primevue/toggleswitch'
import { computed, onMounted, ref } from 'vue'
import type { NamespaceInfo } from './mockNamespaces'
import { mockNamespaces } from './mockNamespaces'
import NamespaceDetailsDrawer from './NamespaceDetailsDrawer.vue'

const namespaces = ref<NamespaceInfo[]>(mockNamespaces)
const searchQuery = ref('')
const selectedStatus = ref('All Statuses')
const selectedLabel = ref('All Labels')
const showSystemNamespaces = ref(false)

// Drawer state
const drawerVisible = ref(false)
const selectedNamespace = ref<NamespaceInfo | null>(null)

const statuses = ['All Statuses', 'Active', 'Terminating']

// Collect unique label keys across all namespaces
const labelOptions = computed(() => {
  const keys = new Set<string>()
  namespaces.value.forEach((ns) => {
    Object.keys(ns.labels).forEach((k) => keys.add(k))
  })
  return ['All Labels', ...Array.from(keys)]
})

const filteredNamespaces = computed(() => {
  return namespaces.value.filter((ns) => {
    // System namespace toggle
    if (!showSystemNamespaces.value && ns.isSystem) return false

    // Search filter
    if (searchQuery.value) {
      const q = searchQuery.value.toLowerCase()
      if (!ns.name.toLowerCase().includes(q)) return false
    }

    // Status filter
    if (selectedStatus.value !== 'All Statuses' && ns.status !== selectedStatus.value) {
      return false
    }

    // Label filter
    if (selectedLabel.value !== 'All Labels') {
      if (!(selectedLabel.value in ns.labels)) return false
    }

    return true
  })
})

const onRowClick = (event: { data: NamespaceInfo }) => {
  selectedNamespace.value = event.data
  drawerVisible.value = true
}

const getStatusColor = (status: string) => {
  switch (status) {
    case 'Active':
      return 'bg-emerald-500 text-emerald-500'
    case 'Terminating':
      return 'bg-amber-500 text-amber-500'
    default:
      return 'bg-gray-400 text-gray-400'
  }
}

// Sparkline chart options - minimal, no axes, no tooltips
const sparklineOptions = ref()

onMounted(() => {
  sparklineOptions.value = {
    responsive: true,
    maintainAspectRatio: false,
    plugins: {
      legend: { display: false },
      tooltip: { enabled: false }
    },
    scales: {
      x: { display: false },
      y: { display: false }
    },
    elements: {
      point: { radius: 0, hoverRadius: 0 },
      line: { tension: 0.4, borderWidth: 1.5 }
    },
    events: []
  }
})

const getSparklineData = (ns: NamespaceInfo) => ({
  labels: ['1', '2', '3', '4', '5', '6', '7'],
  datasets: [
    {
      data: ns.podSparkline,
      borderColor: ns.status === 'Terminating' ? '#f59e0b' : '#10b981',
      backgroundColor: 'transparent',
      fill: false
    }
  ]
})

// Max visible labels before showing "+N" overflow
const MAX_VISIBLE_LABELS = 2
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
            placeholder="Search namespaces..."
            class="pl-9 pr-4 py-2 w-full text-xs bg-(--bg-hover)/30 border-(--border) text-(--text-primary) rounded-lg focus:border-violet-500 focus:ring-1 focus:ring-violet-500"
          />
        </div>

        <!-- Status Select -->
        <Select
          v-model="selectedStatus"
          :options="statuses"
          class="text-xs min-w-40 bg-(--bg-hover)/30 border-(--border)"
        />

        <!-- Labels Select -->
        <Select
          v-model="selectedLabel"
          :options="labelOptions"
          class="text-xs min-w-40 bg-(--bg-hover)/30 border-(--border)"
        />
      </div>

      <!-- Right controls -->
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
      :value="filteredNamespaces"
      paginator
      :rows="20"
      class="p-datatable-sm border border-(--border) rounded-lg overflow-hidden cursor-pointer"
      tableClass="w-full text-left text-xs border-collapse"
      paginatorTemplate="FirstPageLink PrevPageLink PageLinks NextPageLink LastPageLink RowsPerPageDropdown CurrentPageReport"
      currentPageReportTemplate="Showing {first} to {last} of {totalRecords} namespaces"
      @row-click="onRowClick"
    >
      <template #empty>
        <div class="text-center py-10 text-(--text-muted) flex flex-col items-center gap-2">
          <Info class="w-8 h-8 text-(--text-muted)/50" />
          <span>No namespaces found matching the filter criteria.</span>
        </div>
      </template>

      <!-- Name Column -->
      <Column field="name" header="Name" sortable class="font-medium p-3 text-(--text-primary)">
        <template #body="{ data }">
          <div class="flex items-center gap-2">
            <span class="font-semibold hover:text-violet-400 transition-colors">{{
              data.name
            }}</span>
            <span
              v-if="data.isSystem"
              class="px-1.5 py-0.5 rounded text-[9px] font-bold uppercase tracking-wider bg-blue-500/10 text-blue-400 border border-blue-500/20"
            >
              System
            </span>
          </div>
        </template>
      </Column>

      <!-- Status Column -->
      <Column field="status" header="Status" sortable class="p-3">
        <template #body="{ data }">
          <div class="flex items-center gap-1.5">
            <span
              class="w-1.5 h-1.5 rounded-full"
              :class="getStatusColor(data.status).split(' ')[0]"
            ></span>
            <span class="font-medium" :class="getStatusColor(data.status).split(' ')[1]">
              {{ data.status }}
            </span>
          </div>
        </template>
      </Column>

      <!-- Pods Column with sparkline -->
      <Column field="pods" header="Pods" sortable class="p-3">
        <template #body="{ data }">
          <div class="flex items-center gap-3">
            <span class="font-mono text-(--text-primary) min-w-6">{{ data.pods }}</span>
            <div class="w-16 h-6 shrink-0" v-if="sparklineOptions">
              <Chart
                type="line"
                :data="getSparklineData(data)"
                :options="sparklineOptions"
                class="w-full h-full"
              />
            </div>
          </div>
        </template>
      </Column>

      <!-- Workloads Column -->
      <Column field="workloads" header="Workloads" sortable class="p-3 text-(--text-secondary)">
        <template #body="{ data }">
          <span class="font-mono">{{ data.workloads }}</span>
        </template>
      </Column>

      <!-- Services Column -->
      <Column field="services" header="Services" sortable class="p-3 text-(--text-secondary)">
        <template #body="{ data }">
          <span class="font-mono">{{ data.services }}</span>
        </template>
      </Column>

      <!-- ConfigMaps Column -->
      <Column field="configMaps" header="ConfigMaps" sortable class="p-3 text-(--text-secondary)">
        <template #body="{ data }">
          <span class="font-mono">{{ data.configMaps }}</span>
        </template>
      </Column>

      <!-- Secrets Column -->
      <Column field="secrets" header="Secrets" sortable class="p-3 text-(--text-secondary)">
        <template #body="{ data }">
          <span class="font-mono">{{ data.secrets }}</span>
        </template>
      </Column>

      <!-- Age Column -->
      <Column field="age" header="Age" sortable class="p-3 text-(--text-muted) font-mono"> </Column>

      <!-- Labels Column -->
      <Column field="labels" header="Labels" class="p-3">
        <template #body="{ data }">
          <div class="flex items-center gap-1 flex-wrap">
            <span
              v-for="(val, key) in Object.fromEntries(
                Object.entries(data.labels).slice(0, MAX_VISIBLE_LABELS)
              )"
              :key="key"
              class="px-1.5 py-0.5 rounded text-[9px] font-mono bg-(--bg-hover) text-(--text-secondary) border border-(--border) whitespace-nowrap"
            >
              {{ key }}: {{ val }}
            </span>
            <span
              v-if="Object.keys(data.labels).length > MAX_VISIBLE_LABELS"
              class="px-1.5 py-0.5 rounded text-[9px] font-bold bg-violet-500/10 text-violet-400 border border-violet-500/20"
            >
              +{{ Object.keys(data.labels).length - MAX_VISIBLE_LABELS }}
            </span>
          </div>
        </template>
      </Column>

      <!-- Actions Column -->
      <Column class="p-3 text-center w-10">
        <template #body>
          <Button severity="secondary" variant="text" size="small" class="p-1">
            <MoreHorizontal class="w-3.5 h-3.5 text-(--text-muted)" />
          </Button>
        </template>
      </Column>
    </DataTable>

    <!-- Details Drawer -->
    <NamespaceDetailsDrawer v-model:visible="drawerVisible" :namespace="selectedNamespace" />
  </div>
</template>

<style scoped>
:deep(.p-datatable) {
  background: transparent;
}
:deep(.p-datatable-thead > tr > th) {
  background: var(--bg-hover);
  color: var(--text-secondary);
  border-bottom: 1px solid var(--border);
  font-weight: 600;
  font-size: 0.7rem;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}
:deep(.p-datatable-tbody > tr) {
  background: transparent;
  color: var(--text-secondary);
  border-bottom: 1px solid rgba(var(--border), 0.5);
  transition: background-color 0.2s;
}
:deep(.p-datatable-tbody > tr:hover) {
  background: var(--bg-hover) !important;
}
:deep(.p-paginator) {
  background: transparent;
  border-top: 1px solid var(--border);
  color: var(--text-secondary);
  padding: 0.75rem;
  font-size: 0.75rem;
}
:deep(.p-paginator-current) {
  color: var(--text-muted);
}
:deep(.p-paginator-page),
:deep(.p-paginator-first),
:deep(.p-paginator-prev),
:deep(.p-paginator-next),
:deep(.p-paginator-last) {
  color: var(--text-secondary);
  border-radius: 6px;
  min-width: 2rem;
  height: 2rem;
}
:deep(.p-paginator-page.p-highlight) {
  background: var(--bg-active);
  color: var(--text-primary);
  font-weight: 600;
}
:deep(.p-select) {
  border-radius: 8px;
}
</style>
