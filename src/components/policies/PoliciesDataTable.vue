<script setup lang="ts">
import { ref, computed } from 'vue'
import DataTable from 'primevue/datatable'
import Column from 'primevue/column'
import Select from 'primevue/select'
import InputText from 'primevue/inputtext'
import Button from 'primevue/button'
import { Search, Info, RefreshCw, Settings2, MoreVertical } from '@lucide/vue'
import type { PolicyInfo } from '@/types/kubernetes'
import { useKubernetesStore } from '@/stores/kubernetesStore'
import PolicyDetailsDrawer from './PolicyDetailsDrawer.vue'
import { useToast } from 'primevue/usetoast'

const toast = useToast()
const k8sStore = useKubernetesStore()

const policies = computed(() => k8sStore.policies)
const searchQuery = ref('')
const selectedNamespace = ref<string[]>([])
const selectedType = ref('All Types')

// Drawer state
const drawerVisible = ref(false)
const selectedPolicy = ref<PolicyInfo | null>(null)

const namespaces = computed(() => {
  const list = new Set(policies.value.filter((p) => p.namespace !== '-').map((p) => p.namespace))
  return ['All Namespaces', ...Array.from(list)]
})

const types = [
  'All Types',
  'Network Policy',
  'Pod Security',
  'Resource Quota',
  'RBAC',
  'Admission Policy'
]

const filteredPolicies = computed(() => {
  return policies.value.filter((p) => {
    // Search Query filter
    if (searchQuery.value) {
      const query = searchQuery.value.toLowerCase()
      const matchesSearch =
        p.name.toLowerCase().includes(query) ||
        p.description.toLowerCase().includes(query) ||
        p.type.toLowerCase().includes(query)

      if (!matchesSearch) return false
    }

    // Namespace filter (only applies to namespaced policies)
    if (selectedNamespace.value.length > 0) {
      if (p.scope === 'Namespaced' && !selectedNamespace.value.includes(p.namespace)) {
        return false
      }
    }

    // Type filter
    if (selectedType.value !== 'All Types' && p.type !== selectedType.value) {
      return false
    }

    return true
  })
})

const onRowClick = (event: { data: PolicyInfo }) => {
  selectedPolicy.value = event.data
  drawerVisible.value = true
}

const getStatusBadgeClass = (status: string) => {
  switch (status) {
    case 'Audit':
      return 'bg-blue-500/10 text-blue-400 border border-blue-500/20'
    case 'Enforced':
      return 'bg-emerald-500/10 text-emerald-400 border border-emerald-500/20'
    case 'Disabled':
      return 'bg-gray-500/10 text-gray-400 border border-gray-500/20'
    default:
      return 'bg-gray-500/10 text-gray-400 border border-gray-500/20'
  }
}

const handleActionClick = (event: Event, action: string, policyName: string) => {
  event.stopPropagation()
  toast.add({
    severity: 'info',
    summary: action,
    detail: `Action triggered for policy: ${policyName}`,
    life: 3000
  })
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
            placeholder="Search policies..."
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
        <div class="flex items-center gap-1">
          <Button
            severity="secondary"
            variant="text"
            size="small"
            class="p-1"
            @click="
              k8sStore.fetchPolicies(
                selectedNamespace.length === 1 ? selectedNamespace[0] : undefined
              )
            "
          >
            <RefreshCw
              class="w-4 h-4 text-(--text-secondary)"
              :class="{ 'animate-spin text-violet-400': k8sStore.policiesLoading }"
            />
          </Button>
          <Button severity="secondary" variant="text" size="small" class="p-1">
            <Settings2 class="w-4 h-4 text-(--text-secondary)" />
          </Button>
        </div>
      </div>
    </div>

    <!-- Data Table -->
    <DataTable
      :value="filteredPolicies"
      :loading="k8sStore.policiesLoading"
      paginator
      :rows="12"
      class="p-datatable-sm border border-(--border) rounded-lg overflow-hidden cursor-pointer"
      tableClass="w-full text-left text-xs border-collapse"
      paginatorTemplate="FirstPageLink PrevPageLink PageLinks NextPageLink LastPageLink RowsPerPageDropdown CurrentPageReport"
      currentPageReportTemplate="Showing {first} to {last} of {totalRecords} policies"
      @row-click="onRowClick"
    >
      <template #empty>
        <div class="text-center py-10 text-(--text-muted) flex flex-col items-center gap-2">
          <Info class="w-8 h-8 text-(--text-muted)/50" />
          <span>No policies found matching the filter criteria.</span>
        </div>
      </template>

      <!-- Name Column -->
      <Column
        field="name"
        header="Name"
        sortable
        class="p-3 font-semibold text-(--text-primary) min-w-48"
      >
        <template #body="{ data }">
          <div class="flex flex-col">
            <span
              class="font-mono text-violet-400 hover:text-violet-300 transition-colors truncate"
              :title="data.name"
            >
              {{ data.name }}
            </span>
          </div>
        </template>
      </Column>

      <!-- Type Column -->
      <Column field="type" header="Type" sortable class="p-3">
        <template #body="{ data }">
          <span class="text-(--text-secondary)">{{ data.type }}</span>
        </template>
      </Column>

      <!-- Scope Column -->
      <Column field="scope" header="Scope" sortable class="p-3">
        <template #body="{ data }">
          <span class="text-[10px] text-(--text-muted) uppercase font-semibold">{{
            data.scope
          }}</span>
        </template>
      </Column>

      <!-- Namespace Column -->
      <Column field="namespace" header="Namespace" sortable class="p-3">
        <template #body="{ data }">
          <span v-if="data.namespace !== '-'" class="font-mono text-(--text-muted)">{{
            data.namespace
          }}</span>
          <span v-else class="text-(--text-muted)">-</span>
        </template>
      </Column>

      <!-- Status Column -->
      <Column field="status" header="Status" sortable class="p-3 min-w-24">
        <template #body="{ data }">
          <span
            class="px-2 py-0.5 rounded text-[10px] font-semibold tracking-wider font-ui border"
            :class="getStatusBadgeClass(data.status)"
          >
            {{ data.status }}
          </span>
        </template>
      </Column>

      <!-- Mode Column -->
      <Column field="mode" header="Mode" sortable class="p-3">
        <template #body="{ data }">
          <span class="font-mono text-(--text-secondary)">{{ data.mode }}</span>
        </template>
      </Column>

      <!-- Violations Column -->
      <Column field="violations" header="Violations (7d)" sortable class="p-3 text-center">
        <template #body="{ data }">
          <span
            class="font-mono font-bold"
            :class="data.violations > 0 ? 'text-red-400' : 'text-(--text-primary)'"
          >
            {{ data.violations }}
          </span>
        </template>
      </Column>

      <!-- Last Updated Column -->
      <Column
        field="lastUpdated"
        header="Last Updated"
        sortable
        class="p-3 text-(--text-muted) font-mono"
      >
        <template #body="{ data }">
          <span>{{ data.lastUpdated }}</span>
        </template>
      </Column>

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
    <PolicyDetailsDrawer v-model:visible="drawerVisible" :policy="selectedPolicy" />
  </div>
</template>
