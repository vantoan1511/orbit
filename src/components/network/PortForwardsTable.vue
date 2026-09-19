<script setup lang="ts">
import GenericResourceTable from '@/components/shared/GenericResourceTable.vue'
import TableFilterSelect from '@/components/shared/TableFilterSelect.vue'
import { kubernetesService } from '@/services/kubernetesService'
import { os } from '@/services/nativeService'
import { useKubernetesStore } from '@/stores/kubernetesStore'
import type { ActivePortForward } from '@/types/kubernetes'
import Button from 'primevue/button'
import Column from 'primevue/column'
import { useToast } from 'primevue/usetoast'
import { computed, ref } from 'vue'

const k8sStore = useKubernetesStore()
const toast = useToast()

const columns = [
  { field: 'name', header: 'Resource', visible: true },
  { field: 'namespace', header: 'Namespace', visible: true },
  { field: 'kind', header: 'Kind', visible: true },
  { field: 'localPort', header: 'Local Port', visible: true },
  { field: 'remotePort', header: 'Remote Port', visible: true },
  { field: 'status', header: 'Status', visible: true },
  { field: 'actions', header: 'Actions', visible: true }
]

const selectedKind = ref('All Kinds')
const kindFilterOptions = ['All Kinds', 'Service', 'Pod', 'Deployment', 'StatefulSet', 'ReplicaSet']

const filteredPortForwards = computed<ActivePortForward[]>(() => {
  return k8sStore.activePortForwards.filter((pf) => {
    if (
      selectedKind.value !== 'All Kinds' &&
      pf.kind.toLowerCase() !== selectedKind.value.toLowerCase()
    ) {
      return false
    }
    return true
  })
})

const stopping = ref<Record<string, boolean>>({})
const isBulkStopping = ref(false)
const isStoppingAll = ref(false)

const getKindClass = (kindName: string) => {
  switch (kindName.toLowerCase()) {
    case 'pod':
      return 'text-(--pod)'
    case 'service':
      return 'text-(--service)'
    case 'deployment':
      return 'text-(--deployment)'
    case 'statefulset':
      return 'text-(--statefulset)'
    case 'replicaset':
      return 'text-(--replicaset)'
    default:
      return 'text-primary'
  }
}

const openBrowser = async (localPort: number) => {
  try {
    await os.open(`http://127.0.0.1:${localPort}`)
  } catch (e) {
    console.error('Failed to open browser URL', e)
    toast.add({
      severity: 'error',
      summary: 'Error',
      detail: 'Failed to launch browser URL',
      life: 3000
    })
  }
}

const stopSinglePortForward = async (id: string, name: string) => {
  stopping.value[id] = true
  try {
    await kubernetesService.stopPortForward({ id })
    toast.add({
      severity: 'info',
      summary: 'Port Forward Stopped',
      detail: `Stopped port forward for ${name}`,
      life: 3000
    })
  } catch (e) {
    console.error('Failed to stop port forward', e)
    toast.add({
      severity: 'error',
      summary: 'Error',
      detail: e instanceof Error ? e.message : 'Failed to stop port forward',
      life: 5000
    })
  } finally {
    stopping.value[id] = false
  }
}

const stopSelectedPortForwards = async (
  selection: ActivePortForward[],
  clearSelection: () => void
) => {
  if (selection.length === 0) return

  isBulkStopping.value = true
  try {
    for (const item of selection) {
      stopping.value[item.id] = true
      try {
        await kubernetesService.stopPortForward({ id: item.id })
      } finally {
        stopping.value[item.id] = false
      }
    }
    clearSelection()
    toast.add({
      severity: 'info',
      summary: 'Port Forwards Stopped',
      detail: `Stopped ${selection.length} port forward(s)`,
      life: 3000
    })
  } catch (e) {
    console.error('Failed to stop selected port forwards', e)
    toast.add({
      severity: 'error',
      summary: 'Error',
      detail: 'Failed to stop some port forwards',
      life: 5000
    })
  } finally {
    isBulkStopping.value = false
  }
}

const stopAllPortForwards = async () => {
  isStoppingAll.value = true
  try {
    await kubernetesService.stopPortForward()
    toast.add({
      severity: 'info',
      summary: 'All Port Forwards Stopped',
      detail: 'Stopped all active port forwards for this cluster',
      life: 3000
    })
  } catch (e) {
    console.error('Failed to stop all port forwards', e)
    toast.add({
      severity: 'error',
      summary: 'Error',
      detail: e instanceof Error ? e.message : 'Failed to stop all port forwards',
      life: 5000
    })
  } finally {
    isStoppingAll.value = false
  }
}

const handleRefresh = async () => {
  try {
    await kubernetesService.getPortForwards()
  } catch (e) {
    console.error('Failed to refresh port forwards', e)
  }
}
</script>

<template>
  <GenericResourceTable
    :data="filteredPortForwards"
    :dataKey="'id'"
    :initialColumns="columns"
    :hideStatusFilter="true"
    :hideStatusColumn="true"
    :hideAgeColumn="true"
    :hideActionsColumn="true"
    :searchFields="['name', 'kind', 'namespace', 'localPort', 'remotePort']"
    storeKey="port-forward"
    searchPlaceholder="Search port forwards..."
    emptyMessage="No active port forwards. Click 'Forward Port' to start forwarding ports."
    reportTemplate="Showing {first} to {last} of {totalRecords} port forwards"
    @refresh="handleRefresh"
  >
    <!-- Left toolbar extra action -->
    <template #actions-left>
      <Button
        v-if="k8sStore.activePortForwards.length > 0"
        v-tooltip.top="'Stop all active port forwards for this cluster'"
        label="Stop All"
        icon="pi pi-stop-circle"
        severity="danger"
        variant="text"
        size="small"
        :loading="isStoppingAll"
        @click="stopAllPortForwards"
      />
    </template>

    <!-- Filters slot -->
    <template #filters>
      <TableFilterSelect v-model="selectedKind" :options="kindFilterOptions" />
    </template>

    <!-- Bulk actions slot -->
    <template #bulk-actions="{ selection, clearSelection }">
      <Button
        :label="`Stop Selected (${selection.length})`"
        icon="pi pi-stop"
        severity="danger"
        size="small"
        :loading="isBulkStopping"
        @click="stopSelectedPortForwards(selection, clearSelection)"
      />
    </template>

    <!-- Name slot -->
    <template #name="{ data }">
      <div class="flex items-center gap-2 font-mono text-xs font-semibold text-primary">
        <span>{{ data.name }}</span>
      </div>
    </template>

    <!-- Middle custom columns slot -->
    <template #default="{ visibleCols }">
      <!-- Kind Column -->
      <Column v-if="visibleCols['kind']" field="kind" header="Kind" sortable class="p-3">
        <template #body="{ data }">
          <span :class="['font-semibold text-xs capitalize', getKindClass(data.kind)]">
            {{ data.kind }}
          </span>
        </template>
      </Column>

      <!-- Local Port Column -->
      <Column
        v-if="visibleCols['localPort']"
        field="localPort"
        header="Local Port"
        sortable
        class="p-3"
      >
        <template #body="{ data }">
          <div class="flex items-center gap-1.5 font-mono text-xs">
            <span class="text-primary font-semibold">127.0.0.1:{{ data.localPort }}</span>
            <Button
              v-tooltip.top="`Open http://127.0.0.1:${data.localPort} in browser`"
              icon="pi pi-external-link"
              variant="text"
              severity="secondary"
              size="small"
              class="w-5 h-5 p-0 text-muted-color hover:text-primary shrink-0"
              @click.stop="openBrowser(data.localPort)"
            />
          </div>
        </template>
      </Column>

      <!-- Remote Port Column -->
      <Column
        v-if="visibleCols['remotePort']"
        field="remotePort"
        header="Remote Port"
        sortable
        class="p-3"
      >
        <template #body="{ data }">
          <span class="font-mono text-xs text-muted-color">{{ data.remotePort }}</span>
        </template>
      </Column>

      <!-- Status Column -->
      <Column v-if="visibleCols['status']" field="status" header="Status" class="p-3">
        <template #body>
          <div
            class="inline-flex items-center gap-1.5 px-2 py-0.5 rounded-full text-xs font-medium bg-(--success-soft) text-(--success)"
          >
            <span class="w-1.5 h-1.5 rounded-full bg-(--success) animate-pulse" />
            <span>Forwarding</span>
          </div>
        </template>
      </Column>

      <!-- Actions Column -->
      <Column v-if="visibleCols['actions']" header="Actions" class="p-3 text-right">
        <template #body="{ data }">
          <div class="flex items-center justify-end gap-1">
            <Button
              v-tooltip.top="`Open http://127.0.0.1:${data.localPort}`"
              icon="pi pi-external-link"
              variant="text"
              severity="secondary"
              size="small"
              class="w-7 h-7 p-0"
              @click.stop="openBrowser(data.localPort)"
            />
            <Button
              v-tooltip.top="'Stop Port Forward'"
              icon="pi pi-stop"
              severity="danger"
              variant="text"
              size="small"
              class="w-7 h-7 p-0"
              :loading="stopping[data.id]"
              @click.stop="stopSinglePortForward(data.id, data.name)"
            />
          </div>
        </template>
      </Column>
    </template>
  </GenericResourceTable>
</template>
