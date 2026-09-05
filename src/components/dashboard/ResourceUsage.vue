<script setup lang="ts">
import { useKubernetesStore } from '@/stores/kubernetesStore'
import { useTheme } from '@/composables/useTheme'
import type { TooltipItem } from 'chart.js'
import Chart from 'primevue/chart'
import { computed } from 'vue'

const store = useKubernetesStore()
const { isDark } = useTheme()

const totalCpu = computed(() => {
  return store.nodes.reduce((acc, node) => acc + parseFloat(node.cpuTotal || '0'), 0)
})

const usedCpu = computed(() => {
  return store.nodes.reduce((acc, node) => acc + parseFloat(node.cpuUsed || '0'), 0)
})

const totalMem = computed(() => {
  return store.nodes.reduce((acc, node) => acc + parseFloat(node.memTotal || '0'), 0)
})

const usedMem = computed(() => {
  return store.nodes.reduce((acc, node) => acc + parseFloat(node.memUsed || '0'), 0)
})

const cpuPct = computed(() => {
  return totalCpu.value > 0 ? ((usedCpu.value / totalCpu.value) * 100).toFixed(0) : '0'
})

const memPct = computed(() => {
  return totalMem.value > 0 ? ((usedMem.value / totalMem.value) * 100).toFixed(0) : '0'
})

const availableCpu = computed(() => {
  return Math.max(0, totalCpu.value - usedCpu.value)
})

const availableMem = computed(() => {
  return Math.max(0, totalMem.value - usedMem.value)
})

const cpuChartData = computed(() => {
  const cpuColor = isDark.value ? '#6aa8ff' : '#4f8cff'
  const trackColor = isDark.value ? 'rgba(255, 255, 255, 0.08)' : 'rgba(0, 0, 0, 0.06)'
  const hasData = totalCpu.value > 0

  return {
    labels: ['Used', 'Available'],
    datasets: [
      {
        data: hasData
          ? [Number(usedCpu.value.toFixed(2)), Number(availableCpu.value.toFixed(2))]
          : [0, 1],
        backgroundColor: hasData ? [cpuColor, trackColor] : [trackColor, trackColor],
        hoverBackgroundColor: hasData ? [cpuColor, trackColor] : [trackColor, trackColor],
        borderWidth: 0,
        hoverOffset: hasData ? 3 : 0
      }
    ]
  }
})

const memChartData = computed(() => {
  const memColor = isDark.value ? '#a78bfa' : '#8e6bff'
  const trackColor = isDark.value ? 'rgba(255, 255, 255, 0.08)' : 'rgba(0, 0, 0, 0.06)'
  const hasData = totalMem.value > 0

  return {
    labels: ['Used', 'Available'],
    datasets: [
      {
        data: hasData
          ? [Number(usedMem.value.toFixed(2)), Number(availableMem.value.toFixed(2))]
          : [0, 1],
        backgroundColor: hasData ? [memColor, trackColor] : [trackColor, trackColor],
        hoverBackgroundColor: hasData ? [memColor, trackColor] : [trackColor, trackColor],
        borderWidth: 0,
        hoverOffset: hasData ? 3 : 0
      }
    ]
  }
})

function createChartOptions(unit: string) {
  return {
    responsive: true,
    maintainAspectRatio: false,
    cutout: '74%',
    plugins: {
      legend: {
        display: false
      },
      tooltip: {
        enabled: true,
        callbacks: {
          label: (context: TooltipItem<'doughnut'>) => {
            const label = context.label || ''
            const value = context.parsed ?? context.raw
            const dataset = context.dataset
            const dataArr = (dataset?.data as number[]) || []
            const total = dataArr.reduce((a: number, b: number) => a + (Number(b) || 0), 0)
            const pct = total > 0 ? ((Number(value) / total) * 100).toFixed(1) : '0'
            return ` ${label}: ${Number(value).toFixed(2)} ${unit} (${pct}%)`
          }
        }
      }
    },
    animation: {
      duration: 400
    }
  }
}

const cpuChartOptions = computed(() => createChartOptions('cores'))
const memChartOptions = computed(() => createChartOptions('GiB'))

const metrics = computed(() => [
  {
    title: 'CPU Usage',
    percentage: cpuPct.value,
    used: usedCpu.value,
    available: availableCpu.value,
    total: totalCpu.value,
    unit: 'cores',
    chartData: cpuChartData.value,
    chartOptions: cpuChartOptions.value,
    dotClass: 'bg-(--accent)'
  },
  {
    title: 'Memory Usage',
    percentage: memPct.value,
    used: usedMem.value,
    available: availableMem.value,
    total: totalMem.value,
    unit: 'GiB',
    chartData: memChartData.value,
    chartOptions: memChartOptions.value,
    dotClass: 'bg-violet-500'
  }
])
</script>

<template>
  <Card>
    <template #title>
      <div class="text-sm uppercase font-semibold text-primary tracking-wider">Resource Usage</div>
    </template>
    <template #content>
      <div class="grid grid-cols-1 lg:grid-cols-2 gap-8">
        <div v-for="metric in metrics" :key="metric.title" class="flex flex-col">
          <!-- Metric Header -->
          <div class="flex items-center justify-between">
            <span class="text-xs text-muted-color font-semibold uppercase tracking-wider">
              {{ metric.title }}
            </span>
            <span class="text-xs text-muted-color font-mono">
              Total: {{ metric.total.toFixed(2) }} {{ metric.unit }}
            </span>
          </div>

          <!-- Doughnut Chart with Center Percentage Overlay -->
          <div class="relative h-44 w-full flex items-center justify-center mt-2">
            <Chart
              type="doughnut"
              :data="metric.chartData"
              :options="metric.chartOptions"
              class="w-full h-full"
            />
            <div
              class="absolute inset-0 flex flex-col items-center justify-center pointer-events-none"
            >
              <span class="text-2xl font-bold text-primary">{{ metric.percentage }}%</span>
              <span class="text-[10px] uppercase font-bold text-muted-color tracking-wider"
                >Used</span
              >
            </div>
          </div>

          <!-- Capacity Breakdown Well -->
          <div class="grid grid-cols-3 gap-2 bg-(--bg-hover)/40 rounded-lg p-3 mt-4 text-center">
            <div class="flex flex-col items-center">
              <div class="flex items-center gap-1.5 text-[11px] text-muted-color font-medium">
                <span class="w-2 h-2 rounded-full" :class="metric.dotClass"></span>
                <span>Used</span>
              </div>
              <span class="text-xs font-semibold font-mono text-primary mt-0.5">
                {{ metric.used.toFixed(2) }} {{ metric.unit }}
              </span>
            </div>
            <div class="flex flex-col items-center">
              <div class="flex items-center gap-1.5 text-[11px] text-muted-color font-medium">
                <span class="w-2 h-2 rounded-full bg-(--border-strong)"></span>
                <span>Available</span>
              </div>
              <span class="text-xs font-semibold font-mono text-primary mt-0.5">
                {{ metric.available.toFixed(2) }} {{ metric.unit }}
              </span>
            </div>
            <div class="flex flex-col items-center">
              <span class="text-[11px] text-muted-color font-medium">Total</span>
              <span class="text-xs font-semibold font-mono text-primary mt-0.5">
                {{ metric.total.toFixed(2) }} {{ metric.unit }}
              </span>
            </div>
          </div>
        </div>
      </div>
    </template>
  </Card>
</template>
