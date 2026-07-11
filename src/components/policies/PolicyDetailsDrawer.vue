<script setup lang="ts">
import { ref } from 'vue'
import Drawer from 'primevue/drawer'
import Tabs from 'primevue/tabs'
import TabList from 'primevue/tablist'
import Tab from 'primevue/tab'
import TabPanels from 'primevue/tabpanels'
import TabPanel from 'primevue/tabpanel'
import { Clock, Server, FileCode } from '@lucide/vue'
import type { PolicyInfo } from './mockPolicies'

const props = defineProps<{
  visible: boolean
  policy: PolicyInfo | null
}>()

const emit = defineEmits<{
  (e: 'update:visible', value: boolean): void
}>()

const activeTab = ref('overview')

const getStatusBadgeClass = (status: string) => {
  switch (status) {
    case 'Audit':
      return 'bg-blue-500/10 text-blue-400 border border-blue-500/20'
    case 'Enforced':
      return 'bg-emerald-500/10 text-emerald-400 border border-emerald-500/20'
    case 'Disabled':
      return 'bg-gray-500/10 text-gray-400 border border-gray-500/20'
    default:
      return 'bg-gray-500/10 text-gray-400 border border-gray-500/20'
  }
}
</script>

<template>
  <Drawer
    :visible="props.visible"
    @update:visible="emit('update:visible', $event)"
    position="right"
    class="w-full sm:max-w-lg border-l border-(--border) bg-(--bg-card) p-0"
    :header="props.policy?.name || 'Policy Details'"
    :style="{ width: '36rem' }"
  >
    <template #header>
      <div class="flex items-center gap-3 w-full" v-if="props.policy">
        <div class="flex items-center gap-1.5">
          <span
            class="w-2.5 h-2.5 rounded-full"
            :class="
              props.policy.status === 'Enforced'
                ? 'bg-emerald-500'
                : props.policy.status === 'Audit'
                  ? 'bg-blue-500'
                  : 'bg-gray-500'
            "
          ></span>
          <span class="text-xs font-bold uppercase tracking-wider text-(--text-muted)">
            {{ props.policy.type }}
          </span>
        </div>
        <div
          v-if="props.policy.namespace !== '-'"
          class="text-xs text-(--text-muted) font-mono bg-(--bg-hover) px-2 py-0.5 rounded border border-(--border)"
        >
          ns/{{ props.policy.namespace }}
        </div>
        <div
          v-else
          class="text-xs text-(--text-muted) font-mono bg-(--bg-hover) px-2 py-0.5 rounded border border-(--border)"
        >
          Cluster Scope
        </div>
        <div
          class="text-[10px] font-semibold uppercase tracking-wider font-ui border px-2 py-0.5 rounded"
          :class="getStatusBadgeClass(props.policy.status)"
        >
          {{ props.policy.status }}
        </div>
      </div>
    </template>

    <div v-if="props.policy" class="h-full flex flex-col">
      <!-- Title/Message Section -->
      <div class="p-6 border-b border-(--border) bg-(--bg-hover)/50">
        <h2
          class="text-sm font-semibold text-(--text-primary) font-ui mb-2 break-words"
          :title="props.policy.description"
        >
          {{ props.policy.description }}
        </h2>
        <div class="text-xs text-(--text-muted) flex items-center gap-2">
          <Clock class="w-3.5 h-3.5" />
          <span>Last Updated: {{ props.policy.lastUpdated }}</span>
        </div>
      </div>

      <!-- Tab Layout -->
      <div class="flex-1 flex flex-col min-h-0">
        <Tabs v-model:value="activeTab" class="flex-1 flex flex-col">
          <TabList class="border-b border-(--border) px-6 bg-(--bg-card)">
            <Tab
              value="overview"
              class="py-3 px-4 text-xs font-bold uppercase tracking-wider flex items-center gap-1"
            >
              <Server class="w-3.5 h-3.5" />
              <span>Overview</span>
            </Tab>
            <Tab
              value="yaml"
              class="py-3 px-4 text-xs font-bold uppercase tracking-wider flex items-center gap-1"
            >
              <FileCode class="w-3.5 h-3.5" />
              <span>YAML</span>
            </Tab>
          </TabList>

          <TabPanels class="p-6 flex-1 overflow-y-auto min-h-0">
            <!-- OVERVIEW PANEL -->
            <TabPanel value="overview" class="space-y-6">
              <!-- General Info Grid -->
              <div class="space-y-4">
                <h3 class="text-xs font-bold text-(--text-muted) uppercase tracking-wider">
                  Policy Properties
                </h3>
                <div
                  class="border border-(--border) rounded-xl overflow-hidden divide-y divide-(--border) bg-(--bg-hover)/10 text-xs"
                >
                  <div class="grid grid-cols-3 p-3">
                    <span class="text-(--text-secondary) font-semibold">Name</span>
                    <span class="col-span-2 font-mono text-(--text-primary)">{{
                      props.policy.name
                    }}</span>
                  </div>
                  <div class="grid grid-cols-3 p-3">
                    <span class="text-(--text-secondary) font-semibold">Type</span>
                    <span class="col-span-2 text-(--text-primary)">{{ props.policy.type }}</span>
                  </div>
                  <div class="grid grid-cols-3 p-3">
                    <span class="text-(--text-secondary) font-semibold">Scope</span>
                    <span class="col-span-2 text-(--text-primary)">{{ props.policy.scope }}</span>
                  </div>
                  <div class="grid grid-cols-3 p-3" v-if="props.policy.namespace !== '-'">
                    <span class="text-(--text-secondary) font-semibold">Namespace</span>
                    <span class="col-span-2 text-(--text-primary) font-mono">{{
                      props.policy.namespace
                    }}</span>
                  </div>
                  <div class="grid grid-cols-3 p-3">
                    <span class="text-(--text-secondary) font-semibold">Status</span>
                    <span class="col-span-2 text-(--text-primary)">{{ props.policy.status }}</span>
                  </div>
                  <div class="grid grid-cols-3 p-3">
                    <span class="text-(--text-secondary) font-semibold">Mode</span>
                    <span class="col-span-2 text-(--text-primary) font-mono">{{
                      props.policy.mode
                    }}</span>
                  </div>
                  <div class="grid grid-cols-3 p-3">
                    <span class="text-(--text-secondary) font-semibold">Violations (7d)</span>
                    <span
                      class="col-span-2 font-mono font-bold"
                      :class="
                        props.policy.violations > 0 ? 'text-red-400' : 'text-(--text-primary)'
                      "
                    >
                      {{ props.policy.violations }}
                    </span>
                  </div>
                  <div class="grid grid-cols-3 p-3">
                    <span class="text-(--text-secondary) font-semibold">UID</span>
                    <span class="col-span-2 font-mono text-[10px] text-(--text-primary)">{{
                      props.policy.uid
                    }}</span>
                  </div>
                </div>
              </div>
            </TabPanel>

            <!-- YAML PANEL -->
            <TabPanel value="yaml" class="h-full flex flex-col gap-2">
              <div
                class="flex-1 min-h-64 border border-(--border) rounded-xl bg-zinc-950 p-4 overflow-y-auto"
              >
                <pre class="font-mono text-[10px] text-zinc-300 leading-relaxed">{{
                  props.policy.rules
                }}</pre>
              </div>
            </TabPanel>
          </TabPanels>
        </Tabs>
      </div>
    </div>
  </Drawer>
</template>

<style scoped>
:deep(.p-tabs) {
  display: flex;
  flex-direction: column;
  height: 100%;
}
:deep(.p-tablist-content) {
  border-bottom: none !important;
}
:deep(.p-tablist-tab-inline) {
  border-bottom: none !important;
}
</style>
