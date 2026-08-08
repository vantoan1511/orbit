<script setup lang="ts">
import { ref, onMounted, watch } from 'vue'
import { useRoute } from 'vue-router'
import ViewLayout from '@/components/shared/ViewLayout.vue'
import ConfigMetricsCards from '../components/config/ConfigMetricsCards.vue'
import ConfigDataTable from '../components/config/ConfigDataTable.vue'
import { useKubernetesStore } from '@/stores/kubernetesStore'

const route = useRoute()
const activeTab = ref<'configmaps' | 'secrets'>(
  (route.query.tab as 'configmaps' | 'secrets') || 'configmaps'
)
const k8sStore = useKubernetesStore()

watch(
  () => route.query.tab,
  (newTab) => {
    if (newTab && (newTab === 'configmaps' || newTab === 'secrets')) {
      activeTab.value = newTab
    }
  },
  { immediate: true }
)

onMounted(async () => {
  await k8sStore.fetchConfigMaps()
  await k8sStore.fetchSecrets()
})
</script>

<template>
  <ViewLayout title="ConfigMaps & Secrets">
    <!-- Content Tabs Layout -->
    <Tabs v-model:value="activeTab">
      <TabList class="border-b border-(--border)">
        <Tab value="configmaps" class="px-5 py-3 text-sm font-semibold">ConfigMaps</Tab>
        <Tab value="secrets" class="px-5 py-3 text-sm font-semibold">Secrets</Tab>
      </TabList>

      <TabPanels class="pt-6">
        <!-- ConfigMaps Tab Panel -->
        <TabPanel value="configmaps">
          <div class="flex flex-col gap-6">
            <ConfigMetricsCards activeTab="configmaps" />
            <ConfigDataTable activeTab="configmaps" />
          </div>
        </TabPanel>

        <!-- Secrets Tab Panel -->
        <TabPanel value="secrets">
          <div class="flex flex-col gap-6">
            <ConfigMetricsCards activeTab="secrets" />
            <ConfigDataTable activeTab="secrets" />
          </div>
        </TabPanel>
      </TabPanels>
    </Tabs>
  </ViewLayout>
</template>
