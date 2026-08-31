<script setup lang="ts">
import NodeDetailsDrawer from '@/components/nodes/NodeDetailsDrawer.vue'
import GenericResourceTable from '@/components/shared/GenericResourceTable.vue'
import { KUBERNETES_RESOURCE_KIND } from '@/constants/kubernetes'
import { kubernetesService } from '@/services/kubernetesService'
import { useKubernetesStore } from '@/stores/kubernetesStore'
import Column from 'primevue/column'

const k8sStore = useKubernetesStore()

const MAX_VISIBLE_LABELS = 2

const columns = [
  { field: 'name', header: 'Name', visible: true },
  { field: 'status', header: 'Status', visible: true },
  { field: 'role', header: 'Role', visible: true },
  { field: 'version', header: 'Version', visible: true },
  { field: 'cpuPct', header: 'CPU', visible: true },
  { field: 'memPct', header: 'Memory', visible: true },
  { field: 'podsCount', header: 'Pods', visible: true },
  { field: 'uptime', header: 'Uptime', visible: true },
  { field: 'labels', header: 'Labels', visible: true }
]

const statuses = ['All Statuses', 'Ready', 'NotReady']

const handleRefresh = async () => {
  try {
    await kubernetesService.getNodes()
  } catch (error) {
    console.error('Error fetching nodes:', error)
  }
}
</script>

<template>
  <GenericResourceTable
    :kind="KUBERNETES_RESOURCE_KIND.Node"
    :data="k8sStore.nodes"
    :initialColumns="columns"
    :statuses="statuses"
    :searchFields="['name', 'role', 'version']"
    :hideNamespaceFilter="true"
    :hideNamespaceColumn="true"
    :hideAgeColumn="true"
    :hideActionsColumn="true"
    searchPlaceholder="Search nodes..."
    emptyMessage="No nodes found in cluster."
    reportTemplate="Showing {first} to {last} of {totalRecords} nodes"
    :loading="k8sStore.nodesLoading"
    @refresh="handleRefresh"
  >
    <template #default="{ visibleCols }">
      <!-- Role Column -->
      <Column v-if="visibleCols['role']" field="role" header="Role" sortable class="p-3">
        <template #body="{ data }">
          <Tag
            rounded
            :severity="data.role === 'control-plane' ? 'info' : 'secondary'"
            :value="data.role"
          />
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
            <Tag
              v-for="label in (data.labels || []).slice(0, MAX_VISIBLE_LABELS)"
              :key="label"
              severity="secondary"
              class="font-mono whitespace-nowrap"
              :title="label"
              :value="label"
            />
            <Tag
              v-if="(data.labels || []).length > MAX_VISIBLE_LABELS"
              severity="secondary"
              :title="(data.labels || []).slice(MAX_VISIBLE_LABELS).join('\n')"
              :value="`+${(data.labels || []).length - MAX_VISIBLE_LABELS}`"
            />
          </div>
        </template>
      </Column>
    </template>

    <!-- Drawer -->
    <template #drawer="{ selectedItem, visible, close }">
      <NodeDetailsDrawer
        :visible="visible"
        :node="selectedItem"
        @update:visible="!$event && close()"
      />
    </template>
  </GenericResourceTable>
</template>
