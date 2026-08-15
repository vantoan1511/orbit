<script setup lang="ts">
import { computed, ref } from 'vue'
import Column from 'primevue/column'
import TableFilterSelect from '@/components/shared/TableFilterSelect.vue'
import ResourceDataTable from '@/components/shared/ResourceDataTable.vue'
import StatusBadge from '@/components/shared/StatusBadge.vue'
import { useKubernetesStore } from '@/stores/kubernetesStore'
import { kubernetesService } from '@/services/kubernetesService'
import { useResourceFilters } from '@/composables/useResourceFilters'
import { useTableColumns } from '@/composables/useTableColumns'

const k8sStore = useKubernetesStore()

const nodesWithNamespace = computed(() => k8sStore.nodes.map((n) => ({ ...n, namespace: '' })))

const { searchQuery, filteredResources } = useResourceFilters(nodesWithNamespace, [
  'name',
  'role',
  'version'
])

const selectedStatus = ref('All Statuses')
const statuses = ['All Statuses', 'Ready', 'NotReady']

const filteredNodes = computed(() => {
  return filteredResources.value.filter((n) => {
    if (selectedStatus.value !== 'All Statuses' && n.status !== selectedStatus.value) {
      return false
    }
    return true
  })
})

const MAX_VISIBLE_LABELS = 2

const { tableColumns, visibleCols } = useTableColumns([
  { field: 'name', header: 'Name', visible: true },
  { field: 'status', header: 'Status', visible: true },
  { field: 'role', header: 'Role', visible: true },
  { field: 'version', header: 'Version', visible: true },
  { field: 'cpuPct', header: 'CPU', visible: true },
  { field: 'memPct', header: 'Memory', visible: true },
  { field: 'podsCount', header: 'Pods', visible: true },
  { field: 'uptime', header: 'Uptime', visible: true },
  { field: 'labels', header: 'Labels', visible: true }
])

const handleRefresh = async () => {
  try {
    await kubernetesService.getNodes()
  } catch (error) {
    console.error('Error fetching nodes:', error)
  }
}
</script>

<template>
  <ResourceDataTable
    :data="filteredNodes"
    v-model:searchQuery="searchQuery"
    v-model:columns="tableColumns"
    searchPlaceholder="Search nodes..."
    emptyMessage="No nodes found in cluster."
    reportTemplate="Showing {first} to {last} of {totalRecords} nodes"
    :loading="k8sStore.nodesLoading"
    @refresh="handleRefresh"
  >
    <!-- Filters -->
    <template #filters>
      <TableFilterSelect v-model="selectedStatus" :options="statuses" />
    </template>

    <!-- Name Column -->
    <Column
      v-if="visibleCols['name']"
      field="name"
      header="Name"
      sortable
      class="p-3"
      bodyClass="font-medium text-primary"
    >
      <template #body="{ data }">
        <span class="font-semibold transition-colors">{{ data.name }}</span>
      </template>
    </Column>

    <!-- Status Column -->
    <Column v-if="visibleCols['status']" field="status" header="Status" sortable class="p-3">
      <template #body="{ data }">
        <StatusBadge :status="data.status" />
      </template>
    </Column>

    <!-- Role Column -->
    <Column v-if="visibleCols['role']" field="role" header="Role" sortable class="p-3">
      <template #body="{ data }">
        <span
          class="px-2 py-0.5 rounded-full text-xs font-bold uppercase tracking-wider border"
          :class="
            data.role === 'control-plane'
              ? 'bg-node/10 text-node border-node/20'
              : 'bg-(--bg-hover) text-muted-color border-(--border)'
          "
        >
          {{ data.role }}
        </span>
      </template>
    </Column>

    <!-- Version Column -->
    <Column
      v-if="visibleCols['version']"
      field="version"
      header="Version"
      class="p-3 font-mono text-muted-color"
    ></Column>

    <!-- CPU Column -->
    <Column
      v-if="visibleCols['cpuPct']"
      field="cpuPct"
      header="CPU"
      sortable
      class="p-3 min-w-35"
      headerClass="text-right justify-end"
      bodyClass="text-right"
    >
      <template #body="{ data }">
        <div class="flex flex-col gap-1 w-full">
          <div class="flex justify-between font-mono text-muted-color">
            <span>{{ Number(data.cpuPct.toFixed(1)) }}%</span>
            <span class="text-muted-color text-[10px]"
              >{{ data.cpuUsed }} / {{ data.cpuTotal }}C</span
            >
          </div>
          <div class="w-full h-1.5 rounded-full bg-(--bg-hover) overflow-hidden">
            <div
              class="h-full bg-blue-500 rounded-full"
              :style="{ width: data.cpuPct + '%' }"
            ></div>
          </div>
        </div>
      </template>
    </Column>

    <!-- Memory Column -->
    <Column
      v-if="visibleCols['memPct']"
      field="memPct"
      header="Memory"
      sortable
      class="p-3 min-w-35"
      headerClass="text-right justify-end"
      bodyClass="text-right"
    >
      <template #body="{ data }">
        <div class="flex flex-col gap-1 w-full">
          <div class="flex justify-between font-mono text-muted-color">
            <span>{{ Number(data.memPct.toFixed(1)) }}%</span>
            <span class="text-muted-color text-[10px]"
              >{{ data.memUsed }} / {{ data.memTotal }}G</span
            >
          </div>
          <div class="w-full h-1.5 rounded-full bg-(--bg-hover) overflow-hidden">
            <div
              class="h-full bg-indigo-500 rounded-full"
              :style="{ width: data.memPct + '%' }"
            ></div>
          </div>
        </div>
      </template>
    </Column>

    <!-- Pods Column -->
    <Column
      v-if="visibleCols['podsCount']"
      field="podsCount"
      header="Pods"
      sortable
      class="p-3 min-w-30"
      headerClass="text-right justify-end"
      bodyClass="text-right"
    >
      <template #body="{ data }">
        <div class="flex flex-col gap-1 w-full">
          <div class="flex justify-between font-mono text-muted-color">
            <span>{{ Math.round((data.podsCount / data.podsLimit) * 100) }}%</span>
            <span class="text-muted-color text-[10px]"
              >{{ data.podsCount }} / {{ data.podsLimit }}</span
            >
          </div>
          <div class="w-full h-1.5 rounded-full bg-(--bg-hover) overflow-hidden">
            <div
              class="h-full bg-sky-500 rounded-full"
              :style="{ width: (data.podsCount / data.podsLimit) * 100 + '%' }"
            ></div>
          </div>
        </div>
      </template>
    </Column>

    <!-- Uptime Column -->
    <Column
      v-if="visibleCols['uptime']"
      field="uptime"
      header="Uptime"
      sortable
      class="p-3"
      bodyClass="text-muted-color"
    ></Column>

    <!-- Labels Column -->
    <Column v-if="visibleCols['labels']" field="labels" header="Labels" class="p-3">
      <template #body="{ data }">
        <div class="flex items-center gap-1 flex-wrap">
          <span
            v-for="label in (data.labels || []).slice(0, MAX_VISIBLE_LABELS)"
            :key="label"
            class="px-1.5 py-0.5 rounded text-[9px] font-mono bg-(--bg-hover) text-muted-color border border-(--border) whitespace-nowrap"
            :title="label"
          >
            {{ label }}
          </span>
          <span
            v-if="(data.labels || []).length > MAX_VISIBLE_LABELS"
            class="px-1.5 py-0.5 rounded text-[9px] font-bold bg-(--bg-hover) text-muted-color border border-(--border)"
            :title="(data.labels || []).slice(MAX_VISIBLE_LABELS).join('\n')"
          >
            +{{ (data.labels || []).length - MAX_VISIBLE_LABELS }}
          </span>
        </div>
      </template>
    </Column>
  </ResourceDataTable>
</template>
