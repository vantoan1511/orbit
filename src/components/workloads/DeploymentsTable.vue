<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import DataTable from 'primevue/datatable'
import Column from 'primevue/column'
import Select from 'primevue/select'
import ToggleSwitch from 'primevue/toggleswitch'
import InputText from 'primevue/inputtext'
import Button from 'primevue/button'
import { Search, Info, RefreshCw } from '@lucide/vue'
import type { DeploymentInfo } from '@/types/kubernetes'
import WorkloadDetailsDrawer from './WorkloadDetailsDrawer.vue'
import { useKubernetesStore } from '@/stores/kubernetesStore'
import { kubernetesService } from '@/services/kubernetesService'

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
watch(() => k8sStore.activeClusterId, () => {
  fetchDeployments()
})

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
  <div class="bg-(--bg-card) border border-(--border) rounded-xl p-6 shadow-sm flex flex-col gap-6">
    <!-- Filter Toolbar -->
    <div class="flex flex-col md:flex-row md:items-center justify-between gap-4">
      <div class="flex items-center gap-3 flex-wrap">
        <!-- Search -->
        <div class="relative min-w-64">
          <Search class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-(--text-muted)" />
          <InputText
            v-model="searchQuery"
            placeholder="Search deployments or images..."
            class="pl-9 pr-4 py-2 w-full text-xs bg-(--bg-hover)/30 border-(--border) text-(--text-primary) rounded-lg focus:border-violet-500 focus:ring-1 focus:ring-violet-500"
          />
        </div>

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
      </div>

      <!-- Toggles and Refresh -->
      <div class="flex items-center gap-4 self-end md:self-auto">
        <div class="flex items-center gap-2">
          <ToggleSwitch v-model="showSystemNamespaces" inputId="system-ns-toggle" />
          <label
            for="system-ns-toggle"
            class="text-xs font-semibold text-(--text-secondary) cursor-pointer select-none"
          >
            Show System
          </label>
        </div>

        <Button severity="secondary" variant="text" size="small" class="p-1" @click="fetchDeployments" :loading="loading">
          <RefreshCw class="w-4 h-4 text-(--text-secondary)" />
        </Button>
      </div>
    </div>

    <!-- Data Table -->
    <DataTable
      :value="filteredDeployments"
      paginator
      :rows="10"
      class="p-datatable-sm border border-(--border) rounded-lg overflow-hidden cursor-pointer"
      tableClass="w-full text-left text-xs border-collapse"
      paginatorTemplate="FirstPageLink PrevPageLink PageLinks NextPageLink LastPageLink RowsPerPageDropdown CurrentPageReport"
      currentPageReportTemplate="Showing {first} to {last} of {totalRecords} deployments"
      @row-click="onRowClick"
    >
      <template #empty>
        <div class="text-center py-10 text-(--text-muted) flex flex-col items-center gap-2">
          <Info class="w-8 h-8 text-(--text-muted)/50" />
          <span>No deployments found matching the filter criteria.</span>
        </div>
      </template>

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
            :class="
              data.available === data.replicas.desired ? 'text-emerald-500' : 'text-amber-500'
            "
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
    </DataTable>

    <!-- Slideout details drawer -->
    <WorkloadDetailsDrawer v-model:visible="drawerVisible" :workload="selectedWorkload" />
  </div>
</template>
