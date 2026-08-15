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
  { field: 'replicas', header: 'Replicas', visible: true },
  { field: 'age', header: 'Age', visible: true },
  { field: 'images', header: 'Images', visible: true }
]

const statuses = ['All Statuses', 'Running', 'Progressing']

const fetchStatefulSets = async (namespace?: string) => {
  loading.value = true
  try {
    await kubernetesService.getStatefulSets(namespace)
  } catch (e) {
    console.error('Error fetching statefulsets', e)
  } finally {
    loading.value = false
  }
}

onMounted(() => {
  if (k8sStore.statefulSets.length === 0 && !k8sStore.statefulSetsLoading) {
    fetchStatefulSets()
  }
})

watch(
  () => k8sStore.activeClusterId,
  () => {
    fetchStatefulSets()
  }
)
</script>

<template>
  <GenericResourceTable
    :data="k8sStore.statefulSets"
    :initialColumns="columns"
    :statuses="statuses"
    :searchFields="['name', 'images']"
    kind="StatefulSet"
    searchPlaceholder="Search statefulsets or images..."
    emptyMessage="No statefulsets found matching the filter criteria."
    reportTemplate="Showing {first} to {last} of {totalRecords} statefulsets"
    :loading="loading || k8sStore.statefulSetsLoading"
    @refresh="fetchStatefulSets"
    @namespace-change="fetchStatefulSets"
  >
    <template #default="{ visibleCols }">
      <!-- Replicas Column -->
      <Column v-if="visibleCols['replicas']" header="Replicas" class="p-3">
        <template #body="{ data }">
          <div class="flex items-center gap-2 font-mono text-muted-color">
            <span class="font-bold">{{ data.replicas.current }}</span>
            <span class="text-muted-color">/</span>
            <span>{{ data.replicas.desired }}</span>
          </div>
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
