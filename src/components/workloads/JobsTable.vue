<script setup lang="ts">
import GenericResourceTable from '@/components/shared/GenericResourceTable.vue'
import { KUBERNETES_JOB_STATUS, KUBERNETES_RESOURCE_KIND } from '@/constants/kubernetes'
import { kubernetesService } from '@/services/kubernetesService'
import { useKubernetesStore } from '@/stores/kubernetesStore'
import Column from 'primevue/column'
import { ref } from 'vue'
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

const statuses = [
  'All Statuses',
  KUBERNETES_JOB_STATUS.Active,
  KUBERNETES_JOB_STATUS.Succeeded,
  KUBERNETES_JOB_STATUS.Failed,
  KUBERNETES_JOB_STATUS.Unknown
]

const fetchJobs = async () => {
  loading.value = true
  try {
    await kubernetesService.getJobs()
  } catch (e) {
    console.error('Error fetching jobs', e)
  } finally {
    loading.value = false
  }
}
</script>

<template>
  <GenericResourceTable
    :data="k8sStore.jobs"
    :initialColumns="columns"
    :statuses="statuses"
    :searchFields="['name', 'images']"
    :kind="KUBERNETES_RESOURCE_KIND.Job"
    searchPlaceholder="Search jobs or images..."
    emptyMessage="No jobs found matching the filter criteria."
    reportTemplate="Showing {first} to {last} of {totalRecords} jobs"
    :loading="loading || k8sStore.jobsLoading"
    @refresh="fetchJobs"
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
            <Tag
              v-for="img in data.images"
              :key="img"
              severity="secondary"
              class="font-mono truncate max-w-full"
              :title="img"
              :value="img.split('/').pop()"
            />
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
