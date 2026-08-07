<script setup lang="ts">
import ViewLayout from '@/components/shared/ViewLayout.vue'
import PodMetricsCards from '../components/pods/PodMetricsCards.vue'
import PodsDataTable from '../components/pods/PodsDataTable.vue'
</script>

<template>
  <ViewLayout title="Pods">
    <PodMetricsCards />
    <PodsDataTable />
  </ViewLayout>
</template>

