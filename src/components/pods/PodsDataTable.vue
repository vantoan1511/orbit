<script setup lang="ts">
import { ref, computed } from 'vue'
import DataTable from 'primevue/datatable'
import Column from 'primevue/column'
import Select from 'primevue/select'
import ToggleSwitch from 'primevue/toggleswitch'
import InputText from 'primevue/inputtext'
import Button from 'primevue/button'
import { Search, Info, RefreshCw, Settings2, Power, Trash2 } from '@lucide/vue'
import { mockPods } from './mockPods'
import type { PodInfo } from './mockPods'
import PodDetailsDrawer from './PodDetailsDrawer.vue'

const pods = ref<PodInfo[]>(mockPods)
const searchQuery = ref('')
const selectedNamespace = ref('All Namespaces')
const selectedStatus = ref('All Statuses')
const showSystemNamespaces = ref(false)

// Drawer state
const drawerVisible = ref(false)
const selectedPod = ref<PodInfo | null>(null)

const namespaces = computed(() => {
  const list = new Set(pods.value.map((p) => p.namespace))
  return ['All Namespaces', ...Array.from(list)]
})

const statuses = [
  'All Statuses',
  'Running',
  'Pending',
  'Failed',
  'CrashLoopBackOff',
  'Completed',
  'Unknown'
]

const filteredPods = computed(() => {
  return pods.value.filter((p) => {
    // Search Query filter
    if (searchQuery.value) {
      const query = searchQuery.value.toLowerCase()
      const matchesName = p.name.toLowerCase().includes(query)
      const matchesImage = p.images.some((img) => img.toLowerCase().includes(query))
      const matchesNode = p.node.toLowerCase().includes(query)
      if (!matchesName && !matchesImage && !matchesNode) return false
    }

    // Namespace filter
    if (selectedNamespace.value !== 'All Namespaces' && p.namespace !== selectedNamespace.value) {
      return false
    }

    // System Namespaces filter
    const isSystem = ['kube-system', 'monitoring', 'logging'].includes(p.namespace)
    if (!showSystemNamespaces.value && isSystem && selectedNamespace.value === 'All Namespaces') {
      return false
    }

    // Status filter
    if (selectedStatus.value !== 'All Statuses') {
      if (selectedStatus.value === 'Failed' && p.status === 'CrashLoopBackOff') {
        // include CrashLoopBackOff under Failed or check exact match
      }
      if (p.status !== selectedStatus.value) {
        return false
      }
    }

    return true
  })
})

const onRowClick = (event: { data: PodInfo }) => {
  selectedPod.value = event.data
  drawerVisible.value = true
}

const getStatusColor = (status: string) => {
  switch (status) {
    case 'Running':
      return 'bg-emerald-500 text-emerald-500'
    case 'Pending':
      return 'bg-amber-500 text-amber-500'
    case 'Failed':
    case 'CrashLoopBackOff':
      return 'bg-rose-500 text-rose-500'
    case 'Completed':
      return 'bg-blue-500 text-blue-500'
    default:
      return 'bg-gray-400 text-gray-400'
  }
}

const handleActionClick = (event: Event, action: string, podName: string) => {
  event.stopPropagation()
  alert(`${action} triggered for pod: ${podName}`)
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
            placeholder="Search pods, images or nodes..."
            class="pl-9 pr-4 py-2 w-full text-xs bg-(--bg-hover)/30 border-(--border) text-(--text-primary) rounded-lg focus:border-violet-500 focus:ring-1 focus:ring-violet-500"
          />
        </div>

        <!-- Namespace Select -->
        <Select
          v-model="selectedNamespace"
          :options="namespaces"
          class="text-xs min-w-44 bg-(--bg-hover)/30 border-(--border)"
        />

        <!-- Status Select -->
        <Select
          v-model="selectedStatus"
          :options="statuses"
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
            Show System
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
      :value="filteredPods"
      paginator
      :rows="10"
      class="p-datatable-sm border border-(--border) rounded-lg overflow-hidden cursor-pointer"
      tableClass="w-full text-left text-xs border-collapse"
      paginatorTemplate="FirstPageLink PrevPageLink PageLinks NextPageLink LastPageLink RowsPerPageDropdown CurrentPageReport"
      currentPageReportTemplate="Showing {first} to {last} of {totalRecords} pods"
      @row-click="onRowClick"
    >
      <template #empty>
        <div class="text-center py-10 text-(--text-muted) flex flex-col items-center gap-2">
          <Info class="w-8 h-8 text-(--text-muted)/50" />
          <span>No pods found matching the filter criteria.</span>
        </div>
      </template>

      <!-- Name Column -->
      <Column field="name" header="Name" sortable class="font-medium p-3 text-(--text-primary)">
        <template #body="{ data }">
          <span class="font-semibold hover:text-violet-400 transition-colors">{{ data.name }}</span>
        </template>
      </Column>

      <!-- Namespace Column -->
      <Column field="namespace" header="Namespace" sortable class="p-3">
        <template #body="{ data }">
          <span class="font-mono text-(--text-muted)">{{ data.namespace }}</span>
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

      <!-- Node Column -->
      <Column field="node" header="Node" sortable class="p-3">
        <template #body="{ data }">
          <span
            class="text-(--text-secondary) font-mono truncate block max-w-44"
            :title="data.node"
          >
            {{ data.node.split('.')[0] }}
          </span>
        </template>
      </Column>

      <!-- Restarts Column -->
      <Column field="restarts" header="Restarts" sortable class="p-3 text-center">
        <template #body="{ data }">
          <span
            class="font-mono px-1.5 py-0.5 rounded text-[10px]"
            :class="
              data.restarts > 0
                ? 'bg-rose-500/10 text-rose-400 border border-rose-500/20'
                : 'text-(--text-muted)'
            "
          >
            {{ data.restarts }}
          </span>
        </template>
      </Column>

      <!-- CPU Column -->
      <Column field="cpu" header="CPU" sortable class="p-3">
        <template #body="{ data }">
          <div class="flex flex-col gap-1 w-24">
            <div class="flex justify-between font-mono text-[10px]">
              <span class="text-(--text-secondary)">{{ data.cpu }}</span>
              <span class="text-(--text-muted)" v-if="data.cpu !== '-'">{{ data.cpuPct }}%</span>
            </div>
            <div
              class="w-full h-1 bg-(--bg-hover) rounded-full overflow-hidden"
              v-if="data.cpu !== '-'"
            >
              <div
                class="h-full rounded-full bg-violet-500"
                :style="{ width: data.cpuPct + '%' }"
              ></div>
            </div>
          </div>
        </template>
      </Column>

      <!-- Memory Column -->
      <Column field="memory" header="Memory" sortable class="p-3">
        <template #body="{ data }">
          <div class="flex flex-col gap-1 w-24">
            <div class="flex justify-between font-mono text-[10px]">
              <span class="text-(--text-secondary)">{{ data.memory }}</span>
              <span class="text-(--text-muted)" v-if="data.memory !== '-'"
                >{{ data.memoryPct }}%</span
              >
            </div>
            <div
              class="w-full h-1 bg-(--bg-hover) rounded-full overflow-hidden"
              v-if="data.memory !== '-'"
            >
              <div
                class="h-full rounded-full bg-blue-500"
                :style="{ width: data.memoryPct + '%' }"
              ></div>
            </div>
          </div>
        </template>
      </Column>

      <!-- Age Column -->
      <Column field="age" header="Age" sortable class="p-3 text-(--text-muted) font-mono"></Column>

      <!-- Actions Column -->
      <Column class="p-3 text-center">
        <template #body="{ data }">
          <div class="flex items-center justify-center gap-1">
            <Button
              severity="secondary"
              variant="text"
              size="small"
              class="p-1"
              title="Terminate Pod"
              @click="handleActionClick($event, 'Terminate', data.name)"
            >
              <Trash2 class="w-3.5 h-3.5 text-rose-400" />
            </Button>
            <Button
              severity="secondary"
              variant="text"
              size="small"
              class="p-1"
              title="Restart Pod"
              @click="handleActionClick($event, 'Restart', data.name)"
            >
              <Power class="w-3.5 h-3.5 text-amber-400" />
            </Button>
          </div>
        </template>
      </Column>
    </DataTable>

    <!-- Slideout details drawer -->
    <PodDetailsDrawer v-model:visible="drawerVisible" :pod="selectedPod" />
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
  font-size: 0.75rem;
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
