<script setup lang="ts">
import { ref } from 'vue'
import ConfigMetricsCards from '../components/config/ConfigMetricsCards.vue'
import ConfigDataTable from '../components/config/ConfigDataTable.vue'

const activeTab = ref<'configmaps' | 'secrets'>('configmaps')
</script>

<template>
  <div class="flex flex-col gap-6">
    <!-- Header/Title -->
    <div class="flex items-center justify-between">
      <h2 class="text-xl font-bold tracking-tight text-(--text-primary)">ConfigMaps & Secrets</h2>
    </div>

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
  </div>
</template>
