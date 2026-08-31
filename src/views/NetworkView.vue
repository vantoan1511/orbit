<script setup lang="ts">
import ResourceTabsLayout, { type ResourceTab } from '@/components/shared/ResourceTabsLayout.vue'
import { useDialog } from 'primevue/usedialog'
import CreateIngressDialog from '@/components/network/CreateIngressDialog.vue'
import CreateServiceDialog from '@/components/network/CreateServiceDialog.vue'
import IngressesTable from '../components/network/IngressesTable.vue'
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

const tabs: ResourceTab[] = [
  {
    id: 'services',
    createAction: { handler: openCreateServiceDialog }
  },
  {
    id: 'ingresses',
    createAction: { handler: openCreateIngressDialog }
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
  </ResourceTabsLayout>
</template>
