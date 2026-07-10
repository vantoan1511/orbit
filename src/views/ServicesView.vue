<script setup lang="ts">
import ServiceMetricsCards from '../components/services/ServiceMetricsCards.vue'
import ServicesDataTable from '../components/services/ServicesDataTable.vue'
</script>

<template>
  <div class="flex flex-col gap-6">
    <!-- Header/Title -->
    <div class="flex items-center justify-between">
      <h2 class="text-xl font-bold tracking-tight text-(--text-primary)">Services</h2>
    </div>

    <!-- Content Components -->
    <ServiceMetricsCards />
    <ServicesDataTable />
  </div>
</template>
