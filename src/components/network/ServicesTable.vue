<script setup lang="ts">
import GenericResourceTable from '@/components/shared/GenericResourceTable.vue'
import TableFilterSelect from '@/components/shared/TableFilterSelect.vue'
import { kubernetesService } from '@/services/kubernetesService'
import { useKubernetesStore } from '@/stores/kubernetesStore'
import { ExternalLink } from '@lucide/vue'
import Column from 'primevue/column'
import { computed, ref } from 'vue'
import ServiceDetailsDrawer from '@/components/services/ServiceDetailsDrawer.vue'

const k8sStore = useKubernetesStore()

const columns = [
  { field: 'namespace', header: 'Namespace', visible: true },
  { field: 'type', header: 'Type', visible: true },
  { field: 'clusterIP', header: 'Cluster IP', visible: true },
  { field: 'externalIP', header: 'External IP', visible: true },
  { field: 'ports', header: 'Ports', visible: true },
  { field: 'endpoints', header: 'Endpoints', visible: true },
  { field: 'age', header: 'Age', visible: true }
]

const selectedType = ref('All Types')
const types = ['All Types', 'ClusterIP', 'NodePort', 'LoadBalancer', 'ExternalName']

const filteredServices = computed(() => {
  return k8sStore.services.filter((s) => {
    if (selectedType.value !== 'All Types' && s.type !== selectedType.value) {
      return false
    }
    return true
  })
})

const handleRefresh = async () => {
  await kubernetesService.getServices()
}

const getTypeSeverity = (type: string) => {
  switch (type) {
    case 'LoadBalancer':
      return 'info'
    case 'ClusterIP':
      return 'success'
    case 'NodePort':
      return 'warn'
    case 'ExternalName':
      return 'contrast'
    default:
      return 'secondary'
  }
}
</script>

<template>
  <GenericResourceTable
    :data="filteredServices"
    :initialColumns="columns"
    :hideStatusFilter="true"
    :hideStatusColumn="true"
    :searchFields="['name', 'clusterIP', 'externalIP']"
    kind="Service"
    searchPlaceholder="Search services..."
    emptyMessage="No services found matching the filter criteria."
    reportTemplate="Showing {first} to {last} of {totalRecords} services"
    :loading="k8sStore.servicesLoading"
    @refresh="handleRefresh"
  >
    <!-- Filter -->
    <template #filters>
      <TableFilterSelect v-model="selectedType" :options="types" />
    </template>

    <template #default="{ visibleCols }">
      <!-- Type Column -->
      <Column v-if="visibleCols['type']" field="type" header="Type" sortable class="p-3">
        <template #body="{ data }">
          <Tag :severity="getTypeSeverity(data.type)" :value="data.type" />
        </template>
      </Column>

      <!-- Cluster IP Column -->
      <Column
        v-if="visibleCols['clusterIP']"
        field="clusterIP"
        header="Cluster IP"
        sortable
        class="p-3"
      >
        <template #body="{ data }">
          <span class="font-mono text-muted-color">{{ data.clusterIP }}</span>
        </template>
      </Column>

      <!-- External IP Column -->
      <Column
        v-if="visibleCols['externalIP']"
        field="externalIP"
        header="External IP"
        sortable
        class="p-3"
      >
        <template #body="{ data }">
          <div class="flex items-center gap-1">
            <span class="font-mono text-muted-color">{{ data.externalIP }}</span>
            <ExternalLink
              v-if="data.externalIP !== '-'"
              class="w-3 h-3 text-violet-400 hover:text-violet-300 cursor-pointer"
            />
          </div>
        </template>
      </Column>

      <!-- Ports Column -->
      <Column v-if="visibleCols['ports']" field="ports" header="Ports" sortable class="p-3">
        <template #body="{ data }">
          <span class="font-mono text-muted-color whitespace-pre-line">{{ data.ports }}</span>
        </template>
      </Column>

      <!-- Endpoints Column -->
      <Column
        v-if="visibleCols['endpoints']"
        field="endpoints"
        header="Endpoints"
        sortable
        class="p-3"
      >
        <template #body="{ data }">
          <span class="font-mono font-medium text-emerald-400">{{ data.endpoints }}</span>
        </template>
      </Column>
    </template>

    <!-- Drawer -->
    <template #drawer="{ selectedItem, visible, close }">
      <ServiceDetailsDrawer
        :visible="visible"
        :service="selectedItem"
        @update:visible="!$event && close()"
      />
    </template>
  </GenericResourceTable>
</template>
