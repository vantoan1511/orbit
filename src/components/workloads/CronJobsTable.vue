<script setup lang="ts">
import GenericResourceTable from '@/components/shared/GenericResourceTable.vue'
import TableFilterSelect from '@/components/shared/TableFilterSelect.vue'
import { kubernetesService } from '@/services/kubernetesService'
import { useKubernetesStore } from '@/stores/kubernetesStore'
import Column from 'primevue/column'
import { computed, ref } from 'vue'
import WorkloadDetailsDrawer from './WorkloadDetailsDrawer.vue'

const k8sStore = useKubernetesStore()
const loading = ref(false)

const columns = [
  { field: 'namespace', header: 'Namespace', visible: true },
  { field: 'schedule', header: 'Schedule', visible: true },
  { field: 'suspend', header: 'Suspend', visible: true },
  { field: 'active', header: 'Active Jobs', visible: true },
  { field: 'lastSchedule', header: 'Last Schedule', visible: true },
  { field: 'age', header: 'Age', visible: true }
]

const selectedSuspend = ref('All Suspend States')
const suspendOptions = ['All Suspend States', 'Suspended', 'Active']

const filteredCronJobs = computed(() => {
  return k8sStore.cronJobs.filter((cj) => {
    if (selectedSuspend.value !== 'All Suspend States') {
      const isSuspended = selectedSuspend.value === 'Suspended'
      if (cj.suspend !== isSuspended) return false
    }
    return true
  })
})

const fetchCronJobs = async () => {
  loading.value = true
  try {
    await kubernetesService.getCronJobs()
  } catch (e) {
    console.error('Error fetching cronjobs', e)
  } finally {
    loading.value = false
  }
}
</script>

<template>
  <GenericResourceTable
    :data="filteredCronJobs"
    :initialColumns="columns"
    :hideStatusFilter="true"
    :hideStatusColumn="true"
    :searchFields="['name', 'images']"
    kind="CronJob"
    searchPlaceholder="Search cronjobs or images..."
    emptyMessage="No cronjobs found matching the filter criteria."
    reportTemplate="Showing {first} to {last} of {totalRecords} cronjobs"
    :loading="loading || k8sStore.cronJobsLoading"
    @refresh="fetchCronJobs"
  >
    <!-- Custom Filter -->
    <template #filters>
      <TableFilterSelect v-model="selectedSuspend" :options="suspendOptions" class="min-w-44" />
    </template>

    <template #default="{ visibleCols }">
      <!-- Schedule Column -->
      <Column
        v-if="visibleCols['schedule']"
        field="schedule"
        header="Schedule"
        sortable
        class="p-3"
        bodyClass="font-mono text-primary"
      ></Column>

      <!-- Suspend Column -->
      <Column v-if="visibleCols['suspend']" field="suspend" header="Suspend" sortable class="p-3">
        <template #body="{ data }">
          <span
            class="font-medium px-2 py-0.5 rounded text-[10px] uppercase tracking-wider"
            :class="
              data.suspend
                ? 'bg-amber-500/10 text-amber-400 border border-amber-500/20'
                : 'bg-emerald-500/10 text-emerald-400 border border-emerald-500/20'
            "
          >
            {{ data.suspend ? 'True' : 'False' }}
          </span>
        </template>
      </Column>

      <!-- Active Column -->
      <Column
        v-if="visibleCols['active']"
        field="active"
        header="Active Jobs"
        sortable
        class="p-3 text-center"
      >
        <template #body="{ data }">
          <span class="font-mono text-muted-color">{{ data.active }}</span>
        </template>
      </Column>

      <!-- Last Schedule Column -->
      <Column
        v-if="visibleCols['lastSchedule']"
        field="lastSchedule"
        header="Last Schedule"
        sortable
        class="p-3"
        bodyClass="font-mono text-muted-color"
      >
        <template #body="{ data }">
          <span>{{ data.lastSchedule || '-' }}</span>
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
