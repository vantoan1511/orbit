<script setup lang="ts">
import ResourceTabsLayout, { type ResourceTab } from '@/components/shared/ResourceTabsLayout.vue'
import { useDialog } from 'primevue/usedialog'
import CreateIngressDialog from '@/components/network/CreateIngressDialog.vue'
import CreatePortForwardDialog from '@/components/network/CreatePortForwardDialog.vue'
import CreateServiceDialog from '@/components/network/CreateServiceDialog.vue'
import IngressesTable from '../components/network/IngressesTable.vue'
import PortForwardsTable from '../components/network/PortForwardsTable.vue'
import ServicesTable from '../components/network/ServicesTable.vue'

const dialog = useDialog()

const openCreateServiceDialog = () => {
  dialog.open(CreateServiceDialog, {
    props: {
      header: 'Create Service',
      style: {
        width: '420px'
      },
      modal: true
    }
  })
}

const openCreateIngressDialog = () => {
  dialog.open(CreateIngressDialog, {
    props: {
      header: 'Create Ingress',
      style: {
        width: '420px'
      },
      modal: true
    }
  })
}

const openCreatePortForwardDialog = () => {
  dialog.open(CreatePortForwardDialog, {
    props: {
      header: 'Forward Port',
      style: {
        width: '440px'
      },
      modal: true
    }
  })
}

const tabs: ResourceTab[] = [
  {
    id: 'services',
    createAction: { handler: openCreateServiceDialog }
  },
  {
    id: 'ingresses',
    createAction: { handler: openCreateIngressDialog }
  },
  {
    id: 'port-forward',
    createAction: {
      label: 'Forward Port',
      icon: 'pi pi-plus',
      handler: openCreatePortForwardDialog
    }
  }
]
</script>

<template>
  <ResourceTabsLayout title="Network" default-tab="services" :tabs="tabs">
    <!-- Services Tab -->
    <template #tab-services>
      <ServicesTable />
    </template>

    <!-- Ingresses Tab -->
    <template #tab-ingresses>
      <IngressesTable />
    </template>

    <!-- Port Forward Tab -->
    <template #tab-port-forward>
      <PortForwardsTable />
    </template>
  </ResourceTabsLayout>
</template>
