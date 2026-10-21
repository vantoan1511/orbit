<script setup lang="ts">
import ViewLayout from '@/components/shared/ViewLayout.vue'
import { resolveSettingsTab } from '@/utils/settingsViewHelpers'
import { ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import SettingsAboutTab from '../components/settings/SettingsAboutTab.vue'
import SettingsAppearanceTab from '../components/settings/SettingsAppearanceTab.vue'
import SettingsClustersTab from '../components/settings/SettingsClustersTab.vue'
import SettingsGeneralTab from '../components/settings/SettingsGeneralTab.vue'
import SettingsPreferencesTab from '../components/settings/SettingsPreferencesTab.vue'

const route = useRoute()
const router = useRouter()

const activeTab = ref(resolveSettingsTab(route.query.tab))

watch(
  () => route.query.tab,
  (queryTab) => {
    const resolved = resolveSettingsTab(queryTab)
    if (activeTab.value !== resolved) {
      activeTab.value = resolved
    }
  }
)

watch(activeTab, (newTab) => {
  if (route.query.tab !== newTab) {
    void router.replace({
      query: {
        ...route.query,
        tab: newTab
      }
    })
  }
})
</script>

<template>
  <ViewLayout data-component="settings-view" title="Settings">
    <!-- Tabs Navigation -->
    <Tabs v-model:value="activeTab" class="flex-1 flex flex-col min-h-0">
      <TabList class="bg-transparent! border-b! border-(--border)! px-1 shrink-0">
        <Tab value="general" class="px-3 py-2 text-xs font-semibold cursor-pointer">General</Tab>
        <Tab value="clusters" class="px-3 py-2 text-xs font-semibold cursor-pointer">
          Clusters
        </Tab>
        <Tab value="preferences" class="px-3 py-2 text-xs font-semibold cursor-pointer">
          Preferences
        </Tab>
        <Tab value="appearance" class="px-3 py-2 text-xs font-semibold cursor-pointer">
          Appearance
        </Tab>
        <Tab
          value="notifications"
          disabled
          class="px-3 py-2 text-xs font-semibold opacity-40 cursor-not-allowed"
        >
          Notifications
        </Tab>
        <Tab
          value="proxy"
          disabled
          class="px-3 py-2 text-xs font-semibold opacity-40 cursor-not-allowed"
        >
          Proxy
        </Tab>
        <Tab value="about" class="px-3 py-2 text-xs font-semibold cursor-pointer">About</Tab>
      </TabList>

      <TabPanels class="bg-transparent border-none p-0 pt-4 flex-1 min-h-0 overflow-y-auto">
        <!-- General Tab -->
        <TabPanel value="general">
          <SettingsGeneralTab />
        </TabPanel>

        <!-- Clusters Tab -->
        <TabPanel value="clusters">
          <SettingsClustersTab />
        </TabPanel>

        <TabPanel value="preferences">
          <SettingsPreferencesTab />
        </TabPanel>

        <TabPanel value="appearance">
          <SettingsAppearanceTab />
        </TabPanel>

        <TabPanel value="notifications">
          <div
            class="text-muted-color flex items-center justify-center p-10 border border-(--border) rounded-[var(--ui-radius-card)] border-dashed"
          >
            Notifications settings coming soon.
          </div>
        </TabPanel>

        <TabPanel value="proxy">
          <div
            class="text-muted-color flex items-center justify-center p-10 border border-(--border) rounded-[var(--ui-radius-card)] border-dashed"
          >
            Proxy settings coming soon.
          </div>
        </TabPanel>

        <TabPanel value="about">
          <SettingsAboutTab />
        </TabPanel>
      </TabPanels>
    </Tabs>
  </ViewLayout>
</template>
