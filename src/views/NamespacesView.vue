<script setup lang="ts">
import NamespaceMetricsCards from '../components/namespaces/NamespaceMetricsCards.vue'
import NamespacesDataTable from '../components/namespaces/NamespacesDataTable.vue'
</script>

<template>
  <div class="flex flex-col gap-6">
    <!-- Header/Title -->
    <div class="flex items-center justify-between">
      <h2 class="text-xl font-bold tracking-tight text-(--text-primary)">Namespaces</h2>
    </div>

    <!-- Content Components -->
    <NamespaceMetricsCards />
    <NamespacesDataTable />
  </div>
</template>
