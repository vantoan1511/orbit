<script setup lang="ts">
import BaseResourceDrawer from '@/components/shared/BaseResourceDrawer.vue'
import type { PolicyInfo } from '@/types/kubernetes'
import { Clock, FileCode, Server } from '@lucide/vue'
import Tab from 'primevue/tab'
import TabList from 'primevue/tablist'
import TabPanel from 'primevue/tabpanel'
import TabPanels from 'primevue/tabpanels'
import Tabs from 'primevue/tabs'
import { ref } from 'vue'

const props = defineProps<{
  visible: boolean
  policy: PolicyInfo | null
}>()

const emit = defineEmits<{
  (e: 'update:visible', value: boolean): void
}>()

const activeTab = ref('overview')

const getStatusSeverity = (status: string) => {
  switch (status) {
    case 'Audit':
      return 'info'
    case 'Enforced':
      return 'success'
    case 'Disabled':
      return 'secondary'
    default:
      return 'secondary'
  }
}

const getPolicyBadgeClass = (status: string) => {
  if (status === 'Enforced') return 'bg-emerald-500'
  if (status === 'Audit') return 'bg-blue-500'
  return 'bg-gray-500'
}
</script>

<template>
  <BaseResourceDrawer
    :visible="props.visible"
    :has-resource="!!props.policy"
    :title="props.policy?.name ?? ''"
    :kind="props.policy?.status ?? ''"
    :kind-severity="props.policy ? getStatusSeverity(props.policy.status) : 'secondary'"
    :status-badge-class="props.policy ? getPolicyBadgeClass(props.policy.status) : 'bg-gray-500'"
    @update:visible="emit('update:visible', $event)"
  >
    <template #metadata>
      <div
        v-if="props.policy"
        class="flex items-center gap-2 text-xs text-muted-color font-mono mt-0.5"
      >
        <span>{{
          props.policy.namespace !== '-' ? `ns: ${props.policy.namespace}` : 'Cluster Scope'
        }}</span>
        <span class="text-muted-color/60">•</span>
        <span class="flex items-center gap-1">
          <Clock class="w-3 h-3" />
          <span>{{ props.policy.lastUpdated }}</span>
        </span>
      </div>
    </template>

    <div v-if="props.policy" class="flex flex-col h-full">
      <!-- Tab Layout -->
      <Tabs v-model:value="activeTab" class="flex flex-col flex-1 min-h-0">
        <TabList class="bg-transparent! border-b! border-(--border)! px-2">
          <Tab value="overview" class="text-xs! flex items-center gap-1.5 py-2.5 px-3">
            <Server class="w-3.5 h-3.5" />
            <span>Overview</span>
          </Tab>
          <Tab value="yaml" class="text-xs! flex items-center gap-1.5 py-2.5 px-3">
            <FileCode class="w-3.5 h-3.5" />
            <span>YAML</span>
          </Tab>
        </TabList>

        <TabPanels class="flex-1 overflow-y-auto p-6! bg-transparent!">
          <!-- OVERVIEW PANEL -->
          <TabPanel value="overview" class="space-y-6">
            <!-- Description -->
            <div v-if="props.policy.description" class="bg-(--bg-hover)/40 rounded-xl p-4">
              <div class="text-[10px] font-bold text-muted-color uppercase tracking-wider mb-1.5">
                Description
              </div>
              <div class="text-xs text-primary leading-relaxed break-words font-ui">
                {{ props.policy.description }}
              </div>
            </div>

            <!-- General Info Grid -->
            <div>
              <h3 class="text-xs font-bold text-muted-color uppercase tracking-wider mb-3">
                Policy Properties
              </h3>
              <div class="bg-(--bg-hover)/40 rounded-xl p-4 flex flex-col gap-3 text-xs font-ui">
                <div class="grid grid-cols-3">
                  <span class="text-muted-color font-semibold">Name</span>
                  <span class="col-span-2 font-mono text-primary">{{ props.policy.name }}</span>
                </div>
                <div class="grid grid-cols-3">
                  <span class="text-muted-color font-semibold">Type</span>
                  <span class="col-span-2 text-primary">{{ props.policy.type }}</span>
                </div>
                <div class="grid grid-cols-3">
                  <span class="text-muted-color font-semibold">Scope</span>
                  <span class="col-span-2 text-primary">{{ props.policy.scope }}</span>
                </div>
                <div class="grid grid-cols-3" v-if="props.policy.namespace !== '-'">
                  <span class="text-muted-color font-semibold">Namespace</span>
                  <span class="col-span-2 text-primary font-mono">{{
                    props.policy.namespace
                  }}</span>
                </div>
                <div class="grid grid-cols-3">
                  <span class="text-muted-color font-semibold">Status</span>
                  <span class="col-span-2 text-primary">{{ props.policy.status }}</span>
                </div>
                <div class="grid grid-cols-3">
                  <span class="text-muted-color font-semibold">Mode</span>
                  <span class="col-span-2 text-primary font-mono">{{ props.policy.mode }}</span>
                </div>
                <div class="grid grid-cols-3">
                  <span class="text-muted-color font-semibold">Violations (7d)</span>
                  <span
                    class="col-span-2 font-mono font-bold"
                    :class="props.policy.violations > 0 ? 'text-rose-400' : 'text-primary'"
                  >
                    {{ props.policy.violations }}
                  </span>
                </div>
                <div class="grid grid-cols-3 items-start gap-4">
                  <span class="text-muted-color font-semibold shrink-0">UID</span>
                  <span class="col-span-2 font-mono text-[10px] text-primary truncate">{{
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
  </BaseResourceDrawer>
</template>
