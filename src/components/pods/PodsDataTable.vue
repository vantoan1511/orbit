<script setup lang="ts">
import GenericResourceTable from '@/components/shared/GenericResourceTable.vue'
import { useKubernetesStore } from '@/stores/kubernetesStore'
import Column from 'primevue/column'
import PodDetailsDrawer from './PodDetailsDrawer.vue'

const k8sStore = useKubernetesStore()

const columns = [
  { field: 'namespace', header: 'Namespace', visible: true },
  { field: 'status', header: 'Status', visible: true },
  { field: 'node', header: 'Node', visible: true },
  { field: 'restarts', header: 'Restarts', visible: true },
  { field: 'cpu', header: 'CPU', visible: true },
  { field: 'memory', header: 'Memory', visible: true },
  { field: 'age', header: 'Age', visible: true }
]

const statuses = [
  'All Statuses',
  'Running',
  'Pending',
  'Failed',
  'CrashLoopBackOff',
  'Completed',
  'Unknown'
]
</script>

<template>
  <GenericResourceTable
    :data="k8sStore.pods"
    :initialColumns="columns"
    :statuses="statuses"
    :searchFields="['name', 'node']"
    kind="Pod"
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
            :title="data.node || 'N/A'"
          >
            {{ data.node ? data.node.split('.')[0] : 'N/A' }}
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
          <span
            class="font-mono px-1.5 py-0.5 rounded text-[10px]"
            :class="
              (data.restarts || 0) > 0
                ? 'bg-rose-500/10 text-rose-400 border border-rose-500/20'
                : 'text-muted-color'
            "
          >
            {{ data.restarts || 0 }}
          </span>
        </template>
      </Column>

      <!-- CPU Column -->
      <Column v-if="visibleCols['cpu']" field="cpu" header="CPU" sortable class="p-3">
        <template #body="{ data }">
          <div class="flex flex-col gap-1 w-24">
            <div class="flex justify-between font-mono text-[10px]">
              <span class="text-muted-color">{{ data.cpu || '-' }}</span>
              <span class="text-muted-color" v-if="data.cpu && data.cpu !== '-'"
                >{{ data.cpuPct || 0 }}%</span
              >
            </div>
            <div
              class="w-full h-1 bg-(--bg-hover) rounded-full overflow-hidden"
              v-if="data.cpu && data.cpu !== '-'"
            >
              <div
                class="h-full rounded-full bg-violet-500"
                :style="{ width: (data.cpuPct || 0) + '%' }"
              ></div>
            </div>
          </div>
        </template>
      </Column>

      <!-- Memory Column -->
      <Column v-if="visibleCols['memory']" field="memory" header="Memory" sortable class="p-3">
        <template #body="{ data }">
          <div class="flex flex-col gap-1 w-24">
            <div class="flex justify-between font-mono text-[10px]">
              <span class="text-muted-color">{{ data.memory || '-' }}</span>
              <span class="text-muted-color" v-if="data.memory && data.memory !== '-'"
                >{{ data.memoryPct || 0 }}%</span
              >
            </div>
            <div
              class="w-full h-1 bg-(--bg-hover) rounded-full overflow-hidden"
              v-if="data.memory && data.memory !== '-'"
            >
              <div
                class="h-full rounded-full bg-blue-500"
                :style="{ width: (data.memoryPct || 0) + '%' }"
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
