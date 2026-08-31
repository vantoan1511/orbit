<script setup lang="ts">
import GenericResourceTable from '@/components/shared/GenericResourceTable.vue'
import TableFilterSelect from '@/components/shared/TableFilterSelect.vue'
import {
  KUBERNETES_EVENT_TYPE,
  KUBERNETES_RESOURCE_KIND
} from '@/constants/kubernetes'
import { useKubernetesStore } from '@/stores/kubernetesStore'
import { getEventTypeSeverity } from '@/utils/severity'
import { storeToRefs } from 'pinia'
import Column from 'primevue/column'
import { computed, ref } from 'vue'
import EventDetailsDrawer from './EventDetailsDrawer.vue'

const k8sStore = useKubernetesStore()
const { events } = storeToRefs(k8sStore)

const columns = [
  { field: 'time', header: 'Time', visible: true },
  { field: 'type', header: 'Type', visible: true },
  { field: 'reason', header: 'Reason', visible: true },
  { field: 'objectName', header: 'Object', visible: true },
  { field: 'message', header: 'Message', visible: true },
  { field: 'namespace', header: 'Namespace', visible: true },
  { field: 'source', header: 'Source', visible: true }
]

const selectedType = ref('All Types')
const types = [
  'All Types',
  KUBERNETES_EVENT_TYPE.Normal,
  KUBERNETES_EVENT_TYPE.Warning,
  'Error'
]

const eventsWithResourceItem = computed(() =>
  events.value.map((e) => ({ ...e, name: e.objectName }))
)

const filteredEvents = computed(() => {
  return eventsWithResourceItem.value.filter((e) => {
    if (selectedType.value !== 'All Types' && e.type !== selectedType.value) {
      return false
    }
    return true
  })
})

const handleRefresh = async () => {
  try {
    await k8sStore.fetchEvents()
  } catch (error) {
    console.error('Error fetching events:', error)
  }
}
</script>

<template>
  <GenericResourceTable
    :data="filteredEvents"
    :initialColumns="columns"
    :hideStatusFilter="true"
    :hideNameColumn="true"
    :hideNamespaceColumn="true"
    :hideStatusColumn="true"
    :hideAgeColumn="true"
    :searchFields="['message', 'reason', 'name', 'objectKind', 'source']"
    :kind="KUBERNETES_RESOURCE_KIND.Event"
    searchPlaceholder="Search events..."
    emptyMessage="No events found matching the filter criteria."
    reportTemplate="Showing {first} to {last} of {totalRecords} events"
    :loading="k8sStore.eventsLoading"
    @refresh="handleRefresh"
  >
    <!-- Filter -->
    <template #filters>
      <TableFilterSelect v-model="selectedType" :options="types" />
    </template>

    <template #default="{ visibleCols }">
      <!-- Time Column -->
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

      <!-- Type Column -->
      <Column v-if="visibleCols['type']" field="type" header="Type" sortable class="p-3 min-w-24">
        <template #body="{ data }">
          <Tag :severity="getEventTypeSeverity(data.type)" :value="data.type" />
        </template>
      </Column>

      <!-- Reason Column -->
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

      <!-- Object Column -->
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

      <!-- Message Column -->
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

      <!-- Namespace Column -->
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

      <!-- Source Column -->
      <Column
        v-if="visibleCols['source']"
        field="source"
        header="Source"
        sortable
        class="p-3"
        bodyClass="font-mono text-muted-color"
      ></Column>
    </template>

    <!-- Drawer -->
    <template #drawer="{ selectedItem, visible, close }">
      <EventDetailsDrawer
        :visible="visible"
        :event="selectedItem"
        @update:visible="!$event && close()"
      />
    </template>
  </GenericResourceTable>
</template>
