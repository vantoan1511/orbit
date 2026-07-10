<script setup lang="ts">
import { CheckCircle2, FolderOpen, Loader2, Shield, XCircle } from '@lucide/vue'
import Chart from 'primevue/chart'
import { computed, onMounted, ref } from 'vue'
import { mockNamespaces } from './mockNamespaces'

const totalNamespaces = mockNamespaces.length

const activeCount = computed(() => mockNamespaces.filter((n) => n.status === 'Active').length)
const terminatingCount = computed(
  () => mockNamespaces.filter((n) => n.status === 'Terminating').length
)
const failedCount = 0
const systemCount = computed(() => mockNamespaces.filter((n) => n.isSystem).length)

const activeChartData = ref()
const terminatingChartData = ref()
const failedChartData = ref()
const systemChartData = ref()
const donutOptions = ref()

const makeDonutData = (value: number, total: number, color: string) => ({
  datasets: [
    {
      data: [value, total - value],
      backgroundColor: [color, 'rgba(255,255,255,0.05)'],
      borderWidth: 0,
      hoverBackgroundColor: [color, 'rgba(255,255,255,0.05)']
    }
  ]
})

onMounted(() => {
  donutOptions.value = {
    responsive: true,
    maintainAspectRatio: false,
    cutout: '72%',
    plugins: {
      legend: { display: false },
      tooltip: { enabled: false }
    },
    events: []
  }

  activeChartData.value = makeDonutData(
    activeCount.value,
    totalNamespaces,
    '#10b981' // emerald-500
  )
  terminatingChartData.value = makeDonutData(
    terminatingCount.value,
    totalNamespaces,
    '#f59e0b' // amber-500
  )
  failedChartData.value = makeDonutData(failedCount, totalNamespaces, '#ef4444')
  systemChartData.value = makeDonutData(
    systemCount.value,
    totalNamespaces,
    '#6aa8ff' // blue accent
  )
})
</script>

<template>
  <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-5 gap-4">
    <!-- Card 1: Total Namespaces -->
    <div
      class="bg-(--bg-card) border border-(--border) rounded-xl p-5 flex flex-col gap-3 shadow-sm transition-all duration-200 hover:border-(--border-strong)"
    >
      <div class="flex items-center gap-3">
        <div
          class="w-10 h-10 rounded-xl flex items-center justify-center shrink-0 text-violet-400 bg-violet-500/10"
        >
          <FolderOpen class="w-5 h-5" />
        </div>
        <div class="flex-1 min-w-0">
          <div class="text-[10px] font-bold text-(--text-muted) uppercase tracking-wider">
            Total Namespaces
          </div>
          <div class="text-2xl font-bold mt-1 text-(--text-primary)">
            {{ totalNamespaces }}
          </div>
        </div>
      </div>
      <div class="text-[10px] text-(--text-muted) font-medium">Across the cluster</div>
    </div>

    <!-- Card 2: Active -->
    <div
      class="bg-(--bg-card) border border-(--border) rounded-xl p-5 flex flex-col gap-3 shadow-sm transition-all duration-200 hover:border-(--border-strong)"
    >
      <div class="flex items-center justify-between gap-3">
        <div class="flex items-center gap-3">
          <div
            class="w-10 h-10 rounded-xl flex items-center justify-center shrink-0 text-emerald-400 bg-emerald-500/10"
          >
            <CheckCircle2 class="w-5 h-5" />
          </div>
          <div class="flex-1 min-w-0">
            <div class="text-[10px] font-bold text-(--text-muted) uppercase tracking-wider">
              Active
            </div>
            <div class="text-2xl font-bold mt-1 text-emerald-500">{{ activeCount }}</div>
          </div>
        </div>
        <div class="w-14 h-14 shrink-0 relative" v-if="activeChartData">
          <Chart
            type="doughnut"
            :data="activeChartData"
            :options="donutOptions"
            class="w-full h-full"
          />
        </div>
      </div>
      <div class="text-[10px] text-(--text-muted) font-medium">
        {{ Math.round((activeCount / totalNamespaces) * 1000) / 10 }}%
      </div>
    </div>

    <!-- Card 3: Terminating -->
    <div
      class="bg-(--bg-card) border border-(--border) rounded-xl p-5 flex flex-col gap-3 shadow-sm transition-all duration-200 hover:border-(--border-strong)"
    >
      <div class="flex items-center justify-between gap-3">
        <div class="flex items-center gap-3">
          <div
            class="w-10 h-10 rounded-xl flex items-center justify-center shrink-0 text-amber-400 bg-amber-500/10"
          >
            <Loader2 class="w-5 h-5 animate-spin" />
          </div>
          <div class="flex-1 min-w-0">
            <div class="text-[10px] font-bold text-(--text-muted) uppercase tracking-wider">
              Terminating
            </div>
            <div class="text-2xl font-bold mt-1 text-amber-500">{{ terminatingCount }}</div>
          </div>
        </div>
        <div class="w-14 h-14 shrink-0 relative" v-if="terminatingChartData">
          <Chart
            type="doughnut"
            :data="terminatingChartData"
            :options="donutOptions"
            class="w-full h-full"
          />
        </div>
      </div>
      <div class="text-[10px] text-(--text-muted) font-medium">
        {{ Math.round((terminatingCount / totalNamespaces) * 1000) / 10 }}%
      </div>
    </div>

    <!-- Card 4: Failed -->
    <div
      class="bg-(--bg-card) border border-(--border) rounded-xl p-5 flex flex-col gap-3 shadow-sm transition-all duration-200 hover:border-(--border-strong)"
    >
      <div class="flex items-center justify-between gap-3">
        <div class="flex items-center gap-3">
          <div
            class="w-10 h-10 rounded-xl flex items-center justify-center shrink-0 text-rose-400 bg-rose-500/10"
          >
            <XCircle class="w-5 h-5" />
          </div>
          <div class="flex-1 min-w-0">
            <div class="text-[10px] font-bold text-(--text-muted) uppercase tracking-wider">
              Failed
            </div>
            <div class="text-2xl font-bold mt-1 text-rose-500">{{ failedCount }}</div>
          </div>
        </div>
        <div class="w-14 h-14 shrink-0 relative" v-if="failedChartData">
          <Chart
            type="doughnut"
            :data="failedChartData"
            :options="donutOptions"
            class="w-full h-full"
          />
        </div>
      </div>
      <div class="text-[10px] text-(--text-muted) font-medium">0%</div>
    </div>

    <!-- Card 5: System -->
    <div
      class="bg-(--bg-card) border border-(--border) rounded-xl p-5 flex flex-col gap-3 shadow-sm transition-all duration-200 hover:border-(--border-strong)"
    >
      <div class="flex items-center justify-between gap-3">
        <div class="flex items-center gap-3">
          <div
            class="w-10 h-10 rounded-xl flex items-center justify-center shrink-0 text-blue-400 bg-blue-500/10"
          >
            <Shield class="w-5 h-5" />
          </div>
          <div class="flex-1 min-w-0">
            <div class="text-[10px] font-bold text-(--text-muted) uppercase tracking-wider">
              System
            </div>
            <div class="text-2xl font-bold mt-1 text-blue-400">{{ systemCount }}</div>
          </div>
        </div>
        <div class="w-14 h-14 shrink-0 relative" v-if="systemChartData">
          <Chart
            type="doughnut"
            :data="systemChartData"
            :options="donutOptions"
            class="w-full h-full"
          />
        </div>
      </div>
      <div class="text-[10px] text-(--text-muted) font-medium">
        {{ Math.round((systemCount / totalNamespaces) * 1000) / 10 }}%
      </div>
    </div>
  </div>
</template>
