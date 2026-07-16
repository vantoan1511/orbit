<script setup lang="ts">
import NodeMetricsCards from '../components/nodes/NodeMetricsCards.vue'
import NodesTable from '../components/nodes/NodesTable.vue'
</script>

<template>
  <div class="flex flex-col gap-6">
    <NodeMetricsCards />
    <NodesTable />
  </div>
</template>
