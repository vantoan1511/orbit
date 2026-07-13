<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import Chart from 'primevue/chart'
import { useKubernetesStore } from '@/stores/kubernetesStore'

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

const labels = ['-60m', '-50m', '-40m', '-30m', '-20m', '-10m', 'Now']

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
</script>

<template>
  <div
    class="bg-[var(--bg-card)] border border-[var(--border)] rounded-xl p-6 shadow-sm transition-all duration-200"
  >
    <div class="text-sm font-semibold text-[var(--text-primary)] uppercase tracking-wider mb-6">
      Resource Usage
    </div>

    <div class="grid grid-cols-1 lg:grid-cols-2 gap-8">
      <!-- CPU Column -->
      <div class="flex flex-col gap-2">
        <div class="flex items-center justify-between">
          <div class="flex flex-col">
            <span class="text-xs text-[var(--text-muted)] font-medium">CPU Usage</span>
            <span class="text-2xl font-bold text-[var(--text-primary)] mt-1">{{ cpuPct }}%</span>
          </div>
          <span class="text-xs text-[var(--text-muted)] font-mono"
            >{{ usedCpu.toFixed(2) }} / {{ totalCpu.toFixed(2) }} cores</span
          >
        </div>
        <!-- Chart Wrapper -->
        <div class="h-44 w-full mt-2">
          <Chart type="line" :data="cpuChartData" :options="chartOptions" class="h-full w-full" />
        </div>
      </div>

      <!-- Memory Column -->
      <div class="flex flex-col gap-2">
        <div class="flex items-center justify-between">
          <div class="flex flex-col">
            <span class="text-xs text-[var(--text-muted)] font-medium">Memory Usage</span>
            <span class="text-2xl font-bold text-[var(--text-primary)] mt-1">{{ memPct }}%</span>
          </div>
          <span class="text-xs text-[var(--text-muted)] font-mono"
            >{{ usedMem.toFixed(2) }} / {{ totalMem.toFixed(2) }} GiB</span
          >
        </div>
        <!-- Chart Wrapper -->
        <div class="h-44 w-full mt-2">
          <Chart type="line" :data="memChartData" :options="chartOptions" class="h-full w-full" />
        </div>
      </div>
    </div>
  </div>
</template>
