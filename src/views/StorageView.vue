<script setup lang="ts">
import ResourceTabsLayout, { type ResourceTab } from '@/components/shared/ResourceTabsLayout.vue'
import StorageClassesTable from '../components/storage/StorageClassesTable.vue'
import StorageMetricsCards from '../components/storage/StorageMetricsCards.vue'
import StorageOverview from '../components/storage/StorageOverview.vue'
import StoragePVCTable from '../components/storage/StoragePVCTable.vue'
import StoragePVTable from '../components/storage/StoragePVTable.vue'

const tabs: ResourceTab[] = [{ id: 'overview' }, { id: 'pvs' }, { id: 'pvcs' }, { id: 'classes' }]
</script>

<template>
  <ResourceTabsLayout title="Storage" default-tab="overview" :tabs="tabs">
    <!-- Overview Tab Panel -->
    <template #tab-overview>
      <div class="flex flex-col gap-6">
        <StorageMetricsCards />
        <StorageOverview />
      </div>
    </template>

    <!-- Persistent Volumes Tab Panel -->
    <template #tab-pvs>
      <div class="flex flex-col gap-6">
        <StorageMetricsCards />
        <StoragePVTable />
      </div>
    </template>

    <!-- PVCs Tab Panel -->
    <template #tab-pvcs>
      <div class="flex flex-col gap-6">
        <StorageMetricsCards />
        <StoragePVCTable />
      </div>
    </template>

    <!-- StorageClasses Tab Panel -->
    <template #tab-classes>
      <div class="flex flex-col gap-6">
        <StorageMetricsCards />
        <StorageClassesTable />
      </div>
    </template>
  </ResourceTabsLayout>
</template>
