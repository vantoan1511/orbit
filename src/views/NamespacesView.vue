<script setup lang="ts">
import ViewLayout from '@/components/shared/ViewLayout.vue'
import Button from 'primevue/button'
import { useDialog } from 'primevue/usedialog'
import CreateNamespaceDialog from '@/components/namespaces/CreateNamespaceDialog.vue'
import NamespaceMetricsCards from '@/components/namespaces/NamespaceMetricsCards.vue'
import NamespacesDataTable from '@/components/namespaces/NamespacesDataTable.vue'

const dialog = useDialog()

const openCreateNamespaceDialog = () => {
  dialog.open(CreateNamespaceDialog, {
    props: {
      header: 'Create Namespace',
      style: {
        width: '460px'
      },
      modal: true
    }
  })
}
</script>

<template>
  <ViewLayout title="Namespaces">
    <template #actions>
      <Button label="Create" icon="pi pi-plus" size="small" @click="openCreateNamespaceDialog" />
    </template>
    <NamespaceMetricsCards />
    <NamespacesDataTable />
  </ViewLayout>
</template>
