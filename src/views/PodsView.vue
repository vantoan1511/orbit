<script setup lang="ts">
import PodMetricsCards from '../components/pods/PodMetricsCards.vue'
import PodsDataTable from '../components/pods/PodsDataTable.vue'
</script>

<template>
  <div class="flex flex-col gap-6">
    <!-- Header/Title -->
    <div class="flex items-center justify-between">
      <h2 class="text-xl font-bold tracking-tight text-(--text-primary)">Pods</h2>
    </div>

    <!-- Content Components -->
    <PodMetricsCards />
    <PodsDataTable />
  </div>
</template>
