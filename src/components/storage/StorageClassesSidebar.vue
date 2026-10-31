<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import Chart from 'primevue/chart'
import { Tag } from 'primevue'
import { Layers, PieChart, Activity } from '@lucide/vue'
import { useKubernetesStore } from '@/stores/kubernetesStore'
import { useTheme } from '@/composables/useTheme'
import { parseCapacityToGiB, formatDonutTotalCapacity } from '@/utils/storageQuantity'

const k8sStore = useKubernetesStore()
const { isDark } = useTheme()
const storageClasses = computed(() => k8sStore.storageClasses)
const pvs = computed(() => k8sStore.persistentVolumes)

// Chart configuration
const chartData = ref()
const chartOptions = ref()

// Calculate capacity by StorageClass
const scCapacityData = computed(() => {
  const capacityMap: Record<string, number> = {}

  // Initialize SCs
  storageClasses.value.forEach((sc) => {
    capacityMap[sc.name] = 0
  })

  // Aggregate PV capacities
  pvs.value.forEach((pv) => {
    const rawSc = pv.storageClass?.trim()
    const sc = !rawSc || rawSc === '-' ? 'Unassigned' : rawSc
    const capGiB = parseCapacityToGiB(pv.capacity)
    capacityMap[sc] = (capacityMap[sc] || 0) + capGiB
  })

  return capacityMap
})

const totalCapacitySummary = computed(() => {
  const total = Object.values(scCapacityData.value).reduce((sum, v) => sum + v, 0)
  return formatDonutTotalCapacity(total)
})

// Count PVs by Status
const statusCounts = computed(() => {
  const counts = { Bound: 0, Available: 0, Released: 0, Failed: 0 }
  pvs.value.forEach((pv) => {
    if (pv.status in counts) {
      counts[pv.status as keyof typeof counts]++
    }
  })
  return counts
})

function getCssVariable(name: string, fallback: string): string {
  if (typeof window !== 'undefined' && typeof document !== 'undefined') {
    const val = getComputedStyle(document.documentElement).getPropertyValue(name).trim()
    if (val) return val
  }
  return fallback
}

const updateChart = () => {
  const dark = isDark.value

  const colors = dark
    ? ['#46d16e', '#3b82f6', '#a855f7', '#f59e0b', '#ec4899', '#64748b']
    : ['#28a745', '#2563eb', '#8b5cf6', '#d97706', '#db2777', '#475569']

  const hoverColors = dark
    ? [
        'rgba(70, 209, 110, 0.8)',
        'rgba(59, 130, 246, 0.8)',
        'rgba(168, 85, 247, 0.8)',
        'rgba(245, 158, 11, 0.8)',
        'rgba(236, 72, 153, 0.8)',
        'rgba(100, 116, 139, 0.8)'
      ]
    : [
        'rgba(40, 167, 69, 0.8)',
        'rgba(37, 99, 235, 0.8)',
        'rgba(139, 92, 246, 0.8)',
        'rgba(217, 119, 6, 0.8)',
        'rgba(219, 39, 119, 0.8)',
        'rgba(71, 85, 105, 0.8)'
      ]

  const labels = Object.keys(scCapacityData.value)
  const dataValues = Object.values(scCapacityData.value)
  const hasData = dataValues.length > 0 && dataValues.some((v) => v > 0)
  const trackColor = dark ? 'rgba(255, 255, 255, 0.08)' : 'rgba(0, 0, 0, 0.06)'
  const cardBg = getCssVariable(
    '--bg-card',
    dark ? 'rgba(18, 20, 24, 1)' : 'rgba(255, 255, 255, 1)'
  )
  const textColor = getCssVariable(
    '--text-primary',
    dark ? 'rgba(236, 238, 242, 1)' : 'rgba(24, 26, 31, 1)'
  )

  chartData.value = {
    labels: hasData ? labels : ['No Capacity Allocation'],
    datasets: [
      {
        data: hasData ? dataValues : [0, 1],
        backgroundColor: hasData
          ? labels.map((_, idx) => colors[idx % colors.length])
          : [trackColor, trackColor],
        hoverBackgroundColor: hasData
          ? labels.map((_, idx) => hoverColors[idx % hoverColors.length])
          : [trackColor, trackColor],
        borderWidth: hasData ? (dark ? 2 : 1) : 0,
        borderColor: cardBg
      }
    ]
  }

  chartOptions.value = {
    responsive: true,
    maintainAspectRatio: false,
    cutout: '70%',
    plugins: {
      legend: {
        display: hasData,
        position: 'bottom',
        labels: {
          color: textColor,
          font: {
            family: 'Inter',
            size: 10,
            weight: '500'
          },
          padding: 10,
          usePointStyle: true,
          pointStyle: 'circle'
        }
      }
    }
  }
}

watch([scCapacityData, isDark], updateChart, { immediate: true })
</script>

<template>
  <div class="flex flex-col gap-6">
    <!-- Storage Classes Mini List -->
    <div
      class="bg-(--bg-card) border border-(--border) rounded-[var(--ui-radius-card)] p-5 shadow-sm"
    >
      <div
        class="text-sm font-semibold text-primary uppercase tracking-wider mb-4 flex items-center gap-2"
      >
        <Layers class="w-4 h-4 text-(--warning)" />
        Storage Classes ({{ storageClasses.length }})
      </div>

      <div class="flex flex-col gap-2.5">
        <div
          v-for="sc in storageClasses"
          :key="sc.name"
          class="flex items-center justify-between p-2.5 rounded-[var(--ui-radius-card)] border border-(--border) bg-(--bg-hover)/10 hover:bg-(--bg-hover)/30 transition-colors"
        >
          <div class="min-w-0">
            <div class="text-xs font-semibold text-primary font-mono truncate">
              {{ sc.name }}
            </div>
            <div class="text-[10px] text-muted-color truncate mt-0.5">{{ sc.provisioner }}</div>
          </div>
          <div class="flex flex-col items-end shrink-0">
            <span class="text-[10px] font-mono text-muted-color">{{ sc.reclaimPolicy }}</span>
            <Tag
              class="mt-1"
              :severity="sc.allowVolumeExpansion ? 'success' : 'secondary'"
              :value="sc.allowVolumeExpansion ? 'Resizable' : 'Static'"
            />
          </div>
        </div>
      </div>
    </div>

    <!-- Capacity by StorageClass Donut -->
    <div
      class="bg-(--bg-card) border border-(--border) rounded-[var(--ui-radius-card)] p-5 shadow-sm"
    >
      <div
        class="text-sm font-semibold text-primary uppercase tracking-wider mb-4 flex items-center gap-2"
      >
        <PieChart class="w-4 h-4 text-(--accent)" />
        Capacity by SC (GiB)
      </div>

      <div class="relative h-56 w-full flex items-center justify-center mt-2">
        <!-- Donut chart -->
        <Chart
          v-if="chartData"
          type="doughnut"
          :data="chartData"
          :options="chartOptions"
          class="w-full h-full"
        />

        <!-- Total Label overlay -->
        <div class="absolute flex flex-col items-center justify-center pointer-events-none">
          <span class="text-2xl font-bold text-primary">{{ totalCapacitySummary.value }}</span>
          <span class="text-[9px] uppercase font-bold text-muted-color tracking-wider">{{
            totalCapacitySummary.unit
          }}</span>
        </div>
      </div>
    </div>

    <!-- Volume Status Grid -->
    <div
      class="bg-(--bg-card) border border-(--border) rounded-[var(--ui-radius-card)] p-5 shadow-sm"
    >
      <div
        class="text-sm font-semibold text-primary uppercase tracking-wider mb-4 flex items-center gap-2"
      >
        <Activity class="w-4 h-4 text-(--success)" />
        Volume Status
      </div>

      <div class="grid grid-cols-2 gap-3 text-center">
        <div
          class="border border-(--border) bg-(--bg-hover)/10 p-2.5 rounded-[var(--ui-radius-card)]"
        >
          <div class="text-[10px] text-muted-color font-semibold uppercase tracking-wider">
            Bound
          </div>
          <div class="text-lg font-bold text-(--success) mt-0.5">{{ statusCounts.Bound }}</div>
        </div>
        <div
          class="border border-(--border) bg-(--bg-hover)/10 p-2.5 rounded-[var(--ui-radius-card)]"
        >
          <div class="text-[10px] text-muted-color font-semibold uppercase tracking-wider">
            Available
          </div>
          <div class="text-lg font-bold text-(--accent) mt-0.5">{{ statusCounts.Available }}</div>
        </div>
        <div
          class="border border-(--border) bg-(--bg-hover)/10 p-2.5 rounded-[var(--ui-radius-card)]"
        >
          <div class="text-[10px] text-muted-color font-semibold uppercase tracking-wider">
            Released
          </div>
          <div class="text-lg font-bold text-(--warning) mt-0.5">{{ statusCounts.Released }}</div>
        </div>
        <div
          class="border border-(--border) bg-(--bg-hover)/10 p-2.5 rounded-[var(--ui-radius-card)]"
        >
          <div class="text-[10px] text-muted-color font-semibold uppercase tracking-wider">
            Failed
          </div>
          <div class="text-lg font-bold text-(--danger) mt-0.5">{{ statusCounts.Failed }}</div>
        </div>
      </div>
    </div>
  </div>
</template>
