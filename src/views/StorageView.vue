<script setup lang="ts">
import { ref } from 'vue'
import DataTable from 'primevue/datatable'
import Column from 'primevue/column'
import StorageMetricsCards from '../components/storage/StorageMetricsCards.vue'
import StorageOverview from '../components/storage/StorageOverview.vue'
import StoragePVTable from '../components/storage/StoragePVTable.vue'
import StoragePVCTable from '../components/storage/StoragePVCTable.vue'
import { mockStorageClasses } from '../components/storage/mockStorage'

const activeTab = ref<'overview' | 'pvs' | 'pvcs' | 'classes'>('overview')
const storageClasses = ref(mockStorageClasses)
</script>

<template>
  <div class="flex flex-col gap-6">
    <!-- Header/Title -->
    <div class="flex items-center justify-between">
      <h2 class="text-xl font-bold tracking-tight text-(--text-primary)">Storage</h2>
    </div>

    <!-- Content Tabs Layout -->
    <Tabs v-model:value="activeTab">
      <TabList class="border-b border-(--border)">
        <Tab value="overview" class="px-5 py-3 text-sm font-semibold">Overview</Tab>
        <Tab value="pvs" class="px-5 py-3 text-sm font-semibold">PersistentVolumes</Tab>
        <Tab value="pvcs" class="px-5 py-3 text-sm font-semibold">Claims (PVCs)</Tab>
        <Tab value="classes" class="px-5 py-3 text-sm font-semibold">StorageClasses</Tab>
      </TabList>

      <TabPanels class="pt-6">
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
            <div
              class="bg-(--bg-card) border border-(--border) rounded-xl p-5 shadow-sm flex flex-col gap-5"
            >
              <div class="text-sm font-semibold text-(--text-primary) uppercase tracking-wider">
                Storage Classes
              </div>
              <DataTable
                :value="storageClasses"
                class="p-datatable-sm border border-(--border) rounded-lg overflow-hidden"
                tableClass="w-full text-left text-xs border-collapse"
              >
                <Column field="name" header="Name" class="font-medium p-3 text-(--text-primary)">
                  <template #body="{ data }">
                    <span class="font-semibold font-mono">{{ data.name }}</span>
                  </template>
                </Column>
                <Column
                  field="provisioner"
                  header="Provisioner"
                  class="p-3 font-mono text-(--text-secondary)"
                ></Column>
                <Column
                  field="reclaimPolicy"
                  header="Reclaim Policy"
                  class="p-3 text-(--text-muted)"
                ></Column>
                <Column
                  field="volumeBindingMode"
                  header="Volume Binding Mode"
                  class="p-3 text-(--text-secondary)"
                ></Column>
                <Column field="allowVolumeExpansion" header="Allow Volume Expansion" class="p-3">
                  <template #body="{ data }">
                    <span
                      class="px-2 py-0.5 rounded text-[10px] font-bold uppercase tracking-wider"
                      :class="
                        data.allowVolumeExpansion
                          ? 'bg-emerald-500/10 text-emerald-400 border border-emerald-500/20'
                          : 'bg-zinc-500/10 text-zinc-400 border border-zinc-500/20'
                      "
                    >
                      {{ data.allowVolumeExpansion ? 'True' : 'False' }}
                    </span>
                  </template>
                </Column>
                <Column field="age" header="Age" class="p-3 text-(--text-muted) font-mono"></Column>
              </DataTable>
            </div>
          </div>
        </TabPanel>
      </TabPanels>
    </Tabs>
  </div>
</template>

<style scoped>
:deep(.p-datatable-thead > tr > th) {
  background: var(--bg-hover);
  color: var(--text-secondary);
  border-bottom: 1px solid var(--border);
  font-weight: 600;
  font-size: 0.7rem;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

:deep(.p-datatable-tbody > tr:hover) {
  background: var(--bg-hover) / 20;
}
</style>
