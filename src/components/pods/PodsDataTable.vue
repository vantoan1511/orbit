<script setup lang="ts">
import ResourceDataTable from '@/components/shared/ResourceDataTable.vue'
import { useKubernetesStore } from '@/stores/kubernetesStore'
import type { PodInfo } from '@/types/kubernetes'
import { Power, Trash2 } from '@lucide/vue'
import Button from 'primevue/button'
import Column from 'primevue/column'
import Select from 'primevue/select'
import ToggleSwitch from 'primevue/toggleswitch'
import { useToast } from 'primevue/usetoast'
import { computed, ref } from 'vue'
import PodDetailsDrawer from './PodDetailsDrawer.vue'

const toast = useToast()
const k8sStore = useKubernetesStore()

const searchQuery = ref('')
const selectedNamespace = ref('All Namespaces')
const selectedStatus = ref('All Statuses')
const showSystemNamespaces = ref(false)

// Drawer state
const drawerVisible = ref(false)
const selectedPod = ref<PodInfo | null>(null)

const namespaces = computed(() => {
  return k8sStore.namespaces
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
  return k8sStore.pods.filter((p) => {
    // Search Query filter
    if (searchQuery.value) {
      const query = searchQuery.value.toLowerCase()
      const matchesName = p.name.toLowerCase().includes(query)
      if (!matchesName) return false
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
  toast.add({
    severity: action === 'Terminate' ? 'warn' : 'info',
    summary: `${action} Pod`,
    detail: `Action triggered for pod: ${podName}`,
    life: 3000
  })
}
</script>

<template>
  <ResourceDataTable
    :data="filteredPods"
    v-model:searchQuery="searchQuery"
    searchPlaceholder="Search pods, images or nodes..."
    emptyMessage="No pods found matching the filter criteria."
    reportTemplate="Showing {first} to {last} of {totalRecords} pods"
    @row-click="onRowClick"
  >
    <!-- Filters -->
    <template #filters>
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
    </template>

    <!-- Actions Left -->
    <template #actions-left>
      <div class="flex items-center gap-2">
        <ToggleSwitch v-model="showSystemNamespaces" inputId="system-ns-toggle" />
        <label
          for="system-ns-toggle"
          class="text-xs font-semibold text-(--text-secondary) cursor-pointer select-none"
        >
          Show System
        </label>
      </div>
    </template>

    <!-- Columns -->
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
          :title="data.node || 'N/A'"
        >
          {{ data.node ? data.node.split('.')[0] : 'N/A' }}
        </span>
      </template>
    </Column>

    <!-- Restarts Column -->
    <Column field="restarts" header="Restarts" sortable class="p-3 text-center">
      <template #body="{ data }">
        <span
          class="font-mono px-1.5 py-0.5 rounded text-[10px]"
          :class="
            (data.restarts || 0) > 0
              ? 'bg-rose-500/10 text-rose-400 border border-rose-500/20'
              : 'text-(--text-muted)'
          "
        >
          {{ data.restarts || 0 }}
        </span>
      </template>
    </Column>

    <!-- CPU Column -->
    <Column field="cpu" header="CPU" sortable class="p-3">
      <template #body="{ data }">
        <div class="flex flex-col gap-1 w-24">
          <div class="flex justify-between font-mono text-[10px]">
            <span class="text-(--text-secondary)">{{ data.cpu || '-' }}</span>
            <span class="text-(--text-muted)" v-if="data.cpu && data.cpu !== '-'"
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
    <Column field="memory" header="Memory" sortable class="p-3">
      <template #body="{ data }">
        <div class="flex flex-col gap-1 w-24">
          <div class="flex justify-between font-mono text-[10px]">
            <span class="text-(--text-secondary)">{{ data.memory || '-' }}</span>
            <span class="text-(--text-muted)" v-if="data.memory && data.memory !== '-'"
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

    <!-- Drawer -->
    <template #drawer>
      <PodDetailsDrawer v-model:visible="drawerVisible" :pod="selectedPod" />
    </template>
  </ResourceDataTable>
</template>
