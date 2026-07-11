<script setup lang="ts">
import EventMetricsCards from '../components/events/EventMetricsCards.vue'
import EventsDataTable from '../components/events/EventsDataTable.vue'
</script>

<template>
  <div class="flex flex-col gap-6">
    <!-- Header/Title -->
    <div class="flex items-center justify-between">
      <h2 class="text-xl font-bold tracking-tight text-(--text-primary)">Events</h2>
    </div>

    <!-- Content Components -->
    <EventMetricsCards />
    <EventsDataTable />
  </div>
</template>
