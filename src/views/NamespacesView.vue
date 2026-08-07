<script setup lang="ts">
import ViewLayout from '@/components/shared/ViewLayout.vue'
import NamespaceMetricsCards from '../components/namespaces/NamespaceMetricsCards.vue'
import NamespacesDataTable from '../components/namespaces/NamespacesDataTable.vue'
</script>

<template>
  <ViewLayout title="Namespaces">
    <NamespaceMetricsCards />
    <NamespacesDataTable />
  </ViewLayout>
</template>

