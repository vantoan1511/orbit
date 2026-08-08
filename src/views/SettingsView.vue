<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import ViewLayout from '@/components/shared/ViewLayout.vue'
import SettingsAboutTab from '../components/settings/SettingsAboutTab.vue'
import SettingsGeneralTab from '../components/settings/SettingsGeneralTab.vue'
import { ArrowLeft } from '@lucide/vue'

const activeTab = ref('general')

const router = useRouter()
</script>

<template>
  <ViewLayout title="Settings">
    <template #actions>
      <Button severity="secondary" variant="text" size="small" @click="router.back()">
        <ArrowLeft class="w-4 h-4" />
      </Button>
    </template>

    <!-- Tabs Navigation -->
    <Tabs v-model:value="activeTab" class="flex-1 flex flex-col min-h-0">
      <TabList class="border-b border-(--border)">
        <Tab value="general" class="px-5 py-3 text-sm font-semibold">General</Tab>
        <Tab value="clusters" class="px-5 py-3 text-sm font-semibold">Clusters</Tab>
        <Tab value="preferences" class="px-5 py-3 text-sm font-semibold">Preferences</Tab>
        <Tab value="appearance" class="px-5 py-3 text-sm font-semibold">Appearance</Tab>
        <Tab value="notifications" class="px-5 py-3 text-sm font-semibold">Notifications</Tab>
        <Tab value="proxy" class="px-5 py-3 text-sm font-semibold">Proxy</Tab>
        <Tab value="about" class="px-5 py-3 text-sm font-semibold">About</Tab>
      </TabList>

      <TabPanels class="pt-6">
        <!-- General Tab -->
        <TabPanel value="general">
          <SettingsGeneralTab />
        </TabPanel>

        <!-- Placeholder Tabs -->
        <TabPanel value="clusters">
          <div
            class="text-muted-color flex items-center justify-center p-10 border border-(--border) rounded-xl border-dashed"
          >
            Clusters settings coming soon.
          </div>
        </TabPanel>

        <TabPanel value="preferences">
          <div
            class="text-muted-color flex items-center justify-center p-10 border border-(--border) rounded-xl border-dashed"
          >
            Preferences coming soon.
          </div>
        </TabPanel>

        <TabPanel value="appearance">
          <div
            class="text-muted-color flex items-center justify-center p-10 border border-(--border) rounded-xl border-dashed"
          >
            Appearance settings coming soon.
          </div>
        </TabPanel>

        <TabPanel value="notifications">
          <div
            class="text-muted-color flex items-center justify-center p-10 border border-(--border) rounded-xl border-dashed"
          >
            Notifications settings coming soon.
          </div>
        </TabPanel>

        <TabPanel value="proxy">
          <div
            class="text-muted-color flex items-center justify-center p-10 border border-(--border) rounded-xl border-dashed"
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
