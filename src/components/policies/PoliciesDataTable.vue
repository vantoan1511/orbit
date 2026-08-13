<script setup lang="ts">
import { computed, ref } from 'vue'
import Column from 'primevue/column'
import Select from 'primevue/select'
import Button from 'primevue/button'
import { MoreVertical } from '@lucide/vue'
import type { PolicyInfo } from '@/types/kubernetes'
import { useKubernetesStore } from '@/stores/kubernetesStore'
import PolicyDetailsDrawer from './PolicyDetailsDrawer.vue'
import ResourceActionMenu from '@/components/shared/ResourceActionMenu.vue'
import ResourceDataTable from '@/components/shared/ResourceDataTable.vue'
import NamespaceFilter from '@/components/shared/NamespaceFilter.vue'
import { useWorkloadActions } from '@/composables/useWorkloadActions'
import { useResourceFilters } from '@/composables/useResourceFilters'
import { useTableColumns } from '@/composables/useTableColumns'

const k8sStore = useKubernetesStore()

const { searchQuery, selectedNamespace, filteredResources } = useResourceFilters(
  computed(() => k8sStore.policies),
  ['name', 'description', 'type']
)

const { tableColumns, visibleCols } = useTableColumns([
  { field: 'name', header: 'Name', visible: true },
  { field: 'type', header: 'Type', visible: true },
  { field: 'scope', header: 'Scope', visible: true },
  { field: 'namespace', header: 'Namespace', visible: true },
  { field: 'status', header: 'Status', visible: true },
  { field: 'mode', header: 'Mode', visible: true },
  { field: 'violations', header: 'Violations', visible: true },
  { field: 'lastUpdated', header: 'Last Updated', visible: true }
])

const selectedType = ref('All Types')
const drawerVisible = ref(false)
const selectedPolicy = ref<PolicyInfo | null>(null)

const namespaces = computed(() => k8sStore.namespaces)

const types = [
  'All Types',
  'Network Policy',
  'Pod Security',
  'Resource Quota',
  'RBAC',
  'Admission Policy'
]

const filteredPolicies = computed(() => {
  return filteredResources.value.filter((p) => {
    if (selectedType.value !== 'All Types' && p.type !== selectedType.value) {
      return false
    }
    return true
  })
})

const handleRefresh = async () => {
  try {
    const ns = selectedNamespace.value.length === 1 ? selectedNamespace.value[0] : undefined
    await k8sStore.fetchPolicies(ns)
  } catch (error) {
    console.error('Error fetching policies:', error)
  }
}

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

const actionMenu = ref<InstanceType<typeof ResourceActionMenu> | null>(null)
const selectedActionRow = ref<PolicyInfo | null>(null)

const toggleActionMenu = (event: Event, data: PolicyInfo) => {
  event.stopPropagation()
  selectedActionRow.value = data
  actionMenu.value?.toggle(event)
}

const onRowContextMenu = (event: { originalEvent: Event; data: PolicyInfo }) => {
  event.originalEvent?.stopPropagation()
  event.originalEvent?.preventDefault()
  selectedActionRow.value = event.data
  actionMenu.value?.show(event.originalEvent)
}

const { actionMenuItems } = useWorkloadActions(selectedActionRow, {
  kind: 'Policy',
  onViewDetails: (row) => {
    selectedPolicy.value = row
    drawerVisible.value = true
  }
})
</script>

<template>
  <ResourceDataTable
    :data="filteredPolicies"
    v-model:searchQuery="searchQuery"
    v-model:columns="tableColumns"
    searchPlaceholder="Search policies..."
    emptyMessage="No policies found matching the filter criteria."
    reportTemplate="Showing {first} to {last} of {totalRecords} policies"
    :loading="k8sStore.policiesLoading"
    @refresh="handleRefresh"
    @row-click="onRowClick"
    @row-contextmenu="onRowContextMenu"
  >
    <!-- Filters -->
    <template #filters>
      <NamespaceFilter v-model="selectedNamespace" :namespaces="namespaces" />
      <Select v-model="selectedType" :options="types" class="text-xs min-w-40" />
    </template>

    <!-- Name Column -->
    <Column
      v-if="visibleCols['name']"
      field="name"
      header="Name"
      sortable
      class="p-3 min-w-48"
      bodyClass="font-semibold text-primary"
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
    <Column v-if="visibleCols['type']" field="type" header="Type" sortable class="p-3">
      <template #body="{ data }">
        <span class="text-muted-color">{{ data.type }}</span>
      </template>
    </Column>

    <!-- Scope Column -->
    <Column v-if="visibleCols['scope']" field="scope" header="Scope" sortable class="p-3">
      <template #body="{ data }">
        <span class="text-[10px] text-muted-color uppercase font-semibold">{{ data.scope }}</span>
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
        <span v-if="data.namespace !== '-'" class="font-mono text-muted-color">{{
          data.namespace
        }}</span>
        <span v-else class="text-muted-color">-</span>
      </template>
    </Column>

    <!-- Status Column -->
    <Column
      v-if="visibleCols['status']"
      field="status"
      header="Status"
      sortable
      class="p-3 min-w-24"
    >
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
    <Column v-if="visibleCols['mode']" field="mode" header="Mode" sortable class="p-3">
      <template #body="{ data }">
        <span class="font-mono text-muted-color">{{ data.mode }}</span>
      </template>
    </Column>

    <!-- Violations Column -->
    <Column
      v-if="visibleCols['violations']"
      field="violations"
      header="Violations (7d)"
      sortable
      class="p-3 text-center"
    >
      <template #body="{ data }">
        <span
          class="font-mono font-bold"
          :class="data.violations > 0 ? 'text-red-400' : 'text-primary'"
        >
          {{ data.violations }}
        </span>
      </template>
    </Column>

    <!-- Last Updated Column -->
    <Column
      v-if="visibleCols['lastUpdated']"
      field="lastUpdated"
      header="Last Updated"
      sortable
      class="p-3"
      bodyClass="text-muted-color font-mono"
    >
      <template #body="{ data }">
        <span>{{ data.lastUpdated }}</span>
      </template>
    </Column>

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
      <PolicyDetailsDrawer v-model:visible="drawerVisible" :policy="selectedPolicy" />
      <ResourceActionMenu ref="actionMenu" :items="actionMenuItems" />
    </template>
  </ResourceDataTable>
</template>
