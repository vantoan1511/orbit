<script setup lang="ts">
import ViewLayout from '@/components/shared/ViewLayout.vue'
import { useKubernetesStore } from '@/stores/kubernetesStore'
import { onMounted, ref, watch } from 'vue'
import { useRoute } from 'vue-router'
import ConfigDataTable from '../components/config/ConfigDataTable.vue'
import ConfigMetricsCards from '../components/config/ConfigMetricsCards.vue'

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
      <TabPanels>
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
