<script setup lang="ts">
import ResourceTabsLayout, { type ResourceTab } from '@/components/shared/ResourceTabsLayout.vue'
import PoliciesDataTable from '../components/policies/PoliciesDataTable.vue'
import PoliciesMetricsCards from '../components/policies/PoliciesMetricsCards.vue'

const tabs: ResourceTab[] = [
  { id: 'overview' },
  { id: 'network' },
  { id: 'pod-security' },
  { id: 'resource' },
  { id: 'rbac' },
  { id: 'admission' }
]
</script>

<template>
  <ResourceTabsLayout title="Policies" default-tab="overview" :tabs="tabs">
    <!-- Overview Tab -->
    <template #tab-overview>
      <div class="flex flex-col gap-6">
        <PoliciesMetricsCards />
        <PoliciesDataTable />
      </div>
    </template>

    <!-- Other Tabs (Placeholder logic) -->
    <template #tab-network>
      <div
        class="text-muted-color flex items-center justify-center p-10 border border-(--border) rounded-xl border-dashed"
      >
        Network Policies specific view coming soon.
      </div>
    </template>

    <template #tab-pod-security>
      <div
        class="text-muted-color flex items-center justify-center p-10 border border-(--border) rounded-xl border-dashed"
      >
        Pod Security specific view coming soon.
      </div>
    </template>

    <template #tab-resource>
      <div
        class="text-muted-color flex items-center justify-center p-10 border border-(--border) rounded-xl border-dashed"
      >
        Resource Policies specific view coming soon.
      </div>
    </template>

    <template #tab-rbac>
      <div
        class="text-muted-color flex items-center justify-center p-10 border border-(--border) rounded-xl border-dashed"
      >
        RBAC specific view coming soon.
      </div>
    </template>

    <template #tab-admission>
      <div
        class="text-muted-color flex items-center justify-center p-10 border border-(--border) rounded-xl border-dashed"
      >
        Admission Policies specific view coming soon.
      </div>
    </template>
  </ResourceTabsLayout>
</template>
