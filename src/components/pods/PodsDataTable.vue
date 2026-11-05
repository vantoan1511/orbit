<script setup lang="ts">
import GenericResourceTable from '@/components/shared/GenericResourceTable.vue'
import { KUBERNETES_POD_STATUS, KUBERNETES_RESOURCE_KIND } from '@/constants/kubernetes'
import { useKubernetesStore } from '@/stores/kubernetesStore'
import Column from 'primevue/column'
import { computed } from 'vue'
import PodDetailsDrawer from './PodDetailsDrawer.vue'
import { createPodRowDecorator } from '@/utils/workloadMetrics'

const k8sStore = useKubernetesStore()
const podDecorator = createPodRowDecorator()

const pods = computed(() => podDecorator(k8sStore.pods))

const columns = [
  { field: 'namespace', header: 'Namespace', visible: true },
  { field: 'status', header: 'Status', visible: true },
  { field: 'node', header: 'Node', visible: true },
  { field: 'restarts', header: 'Restarts', visible: true },
  { field: 'cpu', header: 'CPU', visible: true },
  { field: 'memory', header: 'Memory', visible: true },
  { field: 'age', header: 'Age', visible: true }
]

const statuses = ['All Statuses', ...Object.values(KUBERNETES_POD_STATUS)]
</script>

<template>
  <GenericResourceTable
    data-component="pods-data-table"
    :data="pods"
    :initialColumns="columns"
    :statuses="statuses"
    :searchFields="['name', 'node']"
    :kind="KUBERNETES_RESOURCE_KIND.Pod"
    searchPlaceholder="Search pods, images or nodes..."
    emptyMessage="No pods found matching the filter criteria."
    reportTemplate="Showing {first} to {last} of {totalRecords} pods"
    :loading="k8sStore.podsLoading"
  >
    <template #default="{ visibleCols }">
      <!-- Node Column -->
      <Column v-if="visibleCols['node']" field="node" header="Node" sortable class="p-3">
        <template #body="{ data }">
          <span
            class="text-muted-color font-mono truncate block max-w-44"
            v-tooltip.top="data.node"
          >
            {{ data.nodeShort }}
          </span>
        </template>
      </Column>

      <!-- Restarts Column -->
      <Column
        v-if="visibleCols['restarts']"
        field="restarts"
        header="Restarts"
        sortable
        class="p-3 text-center"
      >
        <template #body="{ data }">
          <Tag
            v-if="(data.restarts || 0) > 0"
            severity="danger"
            class="font-mono"
            :value="data.restarts"
          />
          <span v-else class="font-mono text-muted-color"> 0 </span>
        </template>
      </Column>

      <!-- CPU Column -->
      <Column v-if="visibleCols['cpu']" field="cpu" header="CPU" sortable class="p-3">
        <template #body="{ data }">
          <div class="flex flex-col gap-1 w-28">
            <div class="flex justify-between font-mono text-[10px]">
              <span class="text-muted-color">{{ data.cpu || '-' }}</span>
              <span class="text-muted-color" v-if="data.hasCpuPct">{{ data.cpuPct }}%</span>
            </div>
            <div
              class="w-full h-1 bg-(--bg-hover) rounded-full overflow-hidden"
              v-if="data.hasCpuPct"
            >
              <div
                class="h-full rounded-full bg-(--accent)"
                :style="{ width: data.cpuWidthStyle }"
              ></div>
            </div>
          </div>
        </template>
      </Column>

      <!-- Memory Column -->
      <Column v-if="visibleCols['memory']" field="memory" header="Memory" sortable class="p-3">
        <template #body="{ data }">
          <div class="flex flex-col gap-1 w-28">
            <div class="flex justify-between font-mono text-[10px]">
              <span class="text-muted-color">{{ data.memory || '-' }}</span>
              <span class="text-muted-color" v-if="data.hasMemoryPct">{{ data.memoryPct }}%</span>
            </div>
            <div
              class="w-full h-1 bg-(--bg-hover) rounded-full overflow-hidden"
              v-if="data.hasMemoryPct"
            >
              <div
                class="h-full rounded-full bg-(--color-node)"
                :style="{ width: data.memoryWidthStyle }"
              ></div>
            </div>
          </div>
        </template>
      </Column>
    </template>

    <!-- Drawer -->
    <template #drawer="{ selectedItem, visible, close }">
      <PodDetailsDrawer
        :visible="visible"
        :pod="selectedItem"
        @update:visible="!$event && close()"
      />
    </template>
  </GenericResourceTable>
</template>
