<script setup lang="ts">
import StatusSummary from "../components/dashboard/StatusSummary.vue";
import ResourceUsage from "../components/dashboard/ResourceUsage.vue";
import HealthOverview from "../components/dashboard/HealthOverview.vue";
import AlertsEvents from "../components/dashboard/AlertsEvents.vue";
import TopConsumers from "../components/dashboard/TopConsumers.vue";
import WorkloadSummaries from "../components/dashboard/WorkloadSummaries.vue";
</script>

<template>
  <div class="flex flex-col gap-6">
    <!-- Top status summaries -->
    <StatusSummary />

    <!-- Resource usage and health overview -->
    <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
      <ResourceUsage class="lg:col-span-2" />
      <HealthOverview class="lg:col-span-1" />
    </div>

    <!-- Warnings, Events and Top Consumers -->
    <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
      <AlertsEvents class="lg:col-span-2" />
      <TopConsumers class="lg:col-span-1" />
    </div>

    <!-- Bottom row workloads -->
    <WorkloadSummaries />
  </div>
</template>
