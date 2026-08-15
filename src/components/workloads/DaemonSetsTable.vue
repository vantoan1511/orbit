<script setup lang="ts">
import GenericResourceTable from '@/components/shared/GenericResourceTable.vue'
import { kubernetesService } from '@/services/kubernetesService'
import { useKubernetesStore } from '@/stores/kubernetesStore'
import Column from 'primevue/column'
import { onMounted, ref, watch } from 'vue'
import WorkloadDetailsDrawer from './WorkloadDetailsDrawer.vue'

const k8sStore = useKubernetesStore()
const loading = ref(false)

const columns = [
  { field: 'namespace', header: 'Namespace', visible: true },
  { field: 'status', header: 'Status', visible: true },
  { field: 'desiredCurrent', header: 'Desired/Current', visible: true },
  { field: 'ready', header: 'Ready', visible: true },
  { field: 'available', header: 'Available', visible: true },
  { field: 'age', header: 'Age', visible: true },
  { field: 'images', header: 'Images', visible: true }
]

const statuses = ['All Statuses', 'Running', 'Progressing']

const fetchDaemonSets = async (namespace?: string) => {
  loading.value = true
  try {
    await kubernetesService.getDaemonSets(namespace)
  } catch (e) {
    console.error('Error fetching daemonsets', e)
  } finally {
    loading.value = false
  }
}

onMounted(() => {
  if (k8sStore.daemonSets.length === 0 && !k8sStore.daemonSetsLoading) {
    fetchDaemonSets()
  }
})

watch(
  () => k8sStore.activeClusterId,
  () => {
    fetchDaemonSets()
  }
)
</script>

<template>
  <GenericResourceTable
    :data="k8sStore.daemonSets"
    :initialColumns="columns"
    :statuses="statuses"
    :searchFields="['name', 'images']"
    kind="DaemonSet"
    searchPlaceholder="Search daemonsets or images..."
    emptyMessage="No daemonsets found matching the filter criteria."
    reportTemplate="Showing {first} to {last} of {totalRecords} daemonsets"
    :loading="loading || k8sStore.daemonSetsLoading"
    @refresh="fetchDaemonSets"
    @namespace-change="fetchDaemonSets"
  >
    <template #default="{ visibleCols }">
      <!-- Desired/Current Column -->
      <Column v-if="visibleCols['desiredCurrent']" header="Desired/Current" class="p-3">
        <template #body="{ data }">
          <span class="font-mono text-muted-color">
            {{ data.replicas.desired }} / {{ data.replicas.current }}
          </span>
        </template>
      </Column>

      <!-- Ready Column -->
      <Column v-if="visibleCols['ready']" header="Ready" class="p-3">
        <template #body="{ data }">
          <span class="font-mono text-emerald-500">{{ data.replicas.ready }}</span>
        </template>
      </Column>

      <!-- Available Column -->
      <Column v-if="visibleCols['available']" header="Available" class="p-3">
        <template #body="{ data }">
          <span class="font-mono text-emerald-400">{{ data.replicas.available }}</span>
        </template>
      </Column>

      <!-- Images Column -->
      <Column v-if="visibleCols['images']" header="Images" class="p-3 max-w-48">
        <template #body="{ data }">
          <div class="flex flex-wrap gap-1">
            <span
              v-for="img in data.images"
              :key="img"
              class="px-1.5 py-0.5 rounded bg-(--bg-hover) text-muted-color text-[10px] border border-(--border) font-mono truncate max-w-full"
              :title="img"
            >
              {{ img.split('/').pop() }}
            </span>
          </div>
        </template>
      </Column>
    </template>

    <!-- Drawer -->
    <template #drawer="{ selectedItem, visible, close }">
      <WorkloadDetailsDrawer
        :visible="visible"
        :workload="selectedItem"
        @update:visible="!$event && close()"
      />
    </template>
  </GenericResourceTable>
</template>
