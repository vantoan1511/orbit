<script setup lang="ts">
import ResourceTabsLayout, { type ResourceTab } from '@/components/shared/ResourceTabsLayout.vue'
import PoliciesDataTable from '../components/policies/PoliciesDataTable.vue'
import PoliciesMetricsCards from '../components/policies/PoliciesMetricsCards.vue'

const tabs: ResourceTab[] = [
  { id: 'overview' },
  { id: 'network' },
  { id: 'pod-security' },
  { id: 'resource' },
  { id: 'admission' }
]
</script>

<template>
  <ResourceTabsLayout title="Policies" default-tab="overview" :tabs="tabs">
    <!-- Overview Tab -->
    <template #tab-overview>
      <div class="flex flex-col gap-6">
        <PoliciesMetricsCards />
        <PoliciesDataTable category-filter="overview" />
      </div>
    </template>

    <!-- Network Tab -->
    <template #tab-network>
      <PoliciesDataTable category-filter="network" />
    </template>

    <!-- Pod Security Tab -->
    <template #tab-pod-security>
      <PoliciesDataTable category-filter="pod-security" />
    </template>

    <!-- Resource Policies Tab -->
    <template #tab-resource>
      <PoliciesDataTable category-filter="resource" />
    </template>

    <!-- Admission Webhooks Tab -->
    <template #tab-admission>
      <PoliciesDataTable category-filter="admission" />
    </template>
  </ResourceTabsLayout>
</template>
