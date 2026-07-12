<script setup lang="ts">
import type { NamespaceInfo } from '@/types/kubernetes'
import { BarChart2, Clock, FileCode, Layers, Tag } from '@lucide/vue'
import Chart from 'primevue/chart'
import Drawer from 'primevue/drawer'
import Tab from 'primevue/tab'
import TabList from 'primevue/tablist'
import TabPanel from 'primevue/tabpanel'
import TabPanels from 'primevue/tabpanels'
import Tabs from 'primevue/tabs'
import { ref, watch } from 'vue'

interface ResourceQuotaInfo {
  cpuRequest: string
  cpuLimit: string
  cpuUsed: string
  cpuPercent: number
  memoryRequest: string
  memoryLimit: string
  memoryUsed: string
  memoryPercent: number
}

interface LimitRangeInfo {
  type: string
  resource: string
  min: string
  max: string
  default: string
  defaultRequest: string
}

export interface DrawerNamespaceInfo extends NamespaceInfo {
  cpuUsage?: string
  cpuPercent?: number
  cpuHistory?: number[]
  memoryUsage?: string
  memoryPercent?: number
  memoryHistory?: number[]
  resourceQuota?: ResourceQuotaInfo | null
  limitRanges?: LimitRangeInfo[]
}

const props = defineProps<{
  visible: boolean
  namespace: DrawerNamespaceInfo | null
}>()

const emit = defineEmits<{
  (e: 'update:visible', value: boolean): void
}>()

const activeTab = ref('overview')

const cpuChartData = ref()
const memoryChartData = ref()
const chartOptions = ref()

const isDarkTheme = () => document.documentElement.classList.contains('my-app-dark')

const updateCharts = (ns: DrawerNamespaceInfo) => {
  const isDark = isDarkTheme()
  const violetColor = '#8b5cf6'
  const blueColor = '#3b82f6'
  const gridColor = isDark ? 'rgba(255, 255, 255, 0.05)' : 'rgba(0, 0, 0, 0.05)'
  const textColor = isDark ? '#9ca3af' : '#4b5563'

  chartOptions.value = {
    responsive: true,
    maintainAspectRatio: false,
    plugins: {
      legend: { display: false },
      tooltip: { enabled: true, mode: 'index', intersect: false }
    },
    scales: {
      x: { grid: { color: gridColor }, ticks: { color: textColor, font: { size: 9 } } },
      y: { grid: { color: gridColor }, ticks: { color: textColor, font: { size: 9 } } }
    },
    elements: {
      point: { radius: 2, hoverRadius: 4 },
      line: { tension: 0.3, borderWidth: 2 }
    }
  }

  cpuChartData.value = {
    labels: ['10m ago', '8m ago', '6m ago', '4m ago', '2m ago', '1m ago', 'Now'],
    datasets: [
      {
        data: ns.cpuHistory || [0, 0, 0, 0, 0, 0, 0],
        borderColor: violetColor,
        backgroundColor: 'rgba(139, 92, 246, 0.05)',
        fill: true
      }
    ]
  }

  memoryChartData.value = {
    labels: ['10m ago', '8m ago', '6m ago', '4m ago', '2m ago', '1m ago', 'Now'],
    datasets: [
      {
        data: ns.memoryHistory || [0, 0, 0, 0, 0, 0, 0],
        borderColor: blueColor,
        backgroundColor: 'rgba(59, 130, 246, 0.05)',
        fill: true
      }
    ]
  }
}

watch(
  () => props.namespace,
  (ns) => {
    if (ns) {
      activeTab.value = 'overview'
      updateCharts(ns)
    }
  },
  { immediate: true }
)

const getStatusBadgeClass = (status: string) => {
  switch (status) {
    case 'Active':
      return 'bg-emerald-500'
    case 'Terminating':
      return 'bg-amber-500'
    default:
      return 'bg-gray-400'
  }
}

const getStatusTextClass = (status: string) => {
  switch (status) {
    case 'Active':
      return 'text-emerald-400'
    case 'Terminating':
      return 'text-amber-400'
    default:
      return 'text-gray-400'
  }
}

// Annotations collapse state
const annotationsExpanded = ref(false)
</script>

<template>
  <Drawer
    :visible="props.visible"
    @update:visible="emit('update:visible', $event)"
    position="right"
    class="w-full sm:max-w-lg border-l border-(--border) bg-(--bg-card) p-0"
    :style="{ width: '36rem' }"
  >
    <template #header>
      <div class="flex items-center gap-3 w-full" v-if="props.namespace">
        <div class="flex items-center gap-1.5">
          <span
            class="w-2.5 h-2.5 rounded-full"
            :class="getStatusBadgeClass(props.namespace.status)"
          ></span>
          <span
            class="text-xs font-bold uppercase tracking-wider"
            :class="getStatusTextClass(props.namespace.status)"
          >
            {{ props.namespace.status }}
          </span>
        </div>
      </div>
    </template>

    <div v-if="props.namespace" class="h-full flex flex-col">
      <!-- Title Section -->
      <div class="p-6 border-b border-(--border) bg-(--bg-hover)/50">
        <h2 class="text-lg font-bold text-(--text-primary) font-ui truncate mb-1">
          {{ props.namespace.name }}
        </h2>
        <div class="text-xs text-(--text-muted) flex items-center gap-2">
          <Clock class="w-3.5 h-3.5" />
          <span>Age: {{ props.namespace.age }}</span>
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
              <Layers class="w-3.5 h-3.5" />
              <span>Overview</span>
            </Tab>
            <Tab
              value="quotas"
              class="py-3 px-4 text-xs font-bold uppercase tracking-wider flex items-center gap-1"
            >
              <BarChart2 class="w-3.5 h-3.5" />
              <span>Resource Quotas</span>
            </Tab>
            <Tab
              value="limitranges"
              class="py-3 px-4 text-xs font-bold uppercase tracking-wider flex items-center gap-1"
            >
              <FileCode class="w-3.5 h-3.5" />
              <span>Limit Ranges</span>
            </Tab>
          </TabList>

          <TabPanels class="p-6 flex-1 overflow-y-auto min-h-0">
            <!-- OVERVIEW PANEL -->
            <TabPanel value="overview" class="space-y-6">
              <!-- General Section -->
              <div>
                <h3 class="text-[10px] font-bold text-(--text-muted) uppercase tracking-wider mb-3">
                  General
                </h3>
                <div
                  class="bg-(--bg-hover)/30 border border-(--border) rounded-xl p-4 space-y-2.5 text-xs font-ui"
                >
                  <div class="flex justify-between items-center">
                    <span class="text-(--text-muted)">Name</span>
                    <span class="font-semibold text-(--text-primary) font-mono">{{
                      props.namespace.name
                    }}</span>
                  </div>
                  <div class="flex justify-between items-center">
                    <span class="text-(--text-muted)">Status</span>
                    <div class="flex items-center gap-1.5">
                      <span
                        class="w-1.5 h-1.5 rounded-full"
                        :class="getStatusBadgeClass(props.namespace.status)"
                      ></span>
                      <span
                        class="font-semibold"
                        :class="getStatusTextClass(props.namespace.status)"
                        >{{ props.namespace.status }}</span
                      >
                    </div>
                  </div>
                  <div class="flex justify-between items-center">
                    <span class="text-(--text-muted)">Created</span>
                    <span class="text-(--text-secondary)">{{ props.namespace.created }}</span>
                  </div>
                  <div class="flex justify-between items-center">
                    <span class="text-(--text-muted)">Age</span>
                    <span class="font-mono text-(--text-secondary)">{{ props.namespace.age }}</span>
                  </div>
                  <div class="flex justify-between items-start gap-4">
                    <span class="text-(--text-muted) shrink-0">UID</span>
                    <span class="font-mono text-(--text-muted) text-[10px] truncate text-right">{{
                      props.namespace.uid
                    }}</span>
                  </div>
                </div>
              </div>

              <!-- Labels Section -->
              <div>
                <h3 class="text-[10px] font-bold text-(--text-muted) uppercase tracking-wider mb-3">
                  Labels
                </h3>
                <div class="flex flex-wrap gap-2">
                  <div
                    v-for="(val, key) in props.namespace.labels"
                    :key="key"
                    class="flex items-center gap-1 bg-violet-500/5 border border-violet-500/10 rounded-md text-[10px] px-2 py-0.5 text-violet-400 font-mono"
                  >
                    <Tag class="w-3 h-3" />
                    <span>{{ key }}={{ val }}</span>
                  </div>
                  <div
                    v-if="Object.keys(props.namespace.labels).length === 0"
                    class="text-xs text-(--text-muted)"
                  >
                    No labels configured.
                  </div>
                </div>
              </div>

              <!-- Annotations Section -->
              <div>
                <div class="flex items-center justify-between mb-3">
                  <h3 class="text-[10px] font-bold text-(--text-muted) uppercase tracking-wider">
                    Annotations
                  </h3>
                  <button
                    v-if="Object.keys(props.namespace.annotations).length > 0"
                    class="text-[10px] text-(--text-muted) hover:text-(--text-secondary) transition-colors"
                    @click="annotationsExpanded = !annotationsExpanded"
                  >
                    {{ Object.keys(props.namespace.annotations).length }}
                    {{ annotationsExpanded ? '↑' : '↓' }}
                  </button>
                </div>
                <div class="space-y-1.5" v-if="annotationsExpanded">
                  <div
                    v-for="(val, key) in props.namespace.annotations"
                    :key="key"
                    class="p-2 rounded bg-(--bg-hover)/50 border border-(--border) text-[10px] font-mono text-(--text-secondary) flex justify-between gap-4"
                  >
                    <span class="text-(--text-muted) truncate shrink-0">{{ key }}</span>
                    <span class="truncate text-right">{{ val }}</span>
                  </div>
                </div>
                <div
                  v-else-if="Object.keys(props.namespace.annotations).length === 0"
                  class="text-xs text-(--text-muted)"
                >
                  No annotations configured.
                </div>
                <div
                  v-else
                  class="text-[10px] text-(--text-muted) cursor-pointer hover:text-(--text-secondary)"
                  @click="annotationsExpanded = true"
                >
                  {{ Object.keys(props.namespace.annotations).length }} annotation(s) — click to
                  expand
                </div>
              </div>

              <!-- Resource Usage Section -->
              <div>
                <div class="flex items-center justify-between mb-3">
                  <h3 class="text-[10px] font-bold text-(--text-muted) uppercase tracking-wider">
                    Resource Usage
                  </h3>
                  <span v-if="props.namespace.cpuUsage" class="text-[10px] text-(--text-muted)"
                    >Last 1 hour</span
                  >
                </div>
                <div
                  v-if="!props.namespace.cpuUsage"
                  class="bg-(--bg-hover)/20 border border-(--border) border-dashed rounded-xl p-6 text-center text-xs text-(--text-muted)"
                >
                  Resource usage metrics are currently unavailable. Dynamic metric monitoring is
                  planned for a future update.
                </div>
                <div v-else class="space-y-4">
                  <!-- CPU Usage -->
                  <div
                    class="bg-(--bg-hover)/30 border border-(--border) rounded-xl p-4 flex flex-col gap-2"
                  >
                    <div class="flex justify-between items-center text-xs">
                      <span class="text-(--text-muted) font-semibold">CPU Usage</span>
                      <span class="font-mono font-bold text-violet-400">
                        {{ props.namespace.cpuUsage }} ({{ props.namespace.cpuPercent }}%)
                      </span>
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

                  <!-- Memory Usage -->
                  <div
                    class="bg-(--bg-hover)/30 border border-(--border) rounded-xl p-4 flex flex-col gap-2"
                  >
                    <div class="flex justify-between items-center text-xs">
                      <span class="text-(--text-muted) font-semibold">Memory Usage</span>
                      <span class="font-mono font-bold text-blue-400">
                        {{ props.namespace.memoryUsage }} ({{ props.namespace.memoryPercent }}%)
                      </span>
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
              </div>

              <!-- View YAML link -->
              <div class="pt-2 border-t border-(--border)">
                <button
                  class="text-xs text-(--text-muted) hover:text-(--text-secondary) transition-colors flex items-center gap-1.5"
                >
                  <FileCode class="w-3.5 h-3.5" />
                  <span>View YAML</span>
                </button>
              </div>
            </TabPanel>

            <!-- RESOURCE QUOTAS PANEL -->
            <TabPanel value="quotas" class="space-y-6">
              <div v-if="props.namespace.resourceQuota" class="space-y-5">
                <h3 class="text-[10px] font-bold text-(--text-muted) uppercase tracking-wider">
                  Resource Quotas
                </h3>

                <!-- CPU Quota -->
                <div class="bg-(--bg-hover)/30 border border-(--border) rounded-xl p-4 space-y-3">
                  <div class="flex justify-between items-center text-xs">
                    <span class="text-(--text-muted) font-semibold">CPU</span>
                    <span class="font-mono text-(--text-secondary)">{{
                      props.namespace.resourceQuota.cpuUsed
                    }}</span>
                  </div>
                  <div class="w-full h-2 rounded-full bg-(--bg-hover) overflow-hidden">
                    <div
                      class="h-full rounded-full bg-blue-500 transition-all duration-500"
                      :style="{ width: props.namespace.resourceQuota.cpuPercent + '%' }"
                    ></div>
                  </div>
                  <div class="flex justify-between text-[10px] text-(--text-muted) font-mono">
                    <span>{{ props.namespace.resourceQuota.cpuRequest }} request</span>
                    <span
                      :class="
                        props.namespace.resourceQuota.cpuPercent >= 80
                          ? 'text-rose-400'
                          : 'text-blue-400'
                      "
                    >
                      {{ props.namespace.resourceQuota.cpuPercent }}%
                    </span>
                    <span>{{ props.namespace.resourceQuota.cpuLimit }} limit</span>
                  </div>
                </div>

                <!-- Memory Quota -->
                <div class="bg-(--bg-hover)/30 border border-(--border) rounded-xl p-4 space-y-3">
                  <div class="flex justify-between items-center text-xs">
                    <span class="text-(--text-muted) font-semibold">Memory</span>
                    <span class="font-mono text-(--text-secondary)">{{
                      props.namespace.resourceQuota.memoryUsed
                    }}</span>
                  </div>
                  <div class="w-full h-2 rounded-full bg-(--bg-hover) overflow-hidden">
                    <div
                      class="h-full rounded-full bg-violet-500 transition-all duration-500"
                      :style="{ width: props.namespace.resourceQuota.memoryPercent + '%' }"
                    ></div>
                  </div>
                  <div class="flex justify-between text-[10px] text-(--text-muted) font-mono">
                    <span>{{ props.namespace.resourceQuota.memoryRequest }} request</span>
                    <span
                      :class="
                        props.namespace.resourceQuota.memoryPercent >= 80
                          ? 'text-rose-400'
                          : 'text-violet-400'
                      "
                    >
                      {{ props.namespace.resourceQuota.memoryPercent }}%
                    </span>
                    <span>{{ props.namespace.resourceQuota.memoryLimit }} limit</span>
                  </div>
                </div>
              </div>

              <!-- No quota state -->
              <div
                v-else
                class="bg-(--bg-hover)/30 border border-(--border) rounded-xl p-8 text-center flex flex-col items-center gap-3"
              >
                <BarChart2 class="w-8 h-8 text-(--text-muted)/40" />
                <div class="text-sm font-semibold text-(--text-secondary)">No Resource Quotas</div>
                <div class="text-xs text-(--text-muted) max-w-xs">
                  This namespace has no resource quotas configured.
                </div>
              </div>
            </TabPanel>

            <!-- LIMIT RANGES PANEL -->
            <TabPanel value="limitranges" class="space-y-4">
              <div v-if="props.namespace.limitRanges && props.namespace.limitRanges.length > 0">
                <h3 class="text-[10px] font-bold text-(--text-muted) uppercase tracking-wider mb-4">
                  Limit Ranges
                </h3>
                <div
                  v-for="(lr, idx) in props.namespace.limitRanges"
                  :key="idx"
                  class="bg-(--bg-hover)/30 border border-(--border) rounded-xl p-4 mb-3"
                >
                  <div class="flex items-center justify-between mb-3">
                    <span class="text-xs font-semibold text-(--text-primary) font-mono">{{
                      lr.type
                    }}</span>
                    <span
                      class="text-[9px] px-1.5 py-0.5 rounded bg-violet-500/10 text-violet-400 border border-violet-500/20 font-mono"
                    >
                      {{ lr.resource }}
                    </span>
                  </div>
                  <div class="grid grid-cols-2 gap-2 text-[10px]">
                    <div>
                      <span class="text-(--text-muted) block">Min</span>
                      <span class="font-mono text-(--text-secondary)">{{ lr.min }}</span>
                    </div>
                    <div>
                      <span class="text-(--text-muted) block">Max</span>
                      <span class="font-mono text-(--text-secondary)">{{ lr.max }}</span>
                    </div>
                    <div>
                      <span class="text-(--text-muted) block">Default</span>
                      <span class="font-mono text-(--text-secondary)">{{ lr.default }}</span>
                    </div>
                    <div>
                      <span class="text-(--text-muted) block">Default Request</span>
                      <span class="font-mono text-(--text-secondary)">{{ lr.defaultRequest }}</span>
                    </div>
                  </div>
                </div>
              </div>

              <!-- No limit ranges state -->
              <div
                v-else
                class="bg-(--bg-hover)/30 border border-(--border) rounded-xl p-8 text-center flex flex-col items-center gap-3"
              >
                <Layers class="w-8 h-8 text-(--text-muted)/40" />
                <div class="text-sm font-semibold text-(--text-secondary)">No Limit Ranges</div>
                <div class="text-xs text-(--text-muted) max-w-xs">
                  This namespace has no limit ranges configured.
                </div>
              </div>
            </TabPanel>
          </TabPanels>
        </Tabs>
      </div>
    </div>
  </Drawer>
</template>
