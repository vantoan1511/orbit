<script setup lang="ts">
import { ref, computed, onMounted, type Ref } from 'vue'
import DataTable from 'primevue/datatable'
import Column from 'primevue/column'
import Select from 'primevue/select'
import ToggleSwitch from 'primevue/toggleswitch'
import InputText from 'primevue/inputtext'
import Button from 'primevue/button'
import { Search, Info, RefreshCw, Settings2, MoreVertical } from '@lucide/vue'
import type { EventInfo } from '@/types/kubernetes'
import EventDetailsDrawer from './EventDetailsDrawer.vue'
import { useKubernetesStore } from '@/stores/kubernetesStore'
import { storeToRefs } from 'pinia'
import ResourceActionMenu from '@/components/shared/ResourceActionMenu.vue'
import { useWorkloadActions } from '@/composables/useWorkloadActions'

const k8sStore = useKubernetesStore()
const { events } = storeToRefs(k8sStore)

const handleRefresh = async () => {
  try {
    await k8sStore.fetchEvents()
  } catch (error) {
    console.error('Error fetching events:', error)
  }
}

onMounted(() => {
  k8sStore.fetchEvents()
})

const searchQuery = ref('')
const selectedNamespace = ref<string[]>([])
const selectedType = ref('All Types')
const showSystemNamespaces = ref(false)

// Drawer state
const drawerVisible = ref(false)
const selectedEvent = ref<EventInfo | null>(null)

const namespaces = computed(() => {
  const list = new Set(events.value.map((e) => e.namespace))
  return ['All Namespaces', ...Array.from(list)]
})

const types = ['All Types', 'Normal', 'Warning', 'Error']

const filteredEvents = computed(() => {
  return events.value.filter((e) => {
    // Search Query filter
    if (searchQuery.value) {
      const query = searchQuery.value.toLowerCase()
      const matchesSearch =
        e.message.toLowerCase().includes(query) ||
        e.reason.toLowerCase().includes(query) ||
        e.objectName.toLowerCase().includes(query) ||
        e.objectKind.toLowerCase().includes(query) ||
        e.source.toLowerCase().includes(query)

      if (!matchesSearch) return false
    }

    // Namespace filter
    if (selectedNamespace.value.length > 0 && !selectedNamespace.value.includes(e.namespace)) {
      return false
    }

    // System Namespaces filter
    const isSystem = ['kube-system', 'monitoring', 'logging'].includes(e.namespace)
    if (!showSystemNamespaces.value && isSystem && selectedNamespace.value.length === 0) {
      return false
    }

    // Type filter
    if (selectedType.value !== 'All Types' && e.type !== selectedType.value) {
      return false
    }

    return true
  })
})

const onRowClick = (event: { data: EventInfo }) => {
  selectedEvent.value = event.data
  drawerVisible.value = true
}

const getTypeBadgeClass = (type: string) => {
  switch (type) {
    case 'Warning':
      return 'bg-amber-500/10 text-amber-400 border border-amber-500/20'
    case 'Error':
      return 'bg-red-500/10 text-red-400 border border-red-500/20'
    case 'Normal':
      return 'bg-emerald-500/10 text-emerald-400 border border-emerald-500/20'
    default:
      return 'bg-gray-500/10 text-gray-400 border border-gray-500/20'
  }
}

const actionMenu = ref<InstanceType<typeof ResourceActionMenu> | null>(null)
const selectedActionRow = ref<(EventInfo & { name: string }) | null>(null)

const toggleActionMenu = (event: Event, data: EventInfo) => {
  event.stopPropagation()
  // satisfy T extends { name: string } constraint
  selectedActionRow.value = { ...data, name: data.objectName }
  actionMenu.value?.toggle(event)
}

const { actionMenuItems } = useWorkloadActions(
  selectedActionRow,
  drawerVisible,
  selectedEvent as Ref<(EventInfo & { name: string }) | null>,
  'Event'
)
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
            placeholder="Search events..."
            class="pl-9 pr-4 py-2 w-full text-xs bg-(--bg-hover)/30 border-(--border) text-(--text-primary) rounded-lg focus:border-violet-500 focus:ring-1 focus:ring-violet-500"
          />
        </div>

        <!-- Namespace Select -->
        <Select
          v-model="selectedNamespace"
          :options="namespaces"
          class="text-xs min-w-44 bg-(--bg-hover)/30 border-(--border)"
        />

        <!-- Type Select -->
        <Select
          v-model="selectedType"
          :options="types"
          class="text-xs min-w-40 bg-(--bg-hover)/30 border-(--border)"
        />
      </div>

      <!-- Toggles and Actions -->
      <div class="flex items-center gap-4 self-end md:self-auto">
        <div class="flex items-center gap-2">
          <ToggleSwitch v-model="showSystemNamespaces" inputId="system-ns-toggle" />
          <label
            for="system-ns-toggle"
            class="text-xs font-semibold text-(--text-secondary) cursor-pointer select-none"
          >
            Show system namespaces
          </label>
        </div>

        <div class="flex items-center gap-1">
          <Button
            severity="secondary"
            variant="text"
            size="small"
            class="p-1"
            @click="handleRefresh"
          >
            <RefreshCw class="w-4 h-4 text-(--text-secondary)" />
          </Button>
          <Button severity="secondary" variant="text" size="small" class="p-1">
            <Settings2 class="w-4 h-4 text-(--text-secondary)" />
          </Button>
        </div>
      </div>
    </div>

    <!-- Data Table -->
    <DataTable
      :value="filteredEvents"
      paginator
      :rows="12"
      class="p-datatable-sm border border-(--border) rounded-lg overflow-hidden cursor-pointer"
      tableClass="w-full text-left text-xs border-collapse"
      paginatorTemplate="FirstPageLink PrevPageLink PageLinks NextPageLink LastPageLink RowsPerPageDropdown CurrentPageReport"
      currentPageReportTemplate="Showing {first} to {last} of {totalRecords} events"
      @row-click="onRowClick"
    >
      <template #empty>
        <div class="text-center py-10 text-(--text-muted) flex flex-col items-center gap-2">
          <Info class="w-8 h-8 text-(--text-muted)/50" />
          <span>No events found matching the filter criteria.</span>
        </div>
      </template>

      <!-- Time Column -->
      <Column
        field="time"
        header="Time"
        sortable
        class="p-3 text-(--text-muted) font-mono min-w-16"
      >
        <template #body="{ data }">
          <span>{{ data.time }}</span>
        </template>
      </Column>

      <!-- Type Column -->
      <Column field="type" header="Type" sortable class="p-3 min-w-24">
        <template #body="{ data }">
          <span
            class="px-2 py-0.5 rounded text-[10px] font-semibold tracking-wider font-ui border"
            :class="getTypeBadgeClass(data.type)"
          >
            {{ data.type }}
          </span>
        </template>
      </Column>

      <!-- Reason Column -->
      <Column
        field="reason"
        header="Reason"
        sortable
        class="p-3 font-semibold text-(--text-primary)"
      >
        <template #body="{ data }">
          <span class="font-mono">{{ data.reason }}</span>
        </template>
      </Column>

      <!-- Object Column -->
      <Column field="objectName" header="Object" sortable class="p-3">
        <template #body="{ data }">
          <div class="flex flex-col">
            <span class="text-[10px] text-(--text-muted) uppercase font-semibold">{{
              data.objectKind
            }}</span>
            <span
              class="font-mono text-violet-400 hover:text-violet-300 transition-colors truncate max-w-48"
              :title="data.objectName"
            >
              {{ data.objectName }}
            </span>
          </div>
        </template>
      </Column>

      <!-- Message Column -->
      <Column field="message" header="Message" class="p-3 max-w-xs md:max-w-md">
        <template #body="{ data }">
          <span class="text-(--text-secondary) block truncate" :title="data.message">
            {{ data.message }}
          </span>
        </template>
      </Column>

      <!-- Namespace Column -->
      <Column field="namespace" header="Namespace" sortable class="p-3">
        <template #body="{ data }">
          <span class="font-mono text-(--text-muted)">{{ data.namespace }}</span>
        </template>
      </Column>

      <!-- Source Column -->
      <Column
        field="source"
        header="Source"
        sortable
        class="p-3 font-mono text-(--text-muted)"
      ></Column>

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

    <!-- Details Drawer -->
    <EventDetailsDrawer v-model:visible="drawerVisible" :event="selectedEvent" />
    <ResourceActionMenu ref="actionMenu" :items="actionMenuItems" />
  </div>
</template>
