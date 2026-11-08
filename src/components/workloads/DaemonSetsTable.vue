<script setup lang="ts">
import GenericResourceTable from '@/components/shared/GenericResourceTable.vue'
import { useDeferredMount } from '@/composables/useDeferredMount'
import { KUBERNETES_RESOURCE_KIND, KUBERNETES_WORKLOAD_STATUS } from '@/constants/kubernetes'
import { kubernetesService } from '@/services/kubernetesService'
import { useKubernetesStore } from '@/stores/kubernetesStore'
import { Pencil } from '@lucide/vue'
import Column from 'primevue/column'
import Tag from 'primevue/tag'
import { computed, ref } from 'vue'
import WorkloadDetailsDrawer from './WorkloadDetailsDrawer.vue'
import WorkloadInlineEditPopovers from './WorkloadInlineEditPopovers.vue'
import { withReplicasSortKey } from '@/utils/workloadMetrics'

const k8sStore = useKubernetesStore()
const { isHydrated } = useDeferredMount()
const loading = ref(false)
const inlineEdit = ref<InstanceType<typeof WorkloadInlineEditPopovers> | null>(null)

const daemonSets = computed(() =>
  isHydrated.value ? withReplicasSortKey(k8sStore.daemonSets) : []
)

const columns = [
  { field: 'namespace', header: 'Namespace', visible: true },
  { field: 'status', header: 'Status', visible: true },
  { field: 'desiredCurrent', header: 'Desired/Current', visible: true },
  { field: 'ready', header: 'Ready', visible: true },
  { field: 'available', header: 'Available', visible: true },
  { field: 'age', header: 'Age', visible: true },
  { field: 'images', header: 'Images', visible: true }
]

const statuses = [
  'All Statuses',
  KUBERNETES_WORKLOAD_STATUS.Running,
  KUBERNETES_WORKLOAD_STATUS.Progressing
]

const fetchDaemonSets = async () => {
  loading.value = true
  try {
    await kubernetesService.getDaemonSets()
  } catch (e) {
    console.error('Error fetching daemonsets', e)
  } finally {
    loading.value = false
  }
}
</script>

<template>
  <GenericResourceTable
    data-component="daemonsets-table"
    :data="daemonSets"
    :initialColumns="columns"
    :statuses="statuses"
    :searchFields="['name', 'images']"
    :kind="KUBERNETES_RESOURCE_KIND.DaemonSet"
    searchPlaceholder="Search daemonsets or images..."
    emptyMessage="No daemonsets found matching the filter criteria."
    reportTemplate="Showing {first} to {last} of {totalRecords} daemonsets"
    :loading="loading || k8sStore.daemonSetsLoading"
    @refresh="fetchDaemonSets"
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
          <span class="font-mono text-(--success)">{{ data.replicas.ready }}</span>
        </template>
      </Column>

      <!-- Available Column -->
      <Column v-if="visibleCols['available']" header="Available" class="p-3">
        <template #body="{ data }">
          <span class="font-mono text-(--success)">{{ data.replicas.available }}</span>
        </template>
      </Column>

      <!-- Images Column -->
      <Column v-if="visibleCols['images']" header="Images" class="p-3 max-w-48">
        <template #body="{ data }">
          <div
            aria-label="Click to edit container images"
            class="group flex items-center gap-1 max-w-full min-w-0 cursor-pointer"
            @click.stop="inlineEdit?.openImagesEdit($event, data)"
          >
            <div class="flex flex-wrap gap-1 max-w-full min-w-0">
              <Tag
                v-for="img in data.imageTags"
                :key="img.full"
                severity="secondary"
                class="group-hover:text-primary font-mono truncate max-w-full transition-colors inline-block"
                v-tooltip.top="img.full"
                :value="img.short"
              />
            </div>
            <Pencil
              class="w-3 h-3 shrink-0 opacity-0 group-hover:opacity-60 transition-opacity text-muted-color ml-0.5"
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

  <WorkloadInlineEditPopovers ref="inlineEdit" :kind="KUBERNETES_RESOURCE_KIND.DaemonSet" />
</template>
