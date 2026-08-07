<script setup lang="ts">
import ViewLayout from '@/components/shared/ViewLayout.vue'
import NodeMetricsCards from '../components/nodes/NodeMetricsCards.vue'
import NodesTable from '../components/nodes/NodesTable.vue'
</script>

<template>
  <ViewLayout title="Nodes">
    <NodeMetricsCards />
    <NodesTable />
  </ViewLayout>
</template>

