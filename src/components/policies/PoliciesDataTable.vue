<script setup lang="ts">
import GenericResourceTable from '@/components/shared/GenericResourceTable.vue'
import TableFilterSelect from '@/components/shared/TableFilterSelect.vue'
import { useKubernetesStore } from '@/stores/kubernetesStore'
import Column from 'primevue/column'
import Tag from 'primevue/tag'
import { computed, ref, watch } from 'vue'
import PolicyDetailsDrawer from './PolicyDetailsDrawer.vue'
import {
  filterPolicies,
  getFilterOptionsForCategory,
  type PolicyCategoryFilter
} from './policyFilter'

const props = withDefaults(
  defineProps<{
    categoryFilter?: PolicyCategoryFilter
  }>(),
  {
    categoryFilter: 'overview'
  }
)

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

const filterOptions = computed(() => getFilterOptionsForCategory(props.categoryFilter))
const selectedType = ref(filterOptions.value[0] || '')

watch(
  () => props.categoryFilter,
  (newCat) => {
    const opts = getFilterOptionsForCategory(newCat)
    selectedType.value = opts[0] || ''
  }
)

const filteredPolicies = computed(() => {
  return filterPolicies(k8sStore.policies, props.categoryFilter, selectedType.value)
})

const handleRefresh = async () => {
  try {
    await k8sStore.fetchPolicies()
  } catch (error) {
    console.error('Error fetching policies:', error)
  }
}

const getStatusSeverity = (status: string) => {
  switch (status) {
    case 'Audit':
      return 'info'
    case 'Enforced':
      return 'success'
    case 'Disabled':
      return 'secondary'
    default:
      return 'secondary'
  }
}
</script>

<template>
  <GenericResourceTable
    data-component="policies-data-table"
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
  >
    <!-- Custom Filter -->
    <template #filters v-if="filterOptions.length > 0">
      <TableFilterSelect v-model="selectedType" :options="filterOptions" />
    </template>

    <!-- Custom Name -->
    <template #name="{ data }">
      <span
        v-tooltip="data.name"
        class="font-mono text-(--accent) hover:underline transition-colors truncate"
      >
        {{ data.name }}
      </span>
    </template>

    <!-- Custom Status -->
    <template #status="{ data }">
      <Tag :severity="getStatusSeverity(data.status)" :value="data.status" />
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
            :class="data.violations > 0 ? 'text-(--danger)' : 'text-primary'"
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
