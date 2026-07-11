<script setup lang="ts">
import { ref } from 'vue'
import SettingsGeneralTab from '../components/settings/SettingsGeneralTab.vue'

const activeTab = ref('general')
</script>

<template>
  <div class="flex flex-col h-full">
    <!-- Header/Title -->
    <div class="flex items-center justify-between mb-6">
      <h2 class="text-xl font-bold tracking-tight text-(--text-primary)">Settings</h2>
    </div>

    <!-- Tabs Navigation -->
    <Tabs v-model:value="activeTab" class="flex-1 flex flex-col min-h-0">
      <TabList class="border-b border-(--border) mb-6">
        <Tab value="general" class="px-5 py-3 text-sm font-semibold">General</Tab>
        <Tab value="clusters" class="px-5 py-3 text-sm font-semibold">Clusters</Tab>
        <Tab value="preferences" class="px-5 py-3 text-sm font-semibold">Preferences</Tab>
        <Tab value="appearance" class="px-5 py-3 text-sm font-semibold">Appearance</Tab>
        <Tab value="notifications" class="px-5 py-3 text-sm font-semibold">Notifications</Tab>
        <Tab value="proxy" class="px-5 py-3 text-sm font-semibold">Proxy</Tab>
        <Tab value="about" class="px-5 py-3 text-sm font-semibold">About</Tab>
      </TabList>

      <TabPanels class="flex-1 min-h-0 p-0 bg-transparent border-none">
        <!-- General Tab -->
        <TabPanel value="general">
          <SettingsGeneralTab />
        </TabPanel>

        <!-- Placeholder Tabs -->
        <TabPanel value="clusters">
          <div
            class="text-(--text-muted) flex items-center justify-center p-10 border border-(--border) rounded-xl border-dashed"
          >
            Clusters settings coming soon.
          </div>
        </TabPanel>

        <TabPanel value="preferences">
          <div
            class="text-(--text-muted) flex items-center justify-center p-10 border border-(--border) rounded-xl border-dashed"
          >
            Preferences coming soon.
          </div>
        </TabPanel>

        <TabPanel value="appearance">
          <div
            class="text-(--text-muted) flex items-center justify-center p-10 border border-(--border) rounded-xl border-dashed"
          >
            Appearance settings coming soon.
          </div>
        </TabPanel>

        <TabPanel value="notifications">
          <div
            class="text-(--text-muted) flex items-center justify-center p-10 border border-(--border) rounded-xl border-dashed"
          >
            Notifications settings coming soon.
          </div>
        </TabPanel>

        <TabPanel value="proxy">
          <div
            class="text-(--text-muted) flex items-center justify-center p-10 border border-(--border) rounded-xl border-dashed"
          >
            Proxy settings coming soon.
          </div>
        </TabPanel>

        <TabPanel value="about">
          <div
            class="text-(--text-muted) flex items-center justify-center p-10 border border-(--border) rounded-xl border-dashed"
          >
            About Orbit coming soon.
          </div>
        </TabPanel>
      </TabPanels>
    </Tabs>
  </div>
</template>
