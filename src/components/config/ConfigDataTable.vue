<script setup lang="ts">
import NamespaceBadge from '@/components/shared/NamespaceBadge.vue'
import NamespaceFilter from '@/components/shared/NamespaceFilter.vue'
import ResourceDataTable from '@/components/shared/ResourceDataTable.vue'
import SystemNamespaceToggle from '@/components/shared/SystemNamespaceToggle.vue'
import { useResourceFilters } from '@/composables/useResourceFilters'
import { useTableColumns } from '@/composables/useTableColumns'
import { useKubernetesStore } from '@/stores/kubernetesStore'
import type { ConfigMapInfo, SecretInfo } from '@/types/kubernetes'
import { FileText, Lock, MoreVertical } from '@lucide/vue'
import { storeToRefs } from 'pinia'
import Column from 'primevue/column'
import Select from 'primevue/select'
import { useToast } from 'primevue/usetoast'
import { computed, ref, watch } from 'vue'
import ConfigDetailsDrawer from './ConfigDetailsDrawer.vue'

const toast = useToast()

const props = defineProps<{
  activeTab: 'configmaps' | 'secrets'
}>()

const k8sStore = useKubernetesStore()
const { configMaps, secrets } = storeToRefs(k8sStore)

const { tableColumns, visibleCols } = useTableColumns([
  { field: 'namespace', header: 'Namespace', visible: true },
  { field: 'labels', header: 'Labels', visible: true },
  { field: 'type', header: 'Type', visible: true },
  { field: 'keysCount', header: 'Data Keys', visible: true },
  { field: 'size', header: 'Size', visible: true },
  { field: 'mountedPods', header: 'Mounted In', visible: true },
  { field: 'age', header: 'Age', visible: true }
])

// Filter out 'Type' column from configuration checkboxes when in configmaps tab
const columnsForConfig = computed(() => {
  if (props.activeTab === 'configmaps') {
    return tableColumns.value.filter((col) => col.field !== 'type')
  }
  return tableColumns.value
})

const handleRefresh = async () => {
  if (props.activeTab === 'configmaps') {
    await k8sStore.fetchConfigMaps()
  } else {
    await k8sStore.fetchSecrets()
  }
}

const { searchQuery, selectedNamespace, showSystemNamespaces, filteredResources } =
  useResourceFilters(
    computed(() => (props.activeTab === 'configmaps' ? configMaps.value : secrets.value))
  )

const selectedLabel = ref('All Labels')

// Drawer state
const drawerVisible = ref(false)
const selectedResource = ref<ConfigMapInfo | SecretInfo | null>(null)

// Reset filters on tab switch
watch(
  () => props.activeTab,
  () => {
    searchQuery.value = ''
    selectedNamespace.value = []
    selectedLabel.value = 'All Labels'
  }
)

const namespaces = computed(() => {
  const currentList = props.activeTab === 'configmaps' ? configMaps.value : secrets.value
  const list = new Set(currentList.map((item) => item.namespace))
  return ['All Namespaces', ...Array.from(list)]
})

const labels = computed(() => {
  const currentList = props.activeTab === 'configmaps' ? configMaps.value : secrets.value
  const labelKeys = new Set<string>()
  currentList.forEach((item) => {
    Object.keys(item.labels).forEach((key) => labelKeys.add(key))
  })
  return ['All Labels', ...Array.from(labelKeys)]
})

const filteredItems = computed(() => {
  return filteredResources.value.filter((item) => {
    // Label filter
    if (selectedLabel.value !== 'All Labels') {
      const hasLabelKey = Object.keys(item.labels).includes(selectedLabel.value)
      if (!hasLabelKey) return false
    }

    return true
  })
})

const onRowClick = (event: { data: ConfigMapInfo | SecretInfo }) => {
  selectedResource.value = event.data
  drawerVisible.value = true
}

const handleActionClick = (event: Event, action: string, resourceName: string) => {
  event.stopPropagation()
  toast.add({
    severity: 'info',
    summary: action,
    detail: `Action triggered for: ${resourceName}`,
    life: 3000
  })
}
</script>

<template>
  <ResourceDataTable
    :data="filteredItems"
    v-model:searchQuery="searchQuery"
    :columns="columnsForConfig"
    @update:columns="tableColumns = $event"
    :searchPlaceholder="
      props.activeTab === 'configmaps' ? 'Search configmaps...' : 'Search secrets...'
    "
    :emptyMessage="`No ${props.activeTab === 'configmaps' ? 'configmaps' : 'secrets'} found matching the filter criteria.`"
    :reportTemplate="
      props.activeTab === 'configmaps'
        ? 'Showing {first} to {last} of {totalRecords} configmaps'
        : 'Showing {first} to {last} of {totalRecords} secrets'
    "
    :rows="12"
    @refresh="handleRefresh"
    @row-click="onRowClick"
  >
    <!-- Filters -->
    <template #filters>
      <!-- Namespace Select -->
      <NamespaceFilter v-model="selectedNamespace" :namespaces="namespaces" />

      <!-- Label Select -->
      <Select
        v-model="selectedLabel"
        :options="labels"
        class="text-xs min-w-44 bg-(--bg-hover)/30 border-(--border)"
      />
    </template>

    <!-- Actions Left -->
    <template #actions-left>
      <SystemNamespaceToggle v-model="showSystemNamespaces" />
    </template>

    <!-- Columns -->
    <!-- Name Column -->
    <Column field="name" header="Name" sortable class="font-medium p-3 text-(--text-primary)">
      <template #body="{ data }">
        <div class="flex items-center gap-2">
          <FileText v-if="props.activeTab === 'configmaps'" class="w-4 h-4 text-sky-400" />
          <Lock v-else class="w-4 h-4 text-rose-400" />
          <span class="font-semibold hover:text-violet-400 transition-colors">{{ data.name }}</span>
        </div>
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
        <NamespaceBadge :namespace="data.namespace" />
      </template>
    </Column>

    <!-- Labels Column -->
    <Column v-if="visibleCols['labels']" field="labels" header="Labels" class="p-3">
      <template #body="{ data }">
        <div class="flex flex-wrap gap-1 max-w-72">
          <span
            v-for="(val, key) in data.labels"
            :key="key"
            class="font-mono text-[9px] bg-(--bg-hover) text-(--text-secondary) border border-(--border) px-1.5 py-0.5 rounded"
          >
            {{ key }}: {{ val }}
          </span>
        </div>
      </template>
    </Column>

    <!-- Secret Type Column (Only for Secrets) -->
    <Column
      v-if="props.activeTab === 'secrets' && visibleCols['type']"
      field="type"
      header="Type"
      sortable
      class="p-3"
    >
      <template #body="{ data }">
        <span class="font-mono text-(--text-secondary)">{{ data.type }}</span>
      </template>
    </Column>

    <!-- Data Keys Column -->
    <Column
      v-if="visibleCols['keysCount']"
      field="keysCount"
      header="Data Keys"
      sortable
      class="p-3 text-center"
    >
      <template #body="{ data }">
        <span class="font-mono text-(--text-primary)">{{ data.keysCount }}</span>
      </template>
    </Column>

    <!-- Size Column -->
    <Column
      v-if="visibleCols['size']"
      field="size"
      header="Size"
      sortable
      class="p-3 text-(--text-muted) font-mono"
    ></Column>

    <!-- Mounted In Column -->
    <Column
      v-if="visibleCols['mountedPods']"
      field="mountedPods"
      header="Mounted In"
      sortable
      class="p-3 text-center"
    >
      <template #body="{ data }">
        <span
          class="font-mono font-semibold"
          :class="data.mountedPods > 0 ? 'text-emerald-400' : 'text-(--text-muted)'"
        >
          {{ data.mountedPods }} {{ data.mountedPods === 1 ? 'pod' : 'pods' }}
        </span>
      </template>
    </Column>

    <!-- Age Column -->
    <Column
      v-if="visibleCols['age']"
      field="age"
      header="Age"
      sortable
      class="p-3 text-(--text-muted) font-mono"
    ></Column>

    <!-- Actions Column -->
    <Column class="p-3 text-center">
      <template #body="{ data }">
        <Button
          severity="secondary"
          variant="text"
          size="small"
          class="p-1"
          title="Actions"
          @click="handleActionClick($event, 'Show Actions Menu', data.name)"
        >
          <MoreVertical class="w-4 h-4 text-(--text-muted)" />
        </Button>
      </template>
    </Column>

    <!-- Drawer -->
    <template #drawer>
      <ConfigDetailsDrawer v-model:visible="drawerVisible" :resource="selectedResource" />
    </template>
  </ResourceDataTable>
</template>
