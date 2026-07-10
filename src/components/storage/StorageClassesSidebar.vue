<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import Chart from 'primevue/chart'
import { Layers, PieChart, Activity } from '@lucide/vue'
import { mockStorageClasses, mockPVs } from './mockStorage'

const storageClasses = ref(mockStorageClasses)
const pvs = ref(mockPVs)

// Chart configuration
const chartData = ref()
const chartOptions = ref()

const parseCapacityToGiB = (cap: string): number => {
  const num = parseFloat(cap)
  if (cap.toLowerCase().includes('tib')) return num * 1024
  if (cap.toLowerCase().includes('gib')) return num
  if (cap.toLowerCase().includes('mib')) return num / 1024
  return num
}

// Calculate capacity by StorageClass
const scCapacityData = computed(() => {
  const capacityMap: Record<string, number> = {}

  // Initialize SCs
  storageClasses.value.forEach((sc) => {
    capacityMap[sc.name] = 0
  })

  // Aggregate PV capacities
  pvs.value.forEach((pv) => {
    const sc = pv.storageClass
    const capGiB = parseCapacityToGiB(pv.capacity)
    capacityMap[sc] = (capacityMap[sc] || 0) + capGiB
  })

  return capacityMap
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

onMounted(() => {
  const isDark = document.documentElement.classList.contains('my-app-dark')

  const colors = isDark
    ? ['#46d16e', '#3b82f6', '#a855f7', '#f59e0b', '#ec4899', '#64748b']
    : ['#28a745', '#2563eb', '#8b5cf6', '#d97706', '#db2777', '#475569']

  const hoverColors = isDark
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

  chartData.value = {
    labels: labels,
    datasets: [
      {
        data: dataValues,
        backgroundColor: colors.slice(0, labels.length),
        hoverBackgroundColor: hoverColors.slice(0, labels.length),
        borderWidth: isDark ? 2 : 1,
        borderColor: isDark ? '#0e1012' : '#ffffff'
      }
    ]
  }

  chartOptions.value = {
    responsive: true,
    maintainAspectRatio: false,
    cutout: '70%',
    plugins: {
      legend: {
        position: 'bottom',
        labels: {
          color: isDark ? '#eceef2' : '#181a1f',
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
})
</script>

<template>
  <div class="flex flex-col gap-6">
    <!-- Storage Classes Mini List -->
    <div class="bg-(--bg-card) border border-(--border) rounded-xl p-5 shadow-sm">
      <div
        class="text-sm font-semibold text-(--text-primary) uppercase tracking-wider mb-4 flex items-center gap-2"
      >
        <Layers class="w-4 h-4 text-amber-400" />
        Storage Classes ({{ storageClasses.length }})
      </div>

      <div class="flex flex-col gap-2.5">
        <div
          v-for="sc in storageClasses"
          :key="sc.name"
          class="flex items-center justify-between p-2.5 rounded-lg border border-(--border) bg-(--bg-hover)/10 hover:bg-(--bg-hover)/30 transition-colors"
        >
          <div class="min-w-0">
            <div class="text-xs font-semibold text-(--text-primary) font-mono truncate">
              {{ sc.name }}
            </div>
            <div class="text-[10px] text-(--text-muted) truncate mt-0.5">{{ sc.provisioner }}</div>
          </div>
          <div class="flex flex-col items-end shrink-0">
            <span class="text-[10px] font-mono text-(--text-secondary)">{{
              sc.reclaimPolicy
            }}</span>
            <span
              class="text-[9px] px-1.5 py-0.5 rounded mt-1 font-semibold"
              :class="
                sc.allowVolumeExpansion
                  ? 'bg-emerald-500/10 text-emerald-400'
                  : 'bg-zinc-500/10 text-zinc-400'
              "
            >
              {{ sc.allowVolumeExpansion ? 'Resizable' : 'Static' }}
            </span>
          </div>
        </div>
      </div>
    </div>

    <!-- Capacity by StorageClass Donut -->
    <div class="bg-(--bg-card) border border-(--border) rounded-xl p-5 shadow-sm">
      <div
        class="text-sm font-semibold text-(--text-primary) uppercase tracking-wider mb-4 flex items-center gap-2"
      >
        <PieChart class="w-4 h-4 text-violet-400" />
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
          <span class="text-2xl font-bold text-(--text-primary)">2,620</span>
          <span class="text-[9px] uppercase font-bold text-(--text-muted) tracking-wider"
            >Total GiB</span
          >
        </div>
      </div>
    </div>

    <!-- Volume Status Grid -->
    <div class="bg-(--bg-card) border border-(--border) rounded-xl p-5 shadow-sm">
      <div
        class="text-sm font-semibold text-(--text-primary) uppercase tracking-wider mb-4 flex items-center gap-2"
      >
        <Activity class="w-4 h-4 text-emerald-400" />
        Volume Status
      </div>

      <div class="grid grid-cols-2 gap-3 text-center">
        <div class="border border-(--border) bg-(--bg-hover)/10 p-2.5 rounded-lg">
          <div class="text-[10px] text-(--text-muted) font-semibold uppercase tracking-wider">
            Bound
          </div>
          <div class="text-lg font-bold text-emerald-500 mt-0.5">{{ statusCounts.Bound }}</div>
        </div>
        <div class="border border-(--border) bg-(--bg-hover)/10 p-2.5 rounded-lg">
          <div class="text-[10px] text-(--text-muted) font-semibold uppercase tracking-wider">
            Available
          </div>
          <div class="text-lg font-bold text-blue-500 mt-0.5">{{ statusCounts.Available }}</div>
        </div>
        <div class="border border-(--border) bg-(--bg-hover)/10 p-2.5 rounded-lg">
          <div class="text-[10px] text-(--text-muted) font-semibold uppercase tracking-wider">
            Released
          </div>
          <div class="text-lg font-bold text-amber-500 mt-0.5">{{ statusCounts.Released }}</div>
        </div>
        <div class="border border-(--border) bg-(--bg-hover)/10 p-2.5 rounded-lg">
          <div class="text-[10px] text-(--text-muted) font-semibold uppercase tracking-wider">
            Failed
          </div>
          <div class="text-lg font-bold text-rose-500 mt-0.5">{{ statusCounts.Failed }}</div>
        </div>
      </div>
    </div>
  </div>
</template>
