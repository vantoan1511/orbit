<script setup lang="ts">
import ViewLayout from '@/components/shared/ViewLayout.vue'
import EventMetricsCards from '../components/events/EventMetricsCards.vue'
import EventsDataTable from '../components/events/EventsDataTable.vue'
</script>

<template>
  <ViewLayout title="Events">
    <EventMetricsCards class="shrink-0" />
    <EventsDataTable class="flex-1 min-h-0" />
  </ViewLayout>
</template>
