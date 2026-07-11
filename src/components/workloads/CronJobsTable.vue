<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import DataTable from 'primevue/datatable'
import Column from 'primevue/column'
import Select from 'primevue/select'
import ToggleSwitch from 'primevue/toggleswitch'
import InputText from 'primevue/inputtext'
import Button from 'primevue/button'
import { Search, Info, RefreshCw } from '@lucide/vue'
import type { CronJobInfo } from '@/types/kubernetes'
import WorkloadDetailsDrawer from './WorkloadDetailsDrawer.vue'
import { useKubernetesStore } from '@/stores/kubernetesStore'
import { kubernetesService } from '@/services/kubernetesService'

const k8sStore = useKubernetesStore()

const searchQuery = ref('')
const selectedNamespace = ref('All Namespaces')
const selectedSuspend = ref('All Suspend States')
const showSystemNamespaces = ref(false)
const loading = ref(false)

// Drawer state
const drawerVisible = ref(false)
const selectedWorkload = ref<CronJobInfo | null>(null)

const namespaces = computed(() => {
  return k8sStore.namespaces
})

const suspendOptions = ['All Suspend States', 'Suspended', 'Active']

const fetchCronJobs = async () => {
  loading.value = true
  try {
    const ns = selectedNamespace.value === 'All Namespaces' ? undefined : selectedNamespace.value
    await kubernetesService.getCronJobs(ns)
  } catch (e) {
    console.error('Error fetching cronjobs', e)
  } finally {
    loading.value = false
  }
}

onMounted(() => {
  fetchCronJobs()
})

watch(selectedNamespace, () => {
  fetchCronJobs()
})

watch(() => k8sStore.activeClusterId, () => {
  fetchCronJobs()
})

const filteredCronJobs = computed(() => {
  return k8sStore.cronJobs.filter((cj) => {
    if (searchQuery.value) {
      const query = searchQuery.value.toLowerCase()
      const matchesName = cj.name.toLowerCase().includes(query)
      const matchesImage = cj.images?.some((img) => img.toLowerCase().includes(query))
      if (!matchesName && !matchesImage) return false
    }

    const isSystem = ['kube-system', 'monitoring', 'logging'].includes(cj.namespace)
    if (!showSystemNamespaces.value && isSystem && selectedNamespace.value === 'All Namespaces') {
      return false
    }

    if (selectedSuspend.value !== 'All Suspend States') {
      const isSuspended = selectedSuspend.value === 'Suspended'
      if (cj.suspend !== isSuspended) return false
    }

    return true
  })
})

const onRowClick = (event: { data: CronJobInfo }) => {
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
            placeholder="Search cronjobs or images..."
            class="pl-9 pr-4 py-2 w-full text-xs bg-(--bg-hover)/30 border-(--border) text-(--text-primary) rounded-lg focus:border-violet-500 focus:ring-1 focus:ring-violet-500"
          />
        </div>

        <!-- Namespace Select -->
        <Select
          v-model="selectedNamespace"
          :options="namespaces"
          class="text-xs min-w-44 bg-(--bg-hover)/30 border-(--border)"
        />

        <!-- Suspend Select -->
        <Select
          v-model="selectedSuspend"
          :options="suspendOptions"
          class="text-xs min-w-44 bg-(--bg-hover)/30 border-(--border)"
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

        <Button severity="secondary" variant="text" size="small" class="p-1" @click="fetchCronJobs" :loading="loading">
          <RefreshCw class="w-4 h-4 text-(--text-secondary)" />
        </Button>
      </div>
    </div>

    <!-- Data Table -->
    <DataTable
      :value="filteredCronJobs"
      paginator
      :rows="10"
      class="p-datatable-sm border border-(--border) rounded-lg overflow-hidden cursor-pointer"
      tableClass="w-full text-left text-xs border-collapse"
      paginatorTemplate="FirstPageLink PrevPageLink PageLinks NextPageLink LastPageLink RowsPerPageDropdown CurrentPageReport"
      currentPageReportTemplate="Showing {first} to {last} of {totalRecords} cronjobs"
      @row-click="onRowClick"
    >
      <template #empty>
        <div class="text-center py-10 text-(--text-muted) flex flex-col items-center gap-2">
          <Info class="w-8 h-8 text-(--text-muted)/50" />
          <span>No cronjobs found matching the filter criteria.</span>
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

      <!-- Schedule Column -->
      <Column field="schedule" header="Schedule" sortable class="p-3 font-mono text-(--text-primary)"></Column>

      <!-- Suspend Column -->
      <Column field="suspend" header="Suspend" sortable class="p-3">
        <template #body="{ data }">
          <span
            class="font-medium px-2 py-0.5 rounded text-[10px] uppercase tracking-wider"
            :class="data.suspend ? 'bg-amber-500/10 text-amber-400 border border-amber-500/20' : 'bg-emerald-500/10 text-emerald-400 border border-emerald-500/20'"
          >
            {{ data.suspend ? 'True' : 'False' }}
          </span>
        </template>
      </Column>

      <!-- Active Column -->
      <Column field="active" header="Active Jobs" sortable class="p-3 text-center">
        <template #body="{ data }">
          <span class="font-mono text-(--text-secondary)">{{ data.active }}</span>
        </template>
      </Column>

      <!-- Last Schedule Column -->
      <Column field="lastSchedule" header="Last Schedule" sortable class="p-3 font-mono text-(--text-secondary)">
        <template #body="{ data }">
          <span>{{ data.lastSchedule || '-' }}</span>
        </template>
      </Column>

      <!-- Age Column -->
      <Column field="age" header="Age" sortable class="p-3 text-(--text-muted) font-mono"></Column>
    </DataTable>

    <!-- Slideout details drawer -->
    <WorkloadDetailsDrawer v-model:visible="drawerVisible" :workload="selectedWorkload" />
  </div>
</template>
