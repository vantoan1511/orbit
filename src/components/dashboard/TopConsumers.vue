<script setup lang="ts">
import { useKubernetesStore } from '@/stores/kubernetesStore'
import type { PodInfo } from '@/types/kubernetes'
import { ArrowRight } from '@lucide/vue'
import { computed, ref } from 'vue'

type MetricType = 'cpu' | 'memory' | 'restarts'

interface ConsumerItem {
  pod: string
  namespace: string
  value: string
  pct: number
  color: string
}

interface SelectOption {
  label: string
  value: MetricType
}

const kubernetesStore = useKubernetesStore()
const activeTab = ref<MetricType>('cpu')

const tabOptions: SelectOption[] = [
  { label: 'CPU', value: 'cpu' },
  { label: 'Memory', value: 'memory' },
  { label: 'Restarts', value: 'restarts' }
]

const METRIC_LABELS: Record<MetricType, string> = {
  cpu: 'CPU (cores)',
  memory: 'Memory',
  restarts: 'Restarts'
}

const METRIC_CONFIGS: Record<
  MetricType,
  {
    sortKey: keyof PodInfo
    getValue: (p: PodInfo) => string
    getPct: (p: PodInfo) => number
    color: string
  }
> = {
  cpu: {
    sortKey: 'cpuPct',
    getValue: (p) => p.cpu || '0m',
    getPct: (p) => Math.min(Math.max(Math.round(p.cpuPct || 0), 0), 100),
    color: 'bg-blue-500'
  },
  memory: {
    sortKey: 'memoryPct',
    getValue: (p) => p.memory || '0Mi',
    getPct: (p) => Math.min(Math.max(Math.round(p.memoryPct || 0), 0), 100),
    color: 'bg-violet-500'
  },
  restarts: {
    sortKey: 'restarts',
    getValue: (p) => String(p.restarts || 0),
    getPct: () => 0,
    color: 'bg-amber-500'
  }
}

const metricHeaderLabel = computed(() => METRIC_LABELS[activeTab.value] ?? '')

const consumers = computed<ConsumerItem[]>(() => {
  const config = METRIC_CONFIGS[activeTab.value]
  if (!config) return []

  const pods = [...kubernetesStore.pods]
  pods.sort((a, b) => ((b[config.sortKey] as number) || 0) - ((a[config.sortKey] as number) || 0))

  return pods.slice(0, 5).map((p) => ({
    pod: p.name,
    namespace: p.namespace,
    value: config.getValue(p),
    pct: config.getPct(p),
    color: config.color
  }))
})
</script>

<template>
  <Card>
    <template #title>
      <div class="flex items-center justify-between">
        <div class="text-sm font-semibold text-primary uppercase tracking-wider">Top Consumers</div>
        <SelectButton
          v-model="activeTab"
          size="small"
          :allow-empty="false"
          :options="tabOptions"
          option-label="label"
          option-value="value"
        >
          <template #option="slotProps">
            <span class="text-xs">{{ slotProps.option.label }}</span>
          </template>
        </SelectButton>
      </div>
    </template>
    <template #content>
      <div>
        <!-- Table -->
        <div class="overflow-x-auto">
          <table class="w-full text-left text-sm border-collapse">
            <thead>
              <tr class="text-muted-color font-semibold border-b border-surface-500 pb-2">
                <th class="pb-2 font-medium">Pod</th>
                <th class="pb-2 font-medium">Namespace</th>
                <th class="pb-2 font-medium" :class="{ 'text-right': activeTab === 'restarts' }">
                  {{ metricHeaderLabel }}
                </th>
                <th v-if="activeTab !== 'restarts'" class="pb-2 font-medium text-right">
                  Usage (%)
                </th>
              </tr>
            </thead>
            <tbody class="divide-y divide-dashed divide-surface-500">
              <tr
                v-for="item in consumers"
                :key="item.pod"
                class="text-surface-500 hover:text-primary"
              >
                <td class="py-2.5 font-medium truncate max-w-25" :title="item.pod">
                  {{ item.pod }}
                </td>
                <td class="py-2.5 text-muted-color">{{ item.namespace }}</td>
                <td class="py-2.5 font-mono" :class="{ 'text-right': activeTab === 'restarts' }">
                  {{ item.value }}
                </td>
                <td v-if="activeTab !== 'restarts'" class="py-2.5 text-right font-mono">
                  <div class="flex items-center justify-end gap-2">
                    <span class="w-8 text-right">{{ item.pct }}%</span>
                    <div
                      class="w-16 h-1.5 rounded-full bg-(--bg-hover) overflow-hidden hidden sm:block"
                    >
                      <div
                        :style="{ width: item.pct + '%' }"
                        :class="item.color"
                        class="h-full"
                      ></div>
                    </div>
                  </div>
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>
    </template>
    <template #footer>
      <router-link
        to="/workloads?tab=pods"
        class="text-xs text-muted-color hover:underline flex justify-between font-semibold w-full mt-5"
      >
        <span>View all resource usage</span>
        <ArrowRight :size="13" />
      </router-link>
    </template>
  </Card>
</template>
