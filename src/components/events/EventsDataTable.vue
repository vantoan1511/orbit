<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import Column from 'primevue/column'
import Select from 'primevue/select'
import Button from 'primevue/button'
import { MoreVertical } from '@lucide/vue'
import type { EventInfo } from '@/types/kubernetes'
import EventDetailsDrawer from './EventDetailsDrawer.vue'
import { useKubernetesStore } from '@/stores/kubernetesStore'
import { storeToRefs } from 'pinia'
import ResourceActionMenu from '@/components/shared/ResourceActionMenu.vue'
import { useResourceActionMenu } from '@/composables/useResourceActionMenu'
import ResourceDataTable from '@/components/shared/ResourceDataTable.vue'
import NamespaceFilter from '@/components/shared/NamespaceFilter.vue'
import SystemNamespaceToggle from '@/components/shared/SystemNamespaceToggle.vue'
import { useWorkloadActions } from '@/composables/useWorkloadActions'
import { useResourceFilters } from '@/composables/useResourceFilters'
import { useTableColumns } from '@/composables/useTableColumns'

const k8sStore = useKubernetesStore()
const { events } = storeToRefs(k8sStore)

const { tableColumns, visibleCols } = useTableColumns([
  { field: 'time', header: 'Time', visible: true },
  { field: 'type', header: 'Type', visible: true },
  { field: 'reason', header: 'Reason', visible: true },
  { field: 'objectName', header: 'Object', visible: true },
  { field: 'message', header: 'Message', visible: true },
  { field: 'namespace', header: 'Namespace', visible: true },
  { field: 'source', header: 'Source', visible: true }
])

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

const eventsWithResourceItem = computed(() =>
  events.value.map((e) => ({ ...e, name: e.objectName }))
)

const { searchQuery, selectedNamespace, showSystemNamespaces, filteredResources } =
  useResourceFilters(eventsWithResourceItem, ['message', 'reason', 'name', 'objectKind', 'source'])

const selectedType = ref('All Types')
const drawerVisible = ref(false)
const selectedEvent = ref<EventInfo | null>(null)

const namespaces = computed(() => k8sStore.namespaces)
const types = ['All Types', 'Normal', 'Warning', 'Error']

const filteredEvents = computed(() => {
  return filteredResources.value.filter((e) => {
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

const actionMenuRef = useResourceActionMenu<EventInfo & { name: string }>()
const actionMenu = actionMenuRef.actionMenu
const selectedActionRow = actionMenuRef.selectedActionRow

const toggleActionMenu = (event: Event, data: EventInfo) => {
  event.stopPropagation()
  selectedActionRow.value = { ...data, name: data.objectName }
  actionMenu.value?.toggle(event)
}

const onRowContextMenu = (event: { originalEvent: Event; data: EventInfo }) => {
  event.originalEvent?.stopPropagation()
  event.originalEvent?.preventDefault()
  selectedActionRow.value = { ...event.data, name: event.data.objectName }
  actionMenu.value?.show(event.originalEvent)
}

const { actionMenuItems } = useWorkloadActions(selectedActionRow, {
  kind: 'Event'
})
</script>

<template>
  <ResourceDataTable
    :data="filteredEvents"
    v-model:searchQuery="searchQuery"
    v-model:columns="tableColumns"
    searchPlaceholder="Search events..."
    emptyMessage="No events found matching the filter criteria."
    reportTemplate="Showing {first} to {last} of {totalRecords} events"
    :loading="k8sStore.eventsLoading"
    @refresh="handleRefresh"
    @row-click="onRowClick"
    @row-contextmenu="onRowContextMenu"
  >
    <!-- Filters -->
    <template #filters>
      <NamespaceFilter v-model="selectedNamespace" :namespaces="namespaces" />
      <Select v-model="selectedType" :options="types" class="text-xs min-w-40" />
    </template>

    <!-- Actions Left -->
    <template #actions-left>
      <SystemNamespaceToggle v-model="showSystemNamespaces" />
    </template>

    <!-- Columns -->
    <Column
      v-if="visibleCols['time']"
      field="time"
      header="Time"
      sortable
      class="p-3 min-w-16"
      bodyClass="text-muted-color font-mono"
    >
      <template #body="{ data }">
        <span>{{ data.time }}</span>
      </template>
    </Column>

    <Column v-if="visibleCols['type']" field="type" header="Type" sortable class="p-3 min-w-24">
      <template #body="{ data }">
        <span
          class="px-2 py-0.5 rounded text-[10px] font-semibold tracking-wider font-ui border"
          :class="getTypeBadgeClass(data.type)"
        >
          {{ data.type }}
        </span>
      </template>
    </Column>

    <Column
      v-if="visibleCols['reason']"
      field="reason"
      header="Reason"
      sortable
      class="p-3"
      bodyClass="font-semibold text-primary"
    >
      <template #body="{ data }">
        <span class="font-mono">{{ data.reason }}</span>
      </template>
    </Column>

    <Column
      v-if="visibleCols['objectName']"
      field="objectName"
      header="Object"
      sortable
      class="p-3"
    >
      <template #body="{ data }">
        <div class="flex flex-col">
          <span class="text-[10px] text-muted-color uppercase font-semibold">{{
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

    <Column
      v-if="visibleCols['message']"
      field="message"
      header="Message"
      class="p-3 max-w-xs md:max-w-md"
    >
      <template #body="{ data }">
        <span class="text-muted-color block truncate" :title="data.message">
          {{ data.message }}
        </span>
      </template>
    </Column>

    <Column
      v-if="visibleCols['namespace']"
      field="namespace"
      header="Namespace"
      sortable
      class="p-3"
    >
      <template #body="{ data }">
        <span class="font-mono text-muted-color">{{ data.namespace }}</span>
      </template>
    </Column>

    <Column
      v-if="visibleCols['source']"
      field="source"
      header="Source"
      sortable
      class="p-3"
      bodyClass="font-mono text-muted-color"
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
          <MoreVertical class="w-4 h-4 text-muted-color" />
        </Button>
      </template>
    </Column>

    <!-- Drawer -->
    <template #drawer>
      <EventDetailsDrawer v-model:visible="drawerVisible" :event="selectedEvent" />
      <ResourceActionMenu ref="actionMenu" :items="actionMenuItems" />
    </template>
  </ResourceDataTable>
</template>
