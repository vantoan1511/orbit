<script setup lang="ts">
import { ref, watch } from 'vue'
import Drawer from 'primevue/drawer'
import Tabs from 'primevue/tabs'
import TabList from 'primevue/tablist'
import Tab from 'primevue/tab'
import TabPanels from 'primevue/tabpanels'
import TabPanel from 'primevue/tabpanel'
import Chart from 'primevue/chart'
import { Clock, Tag, Server, Shield, Activity, FileCode } from '@lucide/vue'
import type { PodInfo } from '@/types/kubernetes'

const props = defineProps<{
  visible: boolean
  pod: PodInfo | null
}>()

const emit = defineEmits<{
  (e: 'update:visible', value: boolean): void
}>()

const activeTab = ref('overview')

// Chart states
const cpuChartData = ref()
const memoryChartData = ref()
const chartOptions = ref()

const isDarkTheme = () => document.documentElement.classList.contains('my-app-dark')

const updateCharts = (pod: PodInfo) => {
  const isDark = isDarkTheme()

  const violetColor = '#8b5cf6' // violet-500
  const blueColor = '#3b82f6' // blue-500
  const gridColor = isDark ? 'rgba(255, 255, 255, 0.05)' : 'rgba(0, 0, 0, 0.05)'
  const textColor = isDark ? '#9ca3af' : '#4b5563'

  chartOptions.value = {
    responsive: true,
    maintainAspectRatio: false,
    plugins: {
      legend: { display: false },
      tooltip: {
        enabled: true,
        mode: 'index',
        intersect: false
      }
    },
    scales: {
      x: {
        grid: { color: gridColor },
        ticks: { color: textColor, font: { size: 9 } }
      },
      y: {
        grid: { color: gridColor },
        ticks: { color: textColor, font: { size: 9 } }
      }
    },
    elements: {
      point: { radius: 2, hoverRadius: 4 },
      line: { tension: 0.3, borderWidth: 2 }
    }
  }

  // Fallback if cpuHistory or memoryHistory are missing
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const cpuData = (pod as any).cpuHistory || [0, 0, 0, 0, 0, 0, 0]
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const memData = (pod as any).memoryHistory || [0, 0, 0, 0, 0, 0, 0]

  cpuChartData.value = {
    labels: ['10s ago', '9s ago', '8s ago', '7s ago', '6s ago', '5s ago', 'Now'],
    datasets: [
      {
        data: cpuData,
        borderColor: violetColor,
        backgroundColor: 'rgba(139, 92, 246, 0.05)',
        fill: true
      }
    ]
  }

  memoryChartData.value = {
    labels: ['10s ago', '9s ago', '8s ago', '7s ago', '6s ago', '5s ago', 'Now'],
    datasets: [
      {
        data: memData,
        borderColor: blueColor,
        backgroundColor: 'rgba(59, 130, 246, 0.05)',
        fill: true
      }
    ]
  }
}

watch(
  () => props.pod,
  (newPod) => {
    if (newPod) {
      updateCharts(newPod)
    }
  },
  { immediate: true }
)

const generateYaml = (p: PodInfo) => {
  const labelsYaml = p.labels
    ? Object.entries(p.labels)
        .map(([k, v]) => `    ${k}: ${v}`)
        .join('\n')
    : '    none'

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const containersList = (p as any).containers || [{ name: p.name, image: 'unknown' }]
  const containersYaml = containersList
    .map(
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      (c: any) => `  - name: ${c.name}
    image: ${c.image}
    resources:
      limits:
        cpu: ${p.cpu && p.cpu !== '-' ? p.cpu : '200m'}
        memory: ${p.memory && p.memory !== '-' ? p.memory : '256Mi'}
      requests:
        cpu: 100m
        memory: 128Mi`
    )
    .join('\n')

  const recordPod = p as unknown as Record<string, unknown>
  return `apiVersion: v1
kind: Pod
metadata:
  name: ${p.name}
  namespace: ${p.namespace}
  labels:
${labelsYaml}
spec:
  containers:
${containersYaml}
  nodeName: ${String(recordPod.node || 'N/A')}
status:
  phase: ${p.status}
  podIP: ${String(recordPod.ip || 'N/A')}
  hostIP: ${String(recordPod.nodeIP || 'N/A')}
`
}

const getStatusBadgeClass = (status: string) => {
  switch (status) {
    case 'Running':
      return 'bg-emerald-500'
    case 'Pending':
      return 'bg-amber-500'
    case 'Failed':
    case 'CrashLoopBackOff':
      return 'bg-rose-500'
    case 'Completed':
      return 'bg-blue-500'
    default:
      return 'bg-gray-400'
  }
}
</script>

<template>
  <Drawer
    :visible="props.visible"
    @update:visible="emit('update:visible', $event)"
    position="right"
    class="w-full sm:max-w-lg border-l border-(--border) bg-(--bg-card) p-0"
    :header="props.pod?.name || 'Pod Details'"
    :style="{ width: '36rem' }"
  >
    <template #header>
      <div class="flex items-center gap-3 w-full" v-if="props.pod">
        <div class="flex items-center gap-1.5">
          <span
            class="w-2.5 h-2.5 rounded-full animate-pulse"
            :class="getStatusBadgeClass(props.pod.status)"
          ></span>
          <span class="text-xs font-bold uppercase tracking-wider text-(--text-muted)">
            {{ props.pod.status }}
          </span>
        </div>
        <div
          class="text-xs text-(--text-muted) font-mono bg-(--bg-hover) px-2 py-0.5 rounded border border-(--border)"
        >
          ns/{{ props.pod.namespace }}
        </div>
      </div>
    </template>

    <div v-if="props.pod" class="h-full flex flex-col">
      <!-- Title Section -->
      <div class="p-6 border-b border-(--border) bg-(--bg-hover)/50">
        <h2
          class="text-lg font-bold text-(--text-primary) font-ui truncate mb-1"
          :title="props.pod.name"
        >
          {{ props.pod.name }}
        </h2>
        <div class="text-xs text-(--text-muted) flex items-center gap-2">
          <Clock class="w-3.5 h-3.5" />
          <span>Age: {{ props.pod.age }}</span>
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
              value="containers"
              class="py-3 px-4 text-xs font-bold uppercase tracking-wider flex items-center gap-1"
            >
              <Shield class="w-3.5 h-3.5" />
              <span>Containers</span>
            </Tab>
            <Tab
              value="events"
              class="py-3 px-4 text-xs font-bold uppercase tracking-wider flex items-center gap-1"
            >
              <Activity class="w-3.5 h-3.5" />
              <span>Events</span>
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
              <!-- Metrics sparklines side by side -->
              <div class="grid grid-cols-2 gap-4" v-if="props.pod.cpu !== '-'">
                <div
                  class="bg-(--bg-hover)/30 border border-(--border) rounded-xl p-4 flex flex-col gap-2"
                >
                  <div class="flex justify-between items-center text-xs">
                    <span class="text-(--text-muted) font-semibold">CPU Usage</span>
                    <span class="font-mono font-bold text-violet-400"
                      >{{ props.pod.cpu }} ({{ props.pod.cpuPct }}%)</span
                    >
                  </div>
                  <div class="h-20 w-full" v-if="cpuChartData">
                    <Chart
                      type="line"
                      :data="cpuChartData"
                      :options="chartOptions"
                      class="w-full h-full"
                    />
                  </div>
                </div>

                <div
                  class="bg-(--bg-hover)/30 border border-(--border) rounded-xl p-4 flex flex-col gap-2"
                >
                  <div class="flex justify-between items-center text-xs">
                    <span class="text-(--text-muted) font-semibold">Memory Usage</span>
                    <span class="font-mono font-bold text-blue-400"
                      >{{ props.pod.memory }} ({{ props.pod.memoryPct }}%)</span
                    >
                  </div>
                  <div class="h-20 w-full" v-if="memoryChartData">
                    <Chart
                      type="line"
                      :data="memoryChartData"
                      :options="chartOptions"
                      class="w-full h-full"
                    />
                  </div>
                </div>
              </div>

              <!-- Metadata Grid -->
              <div>
                <h3 class="text-[10px] font-bold text-(--text-muted) uppercase tracking-wider mb-3">
                  Pod Configuration
                </h3>
                <div
                  class="grid grid-cols-2 gap-4 bg-(--bg-hover)/30 border border-(--border) rounded-xl p-4 text-xs font-ui"
                >
                  <div>
                    <span class="text-(--text-muted) block mb-0.5">Namespace</span>
                    <span class="font-semibold text-(--text-secondary)">{{
                      props.pod.namespace
                    }}</span>
                  </div>
                  <div>
                    <span class="text-(--text-muted) block mb-0.5">Node</span>
                    <span
                      class="font-semibold text-(--text-secondary) font-mono truncate block"
                      :title="props.pod.node"
                    >
                      {{ props.pod.node }}
                    </span>
                  </div>
                  <div>
                    <span class="text-(--text-muted) block mb-0.5">IP Address</span>
                    <span class="font-mono text-(--text-secondary)">{{ props.pod.ip }}</span>
                  </div>
                  <div>
                    <span class="text-(--text-muted) block mb-0.5">Node IP</span>
                    <span class="font-mono text-(--text-secondary)">{{ props.pod.nodeIP }}</span>
                  </div>
                  <div>
                    <span class="text-(--text-muted) block mb-0.5">Controlled By</span>
                    <span
                      class="font-semibold text-(--text-secondary) font-mono truncate block"
                      :title="props.pod.controlledBy"
                    >
                      {{ props.pod.controlledBy }}
                    </span>
                  </div>
                  <div>
                    <span class="text-(--text-muted) block mb-0.5">QoS Class</span>
                    <span class="font-semibold text-(--text-secondary)">{{
                      props.pod.qosClass
                    }}</span>
                  </div>
                  <div>
                    <span class="text-(--text-muted) block mb-0.5">Restarts</span>
                    <span class="font-semibold text-(--text-secondary)">{{
                      props.pod.restarts
                    }}</span>
                  </div>
                  <div>
                    <span class="text-(--text-muted) block mb-0.5">Status</span>
                    <span class="font-semibold text-(--text-secondary)">{{
                      props.pod.status
                    }}</span>
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
                    v-for="(val, key) in props.pod.labels"
                    :key="key"
                    class="flex items-center gap-1 bg-violet-500/5 border border-violet-500/10 rounded-md text-[10px] px-2 py-0.5 text-violet-400 font-mono"
                  >
                    <Tag class="w-3 h-3" />
                    <span>{{ key }}={{ val }}</span>
                  </div>
                  <div
                    v-if="!props.pod.labels || Object.keys(props.pod.labels).length === 0"
                    class="text-xs text-(--text-muted)"
                  >
                    No labels configured.
                  </div>
                </div>
              </div>

              <div>
                <h3 class="text-[10px] font-bold text-(--text-muted) uppercase tracking-wider mb-3">
                  Annotations
                </h3>
                <div class="space-y-1.5">
                  <div
                    v-for="(val, key) in props.pod.annotations"
                    :key="key"
                    class="p-2 rounded bg-(--bg-hover)/50 border border-(--border) text-[10px] font-mono text-(--text-secondary) flex justify-between gap-4"
                  >
                    <span class="text-(--text-muted) truncate shrink-0">{{ key }}</span>
                    <span class="truncate text-right">{{ val }}</span>
                  </div>
                  <div
                    v-if="!props.pod.annotations || Object.keys(props.pod.annotations).length === 0"
                    class="text-xs text-(--text-muted)"
                  >
                    No annotations configured.
                  </div>
                </div>
              </div>
            </TabPanel>

            <!-- CONTAINERS PANEL -->
            <TabPanel value="containers" class="space-y-6">
              <div class="text-[10px] font-bold text-(--text-muted) uppercase tracking-wider mb-1">
                Containers ({{ props.pod.containers ? props.pod.containers.length : 0 }})
              </div>
              <div class="space-y-4" v-if="props.pod.containers">
                <div
                  v-for="c in props.pod.containers"
                  :key="c.name"
                  class="bg-(--bg-hover)/30 border border-(--border) rounded-xl p-4 space-y-3"
                >
                  <div class="flex items-center justify-between">
                    <div class="flex items-center gap-2">
                      <span
                        class="w-2 h-2 rounded-full"
                        :class="c.status === 'Running' ? 'bg-emerald-500' : 'bg-rose-500'"
                      ></span>
                      <span class="text-sm font-semibold text-(--text-primary) font-mono">{{
                        c.name
                      }}</span>
                    </div>
                    <span class="text-xs font-mono text-(--text-secondary)"
                      >Ready: {{ c.ready }}</span
                    >
                  </div>

                  <div class="grid grid-cols-2 gap-2 text-xs font-ui">
                    <div>
                      <span class="text-(--text-muted) block text-[10px]">Image</span>
                      <span
                        class="font-mono text-(--text-secondary) truncate block"
                        :title="c.image"
                        >{{ c.image }}</span
                      >
                    </div>
                    <div>
                      <span class="text-(--text-muted) block text-[10px]">Ports</span>
                      <span class="font-mono text-(--text-secondary)">{{ c.ports || '-' }}</span>
                    </div>
                    <div>
                      <span class="text-(--text-muted) block text-[10px]">Restarts</span>
                      <span class="font-mono text-(--text-secondary)">{{ c.restarts }}</span>
                    </div>
                    <div>
                      <span class="text-(--text-muted) block text-[10px]">Status</span>
                      <span class="font-mono text-(--text-secondary)">{{ c.status }}</span>
                    </div>
                  </div>
                </div>
              </div>
            </TabPanel>

            <!-- EVENTS PANEL -->
            <TabPanel value="events" class="space-y-4">
              <div class="text-[10px] font-bold text-(--text-muted) uppercase tracking-wider mb-1">
                Recent Events
              </div>
              <div
                class="relative pl-4 border-l border-(--border) space-y-4 ml-2"
                v-if="props.pod.events && props.pod.events.length > 0"
              >
                <div v-for="(ev, idx) in props.pod.events" :key="idx" class="relative">
                  <span
                    class="absolute -left-[21px] top-1 w-2.5 h-2.5 rounded-full ring-4 ring-(--bg-card)"
                    :class="ev.type === 'Warning' ? 'bg-rose-500' : 'bg-emerald-500'"
                  ></span>
                  <div class="text-xs font-semibold text-(--text-primary)">{{ ev.reason }}</div>
                  <div class="text-[10px] text-(--text-muted) mt-0.5">
                    {{ ev.message }}
                  </div>
                  <div class="text-[9px] font-mono text-(--text-muted) mt-1">{{ ev.age }} ago</div>
                </div>
              </div>
              <div v-else class="text-center py-10 text-xs text-(--text-muted)">
                No recent events recorded for this pod.
              </div>
            </TabPanel>

            <!-- YAML PANEL -->
            <TabPanel value="yaml" class="h-full flex flex-col">
              <div
                class="flex-1 min-h-0 bg-zinc-950 rounded-lg border border-zinc-800 p-4 overflow-auto font-mono text-[10px] text-zinc-300 leading-relaxed"
              >
                <pre>{{ generateYaml(props.pod) }}</pre>
              </div>
            </TabPanel>
          </TabPanels>
        </Tabs>
      </div>
    </div>
  </Drawer>
</template>
