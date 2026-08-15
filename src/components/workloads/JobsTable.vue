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
  { field: 'completions', header: 'Completions', visible: true },
  { field: 'duration', header: 'Duration', visible: true },
  { field: 'age', header: 'Age', visible: true },
  { field: 'images', header: 'Images', visible: true }
]

const statuses = ['All Statuses', 'Active', 'Succeeded', 'Failed', 'Unknown']

const fetchJobs = async (namespace?: string) => {
  loading.value = true
  try {
    await kubernetesService.getJobs(namespace)
  } catch (e) {
    console.error('Error fetching jobs', e)
  } finally {
    loading.value = false
  }
}

onMounted(() => {
  if (k8sStore.jobs.length === 0 && !k8sStore.jobsLoading) {
    fetchJobs()
  }
})

watch(
  () => k8sStore.activeClusterId,
  () => {
    fetchJobs()
  }
)
</script>

<template>
  <GenericResourceTable
    :data="k8sStore.jobs"
    :initialColumns="columns"
    :statuses="statuses"
    :searchFields="['name', 'images']"
    kind="Job"
    searchPlaceholder="Search jobs or images..."
    emptyMessage="No jobs found matching the filter criteria."
    reportTemplate="Showing {first} to {last} of {totalRecords} jobs"
    :loading="loading || k8sStore.jobsLoading"
    @refresh="fetchJobs"
    @namespace-change="fetchJobs"
  >
    <template #default="{ visibleCols }">
      <!-- Completions Column -->
      <Column
        v-if="visibleCols['completions']"
        field="completions"
        header="Completions"
        sortable
        class="p-3"
        bodyClass="font-mono text-muted-color"
      ></Column>

      <!-- Duration Column -->
      <Column
        v-if="visibleCols['duration']"
        field="duration"
        header="Duration"
        sortable
        class="p-3"
        bodyClass="font-mono text-muted-color"
      >
        <template #body="{ data }">
          <span>{{ data.duration || '-' }}</span>
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
