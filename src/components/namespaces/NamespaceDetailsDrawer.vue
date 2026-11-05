<script setup lang="ts">
import type { NamespaceInfo } from '@/types/kubernetes'
import { KUBERNETES_NAMESPACE_STATUS, KUBERNETES_RESOURCE_KIND } from '@/constants/kubernetes'
import { getNamespaceStatusBadgeClass } from '@/utils/severity'
import { BarChart2, Clock, FileCode, Layers } from '@lucide/vue'
import BaseResourceDrawer from '@/components/shared/BaseResourceDrawer.vue'
import KeyValueBadgeList from '@/components/shared/KeyValueBadgeList.vue'
import ReactiveAge from '@/components/shared/ReactiveAge.vue'
import ResourceYamlTab from '@/components/shared/ResourceYamlTab.vue'
import { useResourceRawManifest } from '@/composables/useResourceRawManifest'
import { useKubernetesStore } from '@/stores/kubernetesStore'
import {
  extractNamespaceResourceQuota,
  extractNamespaceLimitRanges,
  aggregateNamespacePodMetrics,
  type ResourceQuotaInfo,
  type LimitRangeInfo,
  type AggregatedNamespaceMetrics
} from '@/utils/namespaceResources'
import Chart from 'primevue/chart'
import Tab from 'primevue/tab'
import TabPanel from 'primevue/tabpanel'
import Tag from 'primevue/tag'
import { computed, ref, watch } from 'vue'

export type { ResourceQuotaInfo, LimitRangeInfo }

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

const k8sStore = useKubernetesStore()
const activeTab = ref('overview')

const { displayedYaml, isYamlLoading, copied, copyYaml } = useResourceRawManifest({
  kind: computed(() => KUBERNETES_RESOURCE_KIND.Namespace),
  name: computed(() => props.namespace?.name),
  namespace: computed(() => ''),
  visible: computed(() => props.visible)
})

const effectiveResourceQuota = computed<ResourceQuotaInfo | null>(() => {
  if (!props.namespace?.name) return null
  if (props.namespace.resourceQuota !== undefined) {
    return props.namespace.resourceQuota
  }
  return extractNamespaceResourceQuota(k8sStore.policies, props.namespace.name)
})

const effectiveLimitRanges = computed<LimitRangeInfo[]>(() => {
  if (!props.namespace?.name) return []
  if (props.namespace.limitRanges !== undefined) {
    return props.namespace.limitRanges
  }
  return extractNamespaceLimitRanges(k8sStore.policies, props.namespace.name)
})

const aggregatedMetrics = computed<AggregatedNamespaceMetrics>(() => {
  if (!props.namespace?.name) {
    return {
      cpuUsage: '0m',
      cpuCores: 0,
      cpuPercent: 0,
      memoryUsage: '0Mi',
      memoryBytes: 0,
      memoryPercent: 0
    }
  }
  return aggregateNamespacePodMetrics(
    k8sStore.pods,
    props.namespace.name,
    k8sStore.nodes,
    effectiveResourceQuota.value
  )
})

const displayCpuUsage = computed(
  () => props.namespace?.cpuUsage ?? aggregatedMetrics.value.cpuUsage
)
const displayCpuPercent = computed(
  () => props.namespace?.cpuPercent ?? aggregatedMetrics.value.cpuPercent
)
const displayMemoryUsage = computed(
  () => props.namespace?.memoryUsage ?? aggregatedMetrics.value.memoryUsage
)
const displayMemoryPercent = computed(
  () => props.namespace?.memoryPercent ?? aggregatedMetrics.value.memoryPercent
)

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

  const currentCpuPct = displayCpuPercent.value
  const currentMemPct = displayMemoryPercent.value

  cpuChartData.value = {
    labels: ['10m ago', '8m ago', '6m ago', '4m ago', '2m ago', '1m ago', 'Now'],
    datasets: [
      {
        data: ns.cpuHistory || [
          currentCpuPct,
          currentCpuPct,
          currentCpuPct,
          currentCpuPct,
          currentCpuPct,
          currentCpuPct,
          currentCpuPct
        ],
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
        data: ns.memoryHistory || [
          currentMemPct,
          currentMemPct,
          currentMemPct,
          currentMemPct,
          currentMemPct,
          currentMemPct,
          currentMemPct
        ],
        borderColor: blueColor,
        backgroundColor: 'rgba(59, 130, 246, 0.05)',
        fill: true
      }
    ]
  }
}

watch(
  () => [props.namespace, aggregatedMetrics.value],
  ([ns]) => {
    if (ns) {
      updateCharts(ns as DrawerNamespaceInfo)
    }
  },
  { immediate: true }
)

watch(
  () => props.namespace,
  (ns) => {
    if (ns) {
      activeTab.value = 'overview'
    }
  }
)

const getStatusTextClass = (status: string) => {
  switch (status) {
    case KUBERNETES_NAMESPACE_STATUS.Active:
      return 'text-(--success)'
    case KUBERNETES_NAMESPACE_STATUS.Terminating:
      return 'text-(--warning)'
    default:
      return 'text-(--text-muted)'
  }
}
</script>

<template>
  <BaseResourceDrawer
    data-component="namespace-details-drawer"
    v-model:active-tab="activeTab"
    :visible="props.visible"
    :has-resource="!!props.namespace"
    :title="props.namespace?.name ?? ''"
    :kind="KUBERNETES_RESOURCE_KIND.Namespace"
    :kind-severity="
      props.namespace?.status === KUBERNETES_NAMESPACE_STATUS.Active ? 'success' : 'warn'
    "
    :status-badge-class="
      props.namespace ? getNamespaceStatusBadgeClass(props.namespace.status) : 'bg-muted-color'
    "
    @update:visible="emit('update:visible', $event)"
  >
    <template #metadata>
      <div
        v-if="props.namespace"
        class="flex items-center gap-2 text-xs text-muted-color font-mono mt-0.5"
      >
        <span>status: {{ props.namespace.status }}</span>
        <span class="text-muted-color/60">•</span>
        <span class="flex items-center gap-1">
          <Clock class="w-3 h-3" />
          <ReactiveAge :age="props.namespace.age" />
        </span>
      </div>
    </template>

    <!-- Extra Tabs -->
    <template #extra-tabs>
      <Tab value="quotas" class="text-xs flex items-center gap-1.5 py-2.5 px-3">
        <BarChart2 class="w-3.5 h-3.5" />
        <span>Resource Quotas</span>
      </Tab>
      <Tab value="limitranges" class="text-xs flex items-center gap-1.5 py-2.5 px-3">
        <FileCode class="w-3.5 h-3.5" />
        <span>Limit Ranges</span>
      </Tab>
    </template>

    <!-- Overview Panel -->
    <template #overview>
      <div v-if="props.namespace" class="space-y-6">
        <!-- General Section -->
        <div>
          <h3 class="text-[10px] font-bold text-muted-color uppercase tracking-wider mb-3">
            General
          </h3>
          <div
            class="bg-(--bg-hover)/40 rounded-[var(--ui-radius-card)] p-4 flex flex-col gap-3 text-xs font-ui"
          >
            <div class="flex justify-between items-center">
              <span class="text-muted-color">Name</span>
              <span class="font-semibold text-primary font-mono">{{ props.namespace.name }}</span>
            </div>
            <div class="flex justify-between items-center">
              <span class="text-muted-color">Status</span>
              <div class="flex items-center gap-1.5">
                <span
                  class="w-1.5 h-1.5 rounded-full"
                  :class="getNamespaceStatusBadgeClass(props.namespace.status)"
                ></span>
                <span class="font-semibold" :class="getStatusTextClass(props.namespace.status)">{{
                  props.namespace.status
                }}</span>
              </div>
            </div>
            <div class="flex justify-between items-center">
              <span class="text-muted-color">Created</span>
              <span class="text-muted-color">{{ props.namespace.created }}</span>
            </div>
            <div class="flex justify-between items-center">
              <span class="text-muted-color">Age</span>
              <span class="font-mono text-muted-color"
                ><ReactiveAge :age="props.namespace.age"
              /></span>
            </div>
            <div class="flex justify-between items-start gap-4">
              <span class="text-muted-color shrink-0">UID</span>
              <span class="font-mono text-muted-color text-[10px] truncate text-right">{{
                props.namespace.uid
              }}</span>
            </div>
          </div>
        </div>

        <!-- Labels Section -->
        <KeyValueBadgeList title="Labels" :items="props.namespace.labels" variant="tag" />

        <!-- Annotations Section -->
        <KeyValueBadgeList
          title="Annotations"
          :items="props.namespace.annotations"
          variant="list"
        />

        <!-- Resource Usage Section -->
        <div>
          <div class="flex items-center justify-between mb-3">
            <h3 class="text-[10px] font-bold text-muted-color uppercase tracking-wider">
              Resource Usage
            </h3>
            <span class="text-[10px] text-muted-color">Last 1 hour</span>
          </div>
          <div class="space-y-4">
            <!-- CPU Usage -->
            <div class="bg-(--bg-hover)/40 rounded-[var(--ui-radius-card)] p-4 flex flex-col gap-2">
              <div class="flex justify-between items-center text-xs">
                <span class="text-muted-color font-semibold">CPU Usage</span>
                <span class="font-mono font-bold text-(--accent)">
                  {{ displayCpuUsage }} ({{ displayCpuPercent }}%)
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
            <div class="bg-(--bg-hover)/40 rounded-[var(--ui-radius-card)] p-4 flex flex-col gap-2">
              <div class="flex justify-between items-center text-xs">
                <span class="text-muted-color font-semibold">Memory Usage</span>
                <span class="font-mono font-bold text-(--color-service)">
                  {{ displayMemoryUsage }} ({{ displayMemoryPercent }}%)
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
      </div>
    </template>

    <!-- Extra Panels -->
    <template #extra-panels>
      <!-- RESOURCE QUOTAS PANEL -->
      <TabPanel value="quotas" class="space-y-6">
        <div v-if="effectiveResourceQuota" class="space-y-5">
          <h3 class="text-[10px] font-bold text-muted-color uppercase tracking-wider">
            Resource Quotas
          </h3>

          <!-- CPU Quota -->
          <div class="bg-(--bg-hover)/40 rounded-[var(--ui-radius-card)] p-4 space-y-3">
            <div class="flex justify-between items-center text-xs">
              <span class="text-muted-color font-semibold">CPU</span>
              <span class="font-mono text-muted-color"
                >{{ effectiveResourceQuota.cpuUsed }} / {{ effectiveResourceQuota.cpuLimit }}</span
              >
            </div>
            <div class="h-2 bg-(--bg-app) rounded-full overflow-hidden">
              <div
                class="h-full bg-(--accent) rounded-full transition-all duration-500"
                :style="{
                  width: `${Math.min(100, effectiveResourceQuota.cpuPercent)}%`
                }"
              ></div>
            </div>
            <div class="flex justify-between items-center text-[10px] text-muted-color font-mono">
              <span>Usage</span>
              <span class="font-bold text-primary">{{ effectiveResourceQuota.cpuPercent }}%</span>
            </div>
          </div>

          <!-- Memory Quota -->
          <div class="bg-(--bg-hover)/40 rounded-[var(--ui-radius-card)] p-4 space-y-3">
            <div class="flex justify-between items-center text-xs">
              <span class="text-muted-color font-semibold">Memory</span>
              <span class="font-mono text-muted-color"
                >{{ effectiveResourceQuota.memoryUsed }} /
                {{ effectiveResourceQuota.memoryLimit }}</span
              >
            </div>
            <div class="h-2 bg-(--bg-app) rounded-full overflow-hidden">
              <div
                class="h-full bg-(--color-service) rounded-full transition-all duration-500"
                :style="{
                  width: `${Math.min(100, effectiveResourceQuota.memoryPercent)}%`
                }"
              ></div>
            </div>
            <div class="flex justify-between items-center text-[10px] text-muted-color font-mono">
              <span>Usage</span>
              <span class="font-bold text-primary"
                >{{ effectiveResourceQuota.memoryPercent }}%</span
              >
            </div>
          </div>
        </div>
        <div v-else class="text-xs text-muted-color italic py-4">
          No ResourceQuotas defined for this namespace.
        </div>
      </TabPanel>

      <!-- LIMIT RANGES PANEL -->
      <TabPanel value="limitranges" class="space-y-4">
        <div v-if="effectiveLimitRanges && effectiveLimitRanges.length > 0">
          <h3 class="text-[10px] font-bold text-muted-color uppercase tracking-wider mb-4">
            Limit Ranges
          </h3>
          <div
            v-for="(lr, idx) in effectiveLimitRanges"
            :key="idx"
            class="bg-(--bg-hover)/40 rounded-[var(--ui-radius-card)] p-4 mb-3"
          >
            <div class="flex items-center justify-between mb-3">
              <span class="text-xs font-semibold text-primary font-mono">{{ lr.type }}</span>
              <Tag severity="secondary" class="font-mono" :value="lr.resource" />
            </div>
            <div class="grid grid-cols-2 gap-2 text-[10px]">
              <div>
                <span class="text-muted-color block">Min</span>
                <span class="font-mono text-muted-color">{{ lr.min }}</span>
              </div>
              <div>
                <span class="text-muted-color block">Max</span>
                <span class="font-mono text-muted-color">{{ lr.max }}</span>
              </div>
              <div>
                <span class="text-muted-color block">Default</span>
                <span class="font-mono text-muted-color">{{ lr.default }}</span>
              </div>
              <div>
                <span class="text-muted-color block">Default Request</span>
                <span class="font-mono text-muted-color">{{ lr.defaultRequest }}</span>
              </div>
            </div>
          </div>
        </div>

        <!-- No limit ranges state -->
        <div
          v-else
          class="bg-(--bg-hover)/40 rounded-[var(--ui-radius-card)] p-8 text-center flex flex-col items-center gap-3"
        >
          <Layers class="w-8 h-8 text-muted-color/40" />
          <div class="text-sm font-semibold text-muted-color">No Limit Ranges</div>
          <div class="text-xs text-muted-color max-w-xs">
            This namespace has no limit ranges configured.
          </div>
        </div>
      </TabPanel>
    </template>

    <!-- YAML Panel -->
    <template #yaml>
      <ResourceYamlTab
        :displayed-yaml="displayedYaml"
        :is-yaml-loading="isYamlLoading"
        :copied="copied"
        @copy-yaml="copyYaml"
      />
    </template>
  </BaseResourceDrawer>
</template>
