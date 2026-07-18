<script setup lang="ts">
import ResourceDataTable from '@/components/shared/ResourceDataTable.vue'
import { kubernetesService } from '@/services/kubernetesService'
import { useKubernetesStore } from '@/stores/kubernetesStore'
import type { DeploymentInfo } from '@/types/kubernetes'
import Column from 'primevue/column'
import Select from 'primevue/select'
import ToggleSwitch from 'primevue/toggleswitch'
import { computed, onMounted, ref, watch } from 'vue'
import WorkloadDetailsDrawer from './WorkloadDetailsDrawer.vue'

const k8sStore = useKubernetesStore()

const searchQuery = ref('')
const selectedNamespace = ref('All Namespaces')
const selectedStatus = ref('All Statuses')
const showSystemNamespaces = ref(false)
const loading = ref(false)

// Drawer state
const drawerVisible = ref(false)
const selectedWorkload = ref<DeploymentInfo | null>(null)

const namespaces = computed(() => {
  return k8sStore.namespaces
})

const statuses = ['All Statuses', 'Running', 'Progressing', 'Failed']

const fetchDeployments = async () => {
  loading.value = true
  try {
    const ns = selectedNamespace.value === 'All Namespaces' ? undefined : selectedNamespace.value
    await kubernetesService.getDeployments(ns)
  } catch (e) {
    console.error('Error fetching deployments', e)
  } finally {
    loading.value = false
  }
}

onMounted(() => {
  fetchDeployments()
})

// Refetch when namespace changes
watch(selectedNamespace, () => {
  fetchDeployments()
})

// Refetch on cluster change
watch(
  () => k8sStore.activeClusterId,
  () => {
    fetchDeployments()
  }
)

const filteredDeployments = computed(() => {
  return k8sStore.deployments.filter((d) => {
    // Search Query filter
    if (searchQuery.value) {
      const query = searchQuery.value.toLowerCase()
      const matchesName = d.name.toLowerCase().includes(query)
      const matchesImage = d.images?.some((img) => img.toLowerCase().includes(query))
      if (!matchesName && !matchesImage) return false
    }

    // System Namespaces filter
    const isSystem = ['kube-system', 'monitoring', 'logging'].includes(d.namespace)
    if (!showSystemNamespaces.value && isSystem && selectedNamespace.value === 'All Namespaces') {
      return false
    }

    // Status filter
    if (selectedStatus.value !== 'All Statuses' && d.status !== selectedStatus.value) {
      return false
    }

    return true
  })
})

const onRowClick = (event: { data: DeploymentInfo }) => {
  selectedWorkload.value = event.data
  drawerVisible.value = true
}
</script>

<template>
  <ResourceDataTable
    :data="filteredDeployments"
    v-model:searchQuery="searchQuery"
    searchPlaceholder="Search deployments or images..."
    emptyMessage="No deployments found matching the filter criteria."
    reportTemplate="Showing {first} to {last} of {totalRecords} deployments"
    :loading="loading"
    @refresh="fetchDeployments"
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
            :class="
              data.status === 'Running'
                ? 'bg-emerald-500'
                : data.status === 'Progressing'
                  ? 'bg-amber-500'
                  : 'bg-rose-500'
            "
          ></span>
          <span
            class="font-medium"
            :class="
              data.status === 'Running'
                ? 'text-emerald-500'
                : data.status === 'Progressing'
                  ? 'text-amber-500'
                  : 'text-rose-500'
            "
          >
            {{ data.status }}
          </span>
        </div>
      </template>
    </Column>

    <!-- Replicas Column -->
    <Column header="Replicas" class="p-3">
      <template #body="{ data }">
        <div class="flex items-center gap-2 font-mono text-(--text-secondary)">
          <span class="font-bold">{{ data.replicas.current }}</span>
          <span class="text-(--text-muted)">/</span>
          <span>{{ data.replicas.desired }}</span>
        </div>
      </template>
    </Column>

    <!-- Available Column -->
    <Column field="available" header="Available" sortable class="p-3">
      <template #body="{ data }">
        <span
          class="font-mono"
          :class="data.available === data.replicas.desired ? 'text-emerald-500' : 'text-amber-500'"
        >
          {{ data.available }}
        </span>
      </template>
    </Column>

    <!-- Up to Date Column -->
    <Column field="upToDate" header="Up-To-Date" sortable class="p-3">
      <template #body="{ data }">
        <span class="font-mono text-(--text-secondary)">{{ data.upToDate }}</span>
      </template>
    </Column>

    <!-- Age Column -->
    <Column field="age" header="Age" sortable class="p-3 text-(--text-muted) font-mono"></Column>

    <!-- Images Column -->
    <Column header="Images" class="p-3 max-w-48">
      <template #body="{ data }">
        <div class="flex flex-wrap gap-1">
          <span
            v-for="img in data.images"
            :key="img"
            class="px-1.5 py-0.5 rounded bg-(--bg-hover) text-(--text-secondary) text-[10px] border border-(--border) font-mono truncate max-w-full"
            :title="img"
          >
            {{ img.split('/').pop() }}
          </span>
        </div>
      </template>
    </Column>

    <!-- Drawer -->
    <template #drawer>
      <WorkloadDetailsDrawer v-model:visible="drawerVisible" :workload="selectedWorkload" />
    </template>
  </ResourceDataTable>
</template>
