<script setup lang="ts">
import { useKubernetesStore } from '@/stores/kubernetesStore'
import {
  getTopConsumers,
  METRIC_CONFIGS,
  type MetricType,
  type ConsumerItem
} from '@/utils/workloadMetrics'
import { ArrowRight } from '@lucide/vue'
import { computed, ref, shallowRef, watchEffect } from 'vue'

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

const metricHeaderLabel = computed(() => METRIC_LABELS[activeTab.value] ?? '')

const consumers = shallowRef<ConsumerItem[]>([])
let prevMetricFingerprint = ''

watchEffect(() => {
  const config = METRIC_CONFIGS[activeTab.value]
  if (!config) return

  const fingerprint =
    activeTab.value +
    ':' +
    kubernetesStore.pods.map((p) => `${p.namespace}/${p.name}:${p[config.sortKey] ?? 0}`).join('|')

  if (fingerprint !== prevMetricFingerprint) {
    prevMetricFingerprint = fingerprint
    consumers.value = getTopConsumers(kubernetesStore.pods, activeTab.value)
  }
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
                <td class="py-2.5 font-medium truncate max-w-25" v-tooltip="item.pod">
                  {{ item.pod }}
                </td>
                <td class="py-2.5 text-muted-color">{{ item.namespace }}</td>
                <td class="py-2.5 font-mono" :class="{ 'text-right': activeTab === 'restarts' }">
                  {{ item.value }}
                </td>
                <td v-if="activeTab !== 'restarts'" class="py-2.5 text-right font-mono">
                  <div class="flex items-center justify-end gap-2">
                    <span class="min-w-12 text-right">{{ item.pct }}%</span>
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
