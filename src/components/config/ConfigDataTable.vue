<script setup lang="ts">
import GenericResourceTable from '@/components/shared/GenericResourceTable.vue'
import TableFilterSelect from '@/components/shared/TableFilterSelect.vue'
import { useKubernetesStore } from '@/stores/kubernetesStore'
import { FileText, Lock } from '@lucide/vue'
import { storeToRefs } from 'pinia'
import { useTableFilterStore } from '@/stores/tableFilterStore'
import { computed } from 'vue'
import ConfigDetailsDrawer from './ConfigDetailsDrawer.vue'

const props = defineProps<{
  activeTab: 'configmaps' | 'secrets'
}>()

const k8sStore = useKubernetesStore()
const filterStore = useTableFilterStore()
const { configMaps, secrets } = storeToRefs(k8sStore)

const columns = [
  { field: 'namespace', header: 'Namespace', visible: true },
  { field: 'labels', header: 'Labels', visible: true },
  { field: 'type', header: 'Type', visible: true },
  { field: 'keysCount', header: 'Data Keys', visible: true },
  { field: 'size', header: 'Size', visible: true },
  { field: 'mountedPods', header: 'Mounted In', visible: true },
  { field: 'age', header: 'Age', visible: true }
]

const columnsForConfig = computed(() => {
  if (props.activeTab === 'configmaps') {
    return columns.filter((col) => col.field !== 'type')
  }
  return columns
})

const filterKey = computed(() => (props.activeTab === 'configmaps' ? 'configmap' : 'secret'))

const selectedLabel = computed({
  get: () => filterStore.getExtraFilter(filterKey.value, 'label', 'All Labels'),
  set: (val: string) => filterStore.setExtraFilter(filterKey.value, 'label', val)
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
  const currentList = props.activeTab === 'configmaps' ? configMaps.value : secrets.value
  return currentList.filter((item) => {
    if (selectedLabel.value !== 'All Labels') {
      const hasLabelKey = Object.keys(item.labels).includes(selectedLabel.value)
      if (!hasLabelKey) return false
    }
    return true
  })
})

const activeKind = computed(() => (props.activeTab === 'configmaps' ? 'ConfigMap' : 'Secret'))

const handleRefresh = async () => {
  if (props.activeTab === 'configmaps') {
    await k8sStore.fetchConfigMaps()
  } else {
    await k8sStore.fetchSecrets()
  }
}
</script>

<template>
  <GenericResourceTable
    :data="filteredItems"
    :initialColumns="columnsForConfig"
    :hideStatusFilter="true"
    :hideStatusColumn="true"
    :kind="activeKind"
    :searchPlaceholder="
      props.activeTab === 'configmaps' ? 'Search configmaps...' : 'Search secrets...'
    "
    :emptyMessage="`No ${props.activeTab === 'configmaps' ? 'configmaps' : 'secrets'} found matching the filter criteria.`"
    :reportTemplate="
      props.activeTab === 'configmaps'
        ? 'Showing {first} to {last} of {totalRecords} configmaps'
        : 'Showing {first} to {last} of {totalRecords} secrets'
    "
    :loading="
      props.activeTab === 'configmaps' ? k8sStore.configMapsLoading : k8sStore.secretsLoading
    "
    @refresh="handleRefresh"
  >
    <!-- Filter -->
    <template #filters>
      <TableFilterSelect v-model="selectedLabel" :options="labels" class="min-w-44" />
    </template>

    <!-- Custom Name -->
    <template #name="{ data }">
      <div class="flex items-center gap-2">
        <FileText v-if="props.activeTab === 'configmaps'" class="w-4 h-4 text-configmap" />
        <Lock v-else class="w-4 h-4 text-secret" />
        <span class="font-semibold">{{ data.name }}</span>
      </div>
    </template>

    <template #default="{ visibleCols }">
      <!-- Labels Column -->
      <Column v-if="visibleCols['labels']" field="labels" header="Labels" class="p-3">
        <template #body="{ data }">
          <div class="flex flex-wrap gap-1 max-w-72">
            <Tag
              v-for="(val, key) in data.labels"
              :key="key"
              severity="secondary"
              class="font-mono"
              :value="`${key}: ${val}`"
            />
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
          <span class="font-mono text-muted-color">{{ data.type }}</span>
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
          <span class="font-mono text-primary">{{ data.keysCount }}</span>
        </template>
      </Column>

      <!-- Size Column -->
      <Column
        v-if="visibleCols['size']"
        field="size"
        header="Size"
        sortable
        class="p-3"
        bodyClass="text-muted-color font-mono"
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
            :class="data.mountedPods > 0 ? 'text-emerald-400' : 'text-muted-color'"
          >
            {{ data.mountedPods }} {{ data.mountedPods === 1 ? 'pod' : 'pods' }}
          </span>
        </template>
      </Column>
    </template>

    <!-- Drawer -->
    <template #drawer="{ selectedItem, visible, close }">
      <ConfigDetailsDrawer
        :visible="visible"
        :resource="selectedItem"
        @update:visible="!$event && close()"
      />
    </template>
  </GenericResourceTable>
</template>
