<script setup lang="ts">
import { ref } from 'vue'
import Drawer from 'primevue/drawer'
import Tabs from 'primevue/tabs'
import TabList from 'primevue/tablist'
import Tab from 'primevue/tab'
import TabPanels from 'primevue/tabpanels'
import TabPanel from 'primevue/tabpanel'
import { Clock, Tag } from '@lucide/vue'
import type { DeploymentInfo } from './mockDeployments'

const props = defineProps<{
  visible: boolean
  workload: DeploymentInfo | null
}>()

const emit = defineEmits<{
  (e: 'update:visible', value: boolean): void
}>()

const activeTab = ref('overview')

// Generate a dummy YAML representation for the YAML tab
const generateYaml = (w: DeploymentInfo) => {
  return `apiVersion: apps/v1
kind: Deployment
metadata:
  name: ${w.name}
  namespace: ${w.namespace}
  labels:
${Object.entries(w.labels)
  .map(([k, v]) => `    ${k}: ${v}`)
  .join('\n')}
spec:
  replicas: ${w.replicas.desired}
  selector:
    matchLabels:
      app: ${w.name}
  template:
    metadata:
      labels:
        app: ${w.name}
    spec:
      containers:
      - name: ${w.name}
        image: ${w.images[0]}
        ports:
        - containerPort: 80
`
}
</script>

<template>
  <Drawer
    :visible="props.visible"
    @update:visible="emit('update:visible', $event)"
    position="right"
    class="w-full sm:max-w-lg border-l border-(--border) bg-(--bg-card) p-0"
    :header="props.workload?.name || 'Workload Details'"
    :style="{ width: '32rem' }"
  >
    <template #header>
      <div class="flex items-center gap-3 w-full" v-if="props.workload">
        <div class="flex items-center gap-1.5">
          <span
            class="w-2.5 h-2.5 rounded-full animate-pulse"
            :class="
              props.workload.status === 'Running'
                ? 'bg-emerald-500'
                : props.workload.status === 'Progressing'
                  ? 'bg-amber-500'
                  : 'bg-rose-500'
            "
          ></span>
          <span class="text-xs font-bold uppercase tracking-wider text-(--text-muted)">
            {{ props.workload.status }}
          </span>
        </div>
        <div
          class="text-xs text-(--text-muted) font-mono bg-(--bg-hover) px-2 py-0.5 rounded border border-(--border)"
        >
          ns/{{ props.workload.namespace }}
        </div>
      </div>
    </template>

    <div v-if="props.workload" class="h-full flex flex-col">
      <!-- Title Section -->
      <div class="p-6 border-b border-(--border) bg-(--bg-hover)/50">
        <h2 class="text-xl font-bold text-(--text-primary) font-ui truncate mb-1">
          {{ props.workload.name }}
        </h2>
        <div class="text-xs text-(--text-muted) flex items-center gap-2">
          <Clock class="w-3.5 h-3.5" />
          <span>Age: {{ props.workload.age }}</span>
        </div>
      </div>

      <!-- Tab Layout -->
      <div class="flex-1 flex flex-col min-h-0">
        <Tabs v-model:value="activeTab" class="flex-1 flex flex-col">
          <TabList class="border-b border-(--border) px-6 bg-(--bg-card)">
            <Tab value="overview" class="py-3 px-4 text-xs font-bold uppercase tracking-wider"
              >Overview</Tab
            >
            <Tab value="pods" class="py-3 px-4 text-xs font-bold uppercase tracking-wider"
              >Pods</Tab
            >
            <Tab value="events" class="py-3 px-4 text-xs font-bold uppercase tracking-wider"
              >Events</Tab
            >
            <Tab value="yaml" class="py-3 px-4 text-xs font-bold uppercase tracking-wider"
              >YAML</Tab
            >
          </TabList>

          <TabPanels class="p-6 flex-1 overflow-y-auto min-h-0">
            <!-- OVERVIEW PANEL -->
            <TabPanel value="overview" class="space-y-6">
              <!-- Replicas Progress Bars -->
              <div>
                <h3 class="text-[10px] font-bold text-(--text-muted) uppercase tracking-wider mb-3">
                  Replicas Status
                </h3>
                <div class="bg-(--bg-hover)/50 border border-(--border) rounded-xl p-4 space-y-4">
                  <div>
                    <div class="flex justify-between text-xs mb-1">
                      <span class="text-(--text-secondary) font-medium">Desired Replicas</span>
                      <span class="font-mono font-bold text-(--text-primary)">{{
                        props.workload.replicas.desired
                      }}</span>
                    </div>
                    <div class="w-full h-1.5 rounded-full bg-(--bg-hover) overflow-hidden">
                      <div class="h-full rounded-full bg-blue-500" style="width: 100%"></div>
                    </div>
                  </div>

                  <div>
                    <div class="flex justify-between text-xs mb-1">
                      <span class="text-(--text-secondary) font-medium">Current Replicas</span>
                      <span class="font-mono font-bold text-(--text-primary)">{{
                        props.workload.replicas.current
                      }}</span>
                    </div>
                    <div class="w-full h-1.5 rounded-full bg-(--bg-hover) overflow-hidden">
                      <div
                        class="h-full rounded-full bg-indigo-500"
                        :style="{
                          width:
                            (props.workload.replicas.current / props.workload.replicas.desired) *
                              100 +
                            '%'
                        }"
                      ></div>
                    </div>
                  </div>

                  <div>
                    <div class="flex justify-between text-xs mb-1">
                      <span class="text-(--text-secondary) font-medium">Available Replicas</span>
                      <span class="font-mono font-bold text-(--text-primary)">{{
                        props.workload.available
                      }}</span>
                    </div>
                    <div class="w-full h-1.5 rounded-full bg-(--bg-hover) overflow-hidden">
                      <div
                        class="h-full rounded-full bg-emerald-500"
                        :style="{
                          width:
                            (props.workload.available / props.workload.replicas.desired) * 100 + '%'
                        }"
                      ></div>
                    </div>
                  </div>
                </div>
              </div>

              <!-- Metadata Grid -->
              <div>
                <h3 class="text-[10px] font-bold text-(--text-muted) uppercase tracking-wider mb-3">
                  Configuration
                </h3>
                <div
                  class="grid grid-cols-2 gap-4 bg-(--bg-hover)/30 border border-(--border) rounded-xl p-4 text-xs"
                >
                  <div>
                    <span class="text-(--text-muted) block mb-0.5">Namespace</span>
                    <span class="font-semibold text-(--text-secondary)">{{
                      props.workload.namespace
                    }}</span>
                  </div>
                  <div>
                    <span class="text-(--text-muted) block mb-0.5">Strategy</span>
                    <span
                      class="font-semibold text-(--text-secondary) truncate block"
                      :title="props.workload.strategy"
                    >
                      {{ props.workload.strategy }}
                    </span>
                  </div>
                  <div>
                    <span class="text-(--text-muted) block mb-0.5">Min Ready Seconds</span>
                    <span class="font-mono text-(--text-secondary)"
                      >{{ props.workload.minReadySeconds }}s</span
                    >
                  </div>
                  <div>
                    <span class="text-(--text-muted) block mb-0.5">Revision History Limit</span>
                    <span class="font-mono text-(--text-secondary)">{{
                      props.workload.revisionHistory
                    }}</span>
                  </div>
                  <div class="col-span-2">
                    <span class="text-(--text-muted) block mb-0.5">Container Images</span>
                    <div class="space-y-1 mt-1">
                      <span
                        v-for="img in props.workload.images"
                        :key="img"
                        class="inline-block bg-(--bg-hover) text-(--text-secondary) font-mono text-[10px] px-2 py-1 rounded border border-(--border) mr-2 mb-1 truncate max-w-full"
                      >
                        {{ img }}
                      </span>
                    </div>
                  </div>
                </div>
              </div>

              <!-- Labels & Annotations -->
              <div>
                <h3 class="text-[10px] font-bold text-(--text-muted) uppercase tracking-wider mb-3">
                  Labels
                </h3>
                <div class="flex flex-wrap gap-2">
                  <div
                    v-for="(val, key) in props.workload.labels"
                    :key="key"
                    class="flex items-center gap-1 bg-violet-500/5 border border-violet-500/10 rounded-md text-[10px] px-2 py-0.5 text-violet-400 font-mono"
                  >
                    <Tag class="w-3 h-3" />
                    <span>{{ key }}={{ val }}</span>
                  </div>
                </div>
              </div>

              <div>
                <h3 class="text-[10px] font-bold text-(--text-muted) uppercase tracking-wider mb-3">
                  Annotations
                </h3>
                <div class="space-y-1.5">
                  <div
                    v-for="(val, key) in props.workload.annotations"
                    :key="key"
                    class="p-2 rounded bg-(--bg-hover)/50 border border-(--border) text-[10px] font-mono text-(--text-secondary) flex justify-between gap-4"
                  >
                    <span class="text-(--text-muted) truncate shrink-0">{{ key }}</span>
                    <span class="truncate text-right">{{ val }}</span>
                  </div>
                </div>
              </div>
            </TabPanel>

            <!-- PODS PANEL -->
            <TabPanel value="pods" class="space-y-4">
              <div class="text-[10px] font-bold text-(--text-muted) uppercase tracking-wider mb-1">
                Active Pods ({{ props.workload.replicas.current }})
              </div>
              <div class="space-y-2">
                <div
                  v-for="i in props.workload.replicas.current"
                  :key="i"
                  class="flex items-center justify-between p-3.5 bg-(--bg-hover)/30 border border-(--border) rounded-xl"
                >
                  <div class="flex items-center gap-3">
                    <span class="w-2 h-2 rounded-full bg-emerald-500"></span>
                    <div>
                      <span class="text-xs font-semibold text-(--text-secondary) font-mono">
                        {{ props.workload.name }}-{{ Math.random().toString(36).substring(2, 7) }}
                      </span>
                      <span class="block text-[10px] text-(--text-muted) mt-0.5"
                        >IP: 10.244.1.{{ 20 + i }}</span
                      >
                    </div>
                  </div>
                  <div class="text-[10px] font-mono text-(--text-muted)">Ready (1/1)</div>
                </div>
                <div
                  v-if="props.workload.replicas.current === 0"
                  class="text-center py-6 text-xs text-(--text-muted)"
                >
                  No active pods for this workload.
                </div>
              </div>
            </TabPanel>

            <!-- EVENTS PANEL -->
            <TabPanel value="events" class="space-y-4">
              <div class="text-[10px] font-bold text-(--text-muted) uppercase tracking-wider mb-1">
                Recent Events
              </div>
              <div class="relative pl-4 border-l border-(--border) space-y-4 ml-2">
                <div class="relative">
                  <span
                    class="absolute -left-[21px] top-1 w-2.5 h-2.5 rounded-full bg-emerald-500 ring-4 ring-(--bg-card)"
                  ></span>
                  <div class="text-xs font-semibold text-(--text-primary)">ScalingReplicaSet</div>
                  <div class="text-[10px] text-(--text-muted) mt-0.5">
                    Scaled up replica set to {{ props.workload.replicas.desired }}
                  </div>
                  <div class="text-[9px] font-mono text-(--text-muted) mt-1">2m ago</div>
                </div>
                <div class="relative">
                  <span
                    class="absolute -left-[21px] top-1 w-2.5 h-2.5 rounded-full bg-blue-500 ring-4 ring-(--bg-card)"
                  ></span>
                  <div class="text-xs font-semibold text-(--text-primary)">DeploymentRollback</div>
                  <div class="text-[10px] text-(--text-muted) mt-0.5">
                    Recreation strategy triggered successfully
                  </div>
                  <div class="text-[9px] font-mono text-(--text-muted) mt-1">1h ago</div>
                </div>
              </div>
            </TabPanel>

            <!-- YAML PANEL -->
            <TabPanel value="yaml" class="h-full flex flex-col">
              <div
                class="flex-1 min-h-0 bg-zinc-950 rounded-lg border border-zinc-800 p-4 overflow-auto font-mono text-[10px] text-zinc-300 leading-relaxed"
              >
                <pre>{{ generateYaml(props.workload) }}</pre>
              </div>
            </TabPanel>
          </TabPanels>
        </Tabs>
      </div>
    </div>
  </Drawer>
</template>
