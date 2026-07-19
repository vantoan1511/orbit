<script setup lang="ts">
import { ref, computed } from 'vue'
import DataTable from 'primevue/datatable'
import Column from 'primevue/column'
import Select from 'primevue/select'
import InputText from 'primevue/inputtext'
import Button from 'primevue/button'
import {
  Search,
  Info,
  RefreshCw,
  Settings2,
  Database,
  AlertCircle,
  MoreVertical
} from '@lucide/vue'
import { useKubernetesStore } from '@/stores/kubernetesStore'
import ResourceActionMenu from '@/components/shared/ResourceActionMenu.vue'
import { useWorkloadActions } from '@/composables/useWorkloadActions'
import type { PersistentVolumeInfo } from '@/types/kubernetes'

const k8sStore = useKubernetesStore()
const pvs = computed(() => k8sStore.persistentVolumes)

const searchQuery = ref('')
const selectedStorageClass = ref('All Storage Classes')
const selectedStatus = ref('All Statuses')

const storageClasses = computed(() => {
  const classes = new Set(pvs.value.map((pv) => pv.storageClass))
  return ['All Storage Classes', ...Array.from(classes)]
})

const statuses = ['All Statuses', 'Bound', 'Available', 'Released', 'Failed']

const filteredPVs = computed(() => {
  return pvs.value.filter((pv) => {
    // Search Query
    if (searchQuery.value) {
      const query = searchQuery.value.toLowerCase()
      if (!pv.name.toLowerCase().includes(query)) return false
    }

    // Storage Class
    if (
      selectedStorageClass.value !== 'All Storage Classes' &&
      pv.storageClass !== selectedStorageClass.value
    ) {
      return false
    }

    // Status
    if (selectedStatus.value !== 'All Statuses' && pv.status !== selectedStatus.value) {
      return false
    }

    return true
  })
})

const refreshTable = () => {
  searchQuery.value = ''
  selectedStorageClass.value = 'All Storage Classes'
  selectedStatus.value = 'All Statuses'
}

const actionMenu = ref<InstanceType<typeof ResourceActionMenu> | null>(null)
const selectedActionRow = ref<PersistentVolumeInfo | null>(null)
const dummyDrawerVisible = ref(false)

const toggleActionMenu = (event: Event, data: PersistentVolumeInfo) => {
  event.stopPropagation()
  selectedActionRow.value = data
  actionMenu.value?.toggle(event)
}

const { actionMenuItems } = useWorkloadActions(
  selectedActionRow,
  dummyDrawerVisible,
  ref(null),
  'PersistentVolume'
)
</script>

<template>
  <div class="bg-(--bg-card) border border-(--border) rounded-xl p-5 shadow-sm flex flex-col gap-5">
    <div class="flex items-center justify-between">
      <div
        class="text-sm font-semibold text-(--text-primary) uppercase tracking-wider flex items-center gap-2"
      >
        <Database class="w-4 h-4 text-violet-400" />
        Persistent Volumes ({{ filteredPVs.length }})
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
            placeholder="Search PVs..."
            class="pl-8 pr-3 py-1.5 w-full text-xs bg-(--bg-hover)/30 border-(--border) text-(--text-primary) rounded-lg focus:border-violet-500 focus:ring-1 focus:ring-violet-500"
          />
        </div>

        <!-- StorageClass Select -->
        <Select
          v-model="selectedStorageClass"
          :options="storageClasses"
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
      :value="filteredPVs"
      class="p-datatable-sm border border-(--border) rounded-lg overflow-hidden"
      tableClass="w-full text-left text-xs border-collapse"
    >
      <template #empty>
        <div class="text-center py-8 text-(--text-muted) flex flex-col items-center gap-2">
          <Info class="w-6 h-6 text-(--text-muted)/50" />
          <span>No Persistent Volumes found matching the filters.</span>
        </div>
      </template>

      <!-- Name Column -->
      <Column field="name" header="Name" class="font-medium p-2.5 text-(--text-primary)">
        <template #body="{ data }">
          <div class="flex items-center gap-1.5">
            <span
              class="font-semibold hover:text-violet-400 transition-colors font-mono truncate max-w-56"
              :title="data.name"
            >
              {{ data.name }}
            </span>
            <AlertCircle
              v-if="data.status === 'Failed'"
              class="w-3.5 h-3.5 text-rose-400"
              :title="data.reason"
            />
          </div>
        </template>
      </Column>

      <!-- Capacity Column -->
      <Column
        field="capacity"
        header="Capacity"
        class="p-2.5 font-mono text-(--text-primary)"
      ></Column>

      <!-- Access Mode Column -->
      <Column
        field="accessMode"
        header="Access Mode"
        class="p-2.5 text-(--text-secondary)"
      ></Column>

      <!-- Reclaim Policy Column -->
      <Column field="reclaimPolicy" header="Reclaim" class="p-2.5 text-(--text-muted)"></Column>

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
                'bg-blue-500': data.status === 'Available',
                'bg-amber-500': data.status === 'Released',
                'bg-rose-500': data.status === 'Failed'
              }"
            ></span>
            <span
              class="font-medium"
              :class="{
                'text-emerald-500': data.status === 'Bound',
                'text-blue-500': data.status === 'Available',
                'text-amber-500': data.status === 'Released',
                'text-rose-500': data.status === 'Failed'
              }"
            >
              {{ data.status }}
            </span>
          </div>
        </template>
      </Column>

      <!-- Age Column -->
      <Column field="age" header="Age" class="p-2.5 text-(--text-muted) font-mono"></Column>

      <!-- Actions Column -->
      <Column class="p-3 text-center w-12 shrink-0">
        <template #body="{ data }">
          <Button
            severity="secondary"
            variant="text"
            size="small"
            class="p-1"
            title="Actions"
            @click="toggleActionMenu($event, data)"
          >
            <MoreVertical class="w-4 h-4 text-(--text-muted)" />
          </Button>
        </template>
      </Column>
    </DataTable>

    <ResourceActionMenu ref="actionMenu" :items="actionMenuItems" />
  </div>
</template>
