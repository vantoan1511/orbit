<script setup lang="ts">
import { useKubernetesStore } from '@/stores/kubernetesStore'
import Chart from 'primevue/chart'
import { computed, onMounted, ref } from 'vue'

const store = useKubernetesStore()

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

const labels = ['-90s', '-75s', '-60s', '-45s', '-30s', '-15s', 'Now']

const chartOptions = ref()
const cpuGradient = ref<CanvasGradient | null>(null)
const memGradient = ref<CanvasGradient | null>(null)

const cpuChartData = computed(() => {
  return {
    labels: labels,
    datasets: [
      {
        label: 'CPU Usage',
        data: [...store.cpuHistory],
        fill: true,
        borderColor: '#4f8cff',
        backgroundColor: cpuGradient.value || 'rgba(79, 140, 255, 0.1)',
        tension: 0.4,
        borderWidth: 2,
        pointRadius: 0,
        pointHoverRadius: 4
      }
    ]
  }
})

const memChartData = computed(() => {
  return {
    labels: labels,
    datasets: [
      {
        label: 'Memory Usage',
        data: [...store.memHistory],
        fill: true,
        borderColor: '#8e6bff',
        backgroundColor: memGradient.value || 'rgba(142, 107, 255, 0.1)',
        tension: 0.4,
        borderWidth: 2,
        pointRadius: 0,
        pointHoverRadius: 4
      }
    ]
  }
})

onMounted(() => {
  const isDark = document.documentElement.classList.contains('my-app-dark')
  const textColor = isDark ? '#878d98' : '#7b8191'
  const borderColor = isDark ? '#2e343d' : '#d5d9e1'

  const ctxCpu = document.createElement('canvas').getContext('2d')
  if (ctxCpu) {
    const gradient = ctxCpu.createLinearGradient(0, 0, 0, 150)
    gradient.addColorStop(0, 'rgba(79, 140, 255, 0.3)')
    gradient.addColorStop(1, 'rgba(79, 140, 255, 0.0)')
    cpuGradient.value = gradient
  }

  const ctxMem = document.createElement('canvas').getContext('2d')
  if (ctxMem) {
    const gradient = ctxMem.createLinearGradient(0, 0, 0, 150)
    gradient.addColorStop(0, 'rgba(142, 107, 255, 0.3)')
    gradient.addColorStop(1, 'rgba(142, 107, 255, 0.0)')
    memGradient.value = gradient
  }

  chartOptions.value = {
    responsive: true,
    maintainAspectRatio: false,
    plugins: {
      legend: {
        display: false
      },
      tooltip: {
        mode: 'index',
        intersect: false
      }
    },
    scales: {
      x: {
        grid: {
          display: false
        },
        ticks: {
          color: textColor,
          font: {
            size: 10,
            family: 'Inter'
          }
        }
      },
      y: {
        min: 0,
        max: 100,
        ticks: {
          color: textColor,
          font: {
            size: 10,
            family: 'Inter'
          },
          stepSize: 50,
          callback: (value: number) => value + '%'
        },
        grid: {
          color: borderColor,
          drawTicks: false
        }
      }
    }
  }
})

const metrics = computed(() => [
  {
    title: 'CPU Usage',
    percentage: cpuPct.value,
    used: usedCpu.value,
    total: totalCpu.value,
    unit: 'cores',
    chartData: cpuChartData.value
  },
  {
    title: 'Memory Usage',
    percentage: memPct.value,
    used: usedMem.value,
    total: totalMem.value,
    unit: 'GiB',
    chartData: memChartData.value
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
        <div v-for="metric in metrics" :key="metric.title" class="flex flex-col gap-2">
          <div class="flex items-center justify-between">
            <div class="flex flex-col">
              <span class="text-xs text-muted-color font-medium">{{ metric.title }}</span>
              <span class="text-2xl font-bold text-primary mt-1">{{ metric.percentage }}%</span>
            </div>
            <span class="text-xs text-muted-color font-mono">
              {{ metric.used.toFixed(2) }} / {{ metric.total.toFixed(2) }} {{ metric.unit }}
            </span>
          </div>
          <!-- Chart Wrapper -->
          <div class="h-44 w-full mt-2">
            <Chart
              type="line"
              :data="metric.chartData"
              :options="chartOptions"
              class="h-full w-full"
            />
          </div>
        </div>
      </div>
    </template>
  </Card>
</template>
