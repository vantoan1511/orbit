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

const statefulSets = computed(() =>
  isHydrated.value ? withReplicasSortKey(k8sStore.statefulSets) : []
)

const columns = [
  { field: 'namespace', header: 'Namespace', visible: true },
  { field: 'status', header: 'Status', visible: true },
  { field: 'replicas', header: 'Replicas', visible: true },
  { field: 'age', header: 'Age', visible: true },
  { field: 'images', header: 'Images', visible: true }
]

const statuses = [
  'All Statuses',
  KUBERNETES_WORKLOAD_STATUS.Running,
  KUBERNETES_WORKLOAD_STATUS.Progressing
]

const fetchStatefulSets = async () => {
  loading.value = true
  try {
    await kubernetesService.getStatefulSets()
  } catch (e) {
    console.error('Error fetching statefulsets', e)
  } finally {
    loading.value = false
  }
}
</script>

<template>
  <GenericResourceTable
    data-component="statefulsets-table"
    :data="statefulSets"
    :initialColumns="columns"
    :statuses="statuses"
    :searchFields="['name', 'images']"
    :kind="KUBERNETES_RESOURCE_KIND.StatefulSet"
    searchPlaceholder="Search statefulsets or images..."
    emptyMessage="No statefulsets found matching the filter criteria."
    reportTemplate="Showing {first} to {last} of {totalRecords} statefulsets"
    :loading="loading || k8sStore.statefulSetsLoading"
    @refresh="fetchStatefulSets"
  >
    <template #default="{ visibleCols }">
      <!-- Replicas Column -->
      <Column
        v-if="visibleCols['replicas']"
        field="replicas"
        sortField="replicasSortKey"
        header="Replicas"
        sortable
        class="p-3"
      >
        <template #body="{ data }">
          <div
            aria-label="Click to edit replicas"
            class="group inline-flex items-center gap-1.5 font-mono text-muted-color px-1.5 py-0.5 rounded-[var(--ui-radius-control)] hover:bg-(--bg-hover) hover:text-primary transition-colors cursor-pointer"
            @click.stop="inlineEdit?.openReplicasEdit($event, data)"
          >
            <span class="font-bold">{{ data.replicas.current }}</span>
            <span class="text-muted-color">/</span>
            <span>{{ data.replicas.desired }}</span>
            <Pencil
              class="w-3 h-3 opacity-0 group-hover:opacity-60 transition-opacity text-muted-color"
            />
          </div>
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

  <WorkloadInlineEditPopovers ref="inlineEdit" :kind="KUBERNETES_RESOURCE_KIND.StatefulSet" />
</template>
