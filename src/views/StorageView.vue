<script setup lang="ts">
import ViewLayout from '@/components/shared/ViewLayout.vue'
import { ref, watch } from 'vue'
import { useRoute } from 'vue-router'
import StorageClassesTable from '../components/storage/StorageClassesTable.vue'
import StorageMetricsCards from '../components/storage/StorageMetricsCards.vue'
import StorageOverview from '../components/storage/StorageOverview.vue'
import StoragePVCTable from '../components/storage/StoragePVCTable.vue'
import StoragePVTable from '../components/storage/StoragePVTable.vue'

type StorageViewTab = 'overview' | 'pvs' | 'pvcs' | 'classes'

const route = useRoute()
const activeTab = ref<StorageViewTab>((route.query.tab as StorageViewTab) || 'overview')

watch(
  () => route.query.tab,
  (newTab) => {
    if (newTab && typeof newTab === 'string') {
      activeTab.value = newTab as StorageViewTab
    }
  },
  { immediate: true }
)
</script>

<template>
  <ViewLayout title="Storage">
    <!-- Content Tabs Layout -->
    <Tabs v-model:value="activeTab">
      <TabPanels>
        <!-- Overview Tab Panel -->
        <TabPanel value="overview">
          <div class="flex flex-col gap-6">
            <StorageMetricsCards />
            <StorageOverview />
          </div>
        </TabPanel>

        <!-- Persistent Volumes Tab Panel -->
        <TabPanel value="pvs">
          <div class="flex flex-col gap-6">
            <StorageMetricsCards />
            <StoragePVTable />
          </div>
        </TabPanel>

        <!-- PVCs Tab Panel -->
        <TabPanel value="pvcs">
          <div class="flex flex-col gap-6">
            <StorageMetricsCards />
            <StoragePVCTable />
          </div>
        </TabPanel>

        <!-- StorageClasses Tab Panel -->
        <TabPanel value="classes">
          <div class="flex flex-col gap-6">
            <StorageMetricsCards />
            <StorageClassesTable />
          </div>
        </TabPanel>
      </TabPanels>
    </Tabs>
  </ViewLayout>
</template>
