<script setup lang="ts">
import ResourceTabsLayout, { type ResourceTab } from '@/components/shared/ResourceTabsLayout.vue'
import { useDialog } from 'primevue/usedialog'
import { useKubernetesStore } from '@/stores/kubernetesStore'
import { useTableFilterStore } from '@/stores/tableFilterStore'
import { KUBERNETES_RESOURCE_KIND } from '@/constants/kubernetes'
import CreatePVDialog from '@/components/storage/CreatePVDialog.vue'
import CreatePVCDialog from '@/components/storage/CreatePVCDialog.vue'
import CreateStorageClassDialog from '@/components/storage/CreateStorageClassDialog.vue'
import StorageClassesTable from '@/components/storage/StorageClassesTable.vue'
import StorageMetricsCards from '@/components/storage/StorageMetricsCards.vue'
import StorageOverview from '@/components/storage/StorageOverview.vue'
import StoragePVCTable from '@/components/storage/StoragePVCTable.vue'
import StoragePVTable from '@/components/storage/StoragePVTable.vue'

const dialog = useDialog()
const k8sStore = useKubernetesStore()
const filterStore = useTableFilterStore()

const getInitialNamespace = (): string | undefined => {
  const filters = filterStore.getFilters(
    KUBERNETES_RESOURCE_KIND.PersistentVolumeClaim,
    k8sStore.activeClusterId || undefined
  )
  if (filters?.selectedNamespace?.length === 1 && filters.selectedNamespace[0] !== 'all') {
    return filters.selectedNamespace[0]
  }
  return undefined
}

const openCreatePVDialog = () => {
  dialog.open(CreatePVDialog, {
    props: {
      header: 'Create Persistent Volume',
      style: {
        width: '560px'
      },
      modal: true
    }
  })
}

const openCreatePVCDialog = () => {
  dialog.open(CreatePVCDialog, {
    props: {
      header: 'Create Persistent Volume Claim',
      style: {
        width: '520px'
      },
      modal: true
    },
    data: {
      initialNamespace: getInitialNamespace()
    }
  })
}

const openCreateStorageClassDialog = () => {
  dialog.open(CreateStorageClassDialog, {
    props: {
      header: 'Create Storage Class',
      style: {
        width: '540px'
      },
      modal: true
    }
  })
}

const tabs: ResourceTab[] = [
  { id: 'overview' },
  {
    id: 'pvs',
    createAction: {
      label: 'Create PV',
      icon: 'pi pi-plus',
      handler: openCreatePVDialog
    }
  },
  {
    id: 'pvcs',
    createAction: {
      label: 'Create PVC',
      icon: 'pi pi-plus',
      handler: openCreatePVCDialog
    }
  },
  {
    id: 'classes',
    createAction: {
      label: 'Create StorageClass',
      icon: 'pi pi-plus',
      handler: openCreateStorageClassDialog
    }
  }
]
</script>

<template>
  <ResourceTabsLayout title="Storage" default-tab="overview" :tabs="tabs">
    <!-- Overview Tab Panel -->
    <template #tab-overview>
      <div class="flex flex-col gap-6">
        <StorageMetricsCards />
        <StorageOverview />
      </div>
    </template>

    <!-- Persistent Volumes Tab Panel -->
    <template #tab-pvs>
      <div class="flex flex-col gap-6">
        <StorageMetricsCards />
        <StoragePVTable />
      </div>
    </template>

    <!-- PVCs Tab Panel -->
    <template #tab-pvcs>
      <div class="flex flex-col gap-6">
        <StorageMetricsCards />
        <StoragePVCTable />
      </div>
    </template>

    <!-- StorageClasses Tab Panel -->
    <template #tab-classes>
      <div class="flex flex-col gap-6">
        <StorageMetricsCards />
        <StorageClassesTable />
      </div>
    </template>
  </ResourceTabsLayout>
</template>
