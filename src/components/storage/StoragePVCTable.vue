<script setup lang="ts">
import { ref, computed } from 'vue'
import DataTable from 'primevue/datatable'
import Column from 'primevue/column'
import Select from 'primevue/select'
import InputText from 'primevue/inputtext'
import Button from 'primevue/button'
import { Search, Info, RefreshCw, Settings2, FileSpreadsheet } from '@lucide/vue'
import { useKubernetesStore } from '@/stores/kubernetesStore'

const k8sStore = useKubernetesStore()
const pvcs = computed(() => k8sStore.persistentVolumeClaims)

const searchQuery = ref('')
const selectedNamespace = ref<string[]>([])
const selectedStatus = ref('All Statuses')

const namespaces = computed(() => {
  const ns = new Set(pvcs.value.map((pvc) => pvc.namespace))
  return ['All Namespaces', ...Array.from(ns)]
})

const statuses = ['All Statuses', 'Bound', 'Pending', 'Lost']

const filteredPVCs = computed(() => {
  return pvcs.value.filter((pvc) => {
    // Search Query
    if (searchQuery.value) {
      const query = searchQuery.value.toLowerCase()
      if (!pvc.name.toLowerCase().includes(query)) return false
    }

    // Namespace
    if (selectedNamespace.value.length > 0 && !selectedNamespace.value.includes(pvc.namespace)) {
      return false
    }

    // Status
    if (selectedStatus.value !== 'All Statuses' && pvc.status !== selectedStatus.value) {
      return false
    }

    return true
  })
})

const refreshTable = () => {
  searchQuery.value = ''
  selectedNamespace.value = []
  selectedStatus.value = 'All Statuses'
}
</script>

<template>
  <div class="bg-(--bg-card) border border-(--border) rounded-xl p-5 shadow-sm flex flex-col gap-5">
    <div class="flex items-center justify-between">
      <div
        class="text-sm font-semibold text-(--text-primary) uppercase tracking-wider flex items-center gap-2"
      >
        <FileSpreadsheet class="w-4 h-4 text-blue-400" />
        Persistent Volume Claims ({{ filteredPVCs.length }})
      </div>
    </div>

    <!-- Filter Toolbar -->
    <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-3">
      <div class="flex items-center gap-2.5 flex-wrap">
        <!-- Search -->
        <div class="relative min-w-56">
          <Search
            class="absolute left-2.5 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-(--text-muted)"
          />
          <InputText
            v-model="searchQuery"
            placeholder="Search Claims..."
            class="pl-8 pr-3 py-1.5 w-full text-xs bg-(--bg-hover)/30 border-(--border) text-(--text-primary) rounded-lg focus:border-violet-500 focus:ring-1 focus:ring-violet-500"
          />
        </div>

        <!-- Namespace Select -->
        <Select
          v-model="selectedNamespace"
          :options="namespaces"
          class="text-xs min-w-40 bg-(--bg-hover)/30 border-(--border)"
        />

        <!-- Status Select -->
        <Select
          v-model="selectedStatus"
          :options="statuses"
          class="text-xs min-w-36 bg-(--bg-hover)/30 border-(--border)"
        />
      </div>

      <!-- Action buttons -->
      <div class="flex items-center gap-1 self-end sm:self-auto">
        <Button severity="secondary" variant="text" size="small" class="p-1" @click="refreshTable">
          <RefreshCw class="w-3.5 h-3.5 text-(--text-secondary)" />
        </Button>
        <Button severity="secondary" variant="text" size="small" class="p-1">
          <Settings2 class="w-3.5 h-3.5 text-(--text-secondary)" />
        </Button>
      </div>
    </div>

    <!-- DataTable -->
    <DataTable
      :value="filteredPVCs"
      class="p-datatable-sm border border-(--border) rounded-lg overflow-hidden"
      tableClass="w-full text-left text-xs border-collapse"
    >
      <template #empty>
        <div class="text-center py-8 text-(--text-muted) flex flex-col items-center gap-2">
          <Info class="w-6 h-6 text-(--text-muted)/50" />
          <span>No PVCs found matching the filters.</span>
        </div>
      </template>

      <!-- Name Column -->
      <Column field="name" header="Name" class="font-medium p-2.5 text-(--text-primary)">
        <template #body="{ data }">
          <span
            class="font-semibold hover:text-violet-400 transition-colors font-mono truncate max-w-48 block"
            :title="data.name"
          >
            {{ data.name }}
          </span>
        </template>
      </Column>

      <!-- Namespace Column -->
      <Column field="namespace" header="Namespace" class="p-2.5">
        <template #body="{ data }">
          <span
            class="font-mono px-2 py-0.5 rounded text-[10px]"
            :class="[
              data.namespace === 'kube-system'
                ? 'bg-zinc-500/10 text-zinc-400 border border-zinc-500/20'
                : data.namespace === 'monitoring'
                  ? 'bg-amber-500/10 text-amber-400 border border-amber-500/20'
                  : data.namespace === 'logging'
                    ? 'bg-cyan-500/10 text-cyan-400 border border-cyan-500/20'
                    : data.namespace === 'backend'
                      ? 'bg-indigo-500/10 text-indigo-400 border border-indigo-500/20'
                      : 'bg-purple-500/10 text-purple-400 border border-purple-500/20'
            ]"
          >
            {{ data.namespace }}
          </span>
        </template>
      </Column>

      <!-- Volume Column -->
      <Column field="volume" header="Volume" class="p-2.5">
        <template #body="{ data }">
          <span
            v-if="data.volume"
            class="font-mono text-(--text-secondary) truncate max-w-44 block"
            :title="data.volume"
          >
            {{ data.volume }}
          </span>
          <span v-else class="text-(--text-muted) italic font-mono">-</span>
        </template>
      </Column>

      <!-- Capacity Column -->
      <Column
        field="capacity"
        header="Request"
        class="p-2.5 font-mono text-(--text-primary)"
      ></Column>

      <!-- Access Mode Column -->
      <Column
        field="accessMode"
        header="Access Mode"
        class="p-2.5 text-(--text-secondary)"
      ></Column>

      <!-- Storage Class Column -->
      <Column field="storageClass" header="Storage Class" class="p-2.5">
        <template #body="{ data }">
          <span class="font-mono text-violet-400 font-semibold">{{ data.storageClass }}</span>
        </template>
      </Column>

      <!-- Status Column -->
      <Column field="status" header="Status" class="p-2.5">
        <template #body="{ data }">
          <div class="flex items-center gap-1.5">
            <span
              class="w-1.5 h-1.5 rounded-full"
              :class="{
                'bg-emerald-500': data.status === 'Bound',
                'bg-amber-500': data.status === 'Pending',
                'bg-rose-500': data.status === 'Lost'
              }"
            ></span>
            <span
              class="font-medium"
              :class="{
                'text-emerald-500': data.status === 'Bound',
                'text-amber-500': data.status === 'Pending',
                'text-rose-500': data.status === 'Lost'
              }"
            >
              {{ data.status }}
            </span>
          </div>
        </template>
      </Column>

      <!-- Age Column -->
      <Column field="age" header="Age" class="p-2.5 text-(--text-muted) font-mono"></Column>
    </DataTable>
  </div>
</template>
