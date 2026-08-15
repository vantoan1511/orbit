<script setup lang="ts">
import GenericResourceTable from '@/components/shared/GenericResourceTable.vue'
import TableFilterSelect from '@/components/shared/TableFilterSelect.vue'
import { useKubernetesStore } from '@/stores/kubernetesStore'
import Column from 'primevue/column'
import { computed, ref } from 'vue'
import PolicyDetailsDrawer from './PolicyDetailsDrawer.vue'

const k8sStore = useKubernetesStore()

const columns = [
  { field: 'name', header: 'Name', visible: true },
  { field: 'type', header: 'Type', visible: true },
  { field: 'scope', header: 'Scope', visible: true },
  { field: 'namespace', header: 'Namespace', visible: true },
  { field: 'status', header: 'Status', visible: true },
  { field: 'mode', header: 'Mode', visible: true },
  { field: 'violations', header: 'Violations', visible: true },
  { field: 'lastUpdated', header: 'Last Updated', visible: true }
]

const selectedType = ref('All Types')
const types = [
  'All Types',
  'Network Policy',
  'Pod Security',
  'Resource Quota',
  'RBAC',
  'Admission Policy'
]

const filteredPolicies = computed(() => {
  return k8sStore.policies.filter((p) => {
    if (selectedType.value !== 'All Types' && p.type !== selectedType.value) {
      return false
    }
    return true
  })
})

const handleRefresh = async (namespace?: string) => {
  try {
    await k8sStore.fetchPolicies(namespace)
  } catch (error) {
    console.error('Error fetching policies:', error)
  }
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
</script>

<template>
  <GenericResourceTable
    :data="filteredPolicies"
    :initialColumns="columns"
    :hideStatusFilter="true"
    :hideAgeColumn="true"
    :searchFields="['name', 'description', 'type']"
    kind="Policy"
    searchPlaceholder="Search policies..."
    emptyMessage="No policies found matching the filter criteria."
    reportTemplate="Showing {first} to {last} of {totalRecords} policies"
    :loading="k8sStore.policiesLoading"
    @refresh="handleRefresh"
    @namespace-change="handleRefresh"
  >
    <!-- Custom Filter -->
    <template #filters>
      <TableFilterSelect v-model="selectedType" :options="types" />
    </template>

    <!-- Custom Name -->
    <template #name="{ data }">
      <span
        class="font-mono text-violet-400 hover:text-violet-300 transition-colors truncate"
        :title="data.name"
      >
        {{ data.name }}
      </span>
    </template>

    <!-- Custom Status -->
    <template #status="{ data }">
      <span
        class="px-2 py-0.5 rounded text-[10px] font-semibold tracking-wider font-ui border"
        :class="getStatusBadgeClass(data.status)"
      >
        {{ data.status }}
      </span>
    </template>

    <template #default="{ visibleCols }">
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
    </template>

    <!-- Drawer -->
    <template #drawer="{ selectedItem, visible, close }">
      <PolicyDetailsDrawer
        :visible="visible"
        :policy="selectedItem"
        @update:visible="!$event && close()"
      />
    </template>
  </GenericResourceTable>
</template>
