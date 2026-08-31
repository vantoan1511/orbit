<script setup lang="ts">
import GenericResourceTable from '@/components/shared/GenericResourceTable.vue'
import { KUBERNETES_RESOURCE_KIND } from '@/constants/kubernetes'
import { kubernetesService } from '@/services/kubernetesService'
import { useKubernetesStore } from '@/stores/kubernetesStore'
import Column from 'primevue/column'
import IngressDetailsDrawer from './IngressDetailsDrawer.vue'

const k8sStore = useKubernetesStore()

const columns = [
  { field: 'namespace', header: 'Namespace', visible: true },
  { field: 'className', header: 'Class', visible: true },
  { field: 'hosts', header: 'Hosts', visible: true },
  { field: 'address', header: 'Address', visible: true },
  { field: 'ports', header: 'Ports', visible: true },
  { field: 'age', header: 'Age', visible: true }
]

const handleRefresh = async () => {
  await kubernetesService.getIngresses()
}
</script>

<template>
  <GenericResourceTable
    :data="k8sStore.ingresses"
    :initialColumns="columns"
    :hideStatusFilter="true"
    :hideStatusColumn="true"
    :searchFields="['name', 'hosts', 'address']"
    :kind="KUBERNETES_RESOURCE_KIND.Ingress"
    searchPlaceholder="Search ingresses..."
    emptyMessage="No ingresses found matching the filter criteria."
    reportTemplate="Showing {first} to {last} of {totalRecords} ingresses"
    :loading="k8sStore.ingressesLoading"
    @refresh="handleRefresh"
  >
    <template #default="{ visibleCols }">
      <!-- Class Column -->
      <Column v-if="visibleCols['className']" field="className" header="Class" sortable class="p-3">
        <template #body="{ data }">
          <Tag severity="secondary" class="font-mono" :value="data.className || '-'" />
        </template>
      </Column>

      <!-- Hosts Column -->
      <Column v-if="visibleCols['hosts']" field="hosts" header="Hosts" sortable class="p-3">
        <template #body="{ data }">
          <span class="font-mono text-xs text-violet-400 whitespace-pre-line">{{
            data.hosts
          }}</span>
        </template>
      </Column>

      <!-- Address Column -->
      <Column v-if="visibleCols['address']" field="address" header="Address" sortable class="p-3">
        <template #body="{ data }">
          <span class="font-mono text-xs text-muted-color">{{ data.address }}</span>
        </template>
      </Column>

      <!-- Ports Column -->
      <Column v-if="visibleCols['ports']" field="ports" header="Ports" sortable class="p-3">
        <template #body="{ data }">
          <span class="font-mono text-xs text-muted-color">{{ data.ports }}</span>
        </template>
      </Column>
    </template>

    <!-- Drawer -->
    <template #drawer="{ selectedItem, visible, close }">
      <IngressDetailsDrawer
        :visible="visible"
        :ingress="selectedItem"
        @update:visible="!$event && close()"
      />
    </template>
  </GenericResourceTable>
</template>
