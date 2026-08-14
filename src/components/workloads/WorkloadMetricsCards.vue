<script setup lang="ts">
import { useKubernetesStore } from '@/stores/kubernetesStore'
import { AlertTriangle, Boxes, CheckCircle2, Server, TrendingUp } from '@lucide/vue'
import Chart from 'primevue/chart'
import { computed, onMounted, ref } from 'vue'

const k8sStore = useKubernetesStore()
const deployments = computed(() => k8sStore.deployments)

const totals = computed(() => {
  const total = deployments.value.length
  let currentReplicas = 0
  let desiredReplicas = 0
  let availableReplicas = 0
  let upToDateReplicas = 0
  let running = 0
  let progressing = 0
  let failed = 0

  deployments.value.forEach((d) => {
    currentReplicas += d.replicas.current
    desiredReplicas += d.replicas.desired
    availableReplicas += d.available
    upToDateReplicas += d.upToDate
    if (d.status === 'Running') running++
    else if (d.status === 'Progressing') progressing++
    else if (d.status === 'Failed') failed++
  })

  return {
    total,
    currentReplicas,
    desiredReplicas,
    availableReplicas,
    upToDateReplicas,
    running,
    progressing,
    failed
  }
})

// Doughnut chart configurations
const isDark = ref(false)
const miniChartOptions = ref()

onMounted(() => {
  isDark.value = document.documentElement.classList.contains('my-app-dark')

  miniChartOptions.value = {
    responsive: true,
    maintainAspectRatio: false,
    cutout: '75%',
    plugins: {
      legend: { display: false },
      tooltip: { enabled: false }
    }
  }
})

const availableChartData = computed(() => {
  const runningColor = isDark.value ? '#46d16e' : '#28a745'
  const trackColor = isDark.value ? 'rgba(255, 255, 255, 0.05)' : 'rgba(0, 0, 0, 0.05)'
  return {
    datasets: [
      {
        data: [totals.value.running, totals.value.total - totals.value.running],
        backgroundColor: [runningColor, trackColor],
        borderWidth: 0
      }
    ]
  }
})

const progressingChartData = computed(() => {
  const pendingColor = isDark.value ? '#ffc54d' : '#f4a100'
  const trackColor = isDark.value ? 'rgba(255, 255, 255, 0.05)' : 'rgba(0, 0, 0, 0.05)'
  return {
    datasets: [
      {
        data: [totals.value.progressing, totals.value.total - totals.value.progressing],
        backgroundColor: [pendingColor, trackColor],
        borderWidth: 0
      }
    ]
  }
})

const failedChartData = computed(() => {
  const failedColor = isDark.value ? '#ff6b6b' : '#d64545'
  const trackColor = isDark.value ? 'rgba(255, 255, 255, 0.05)' : 'rgba(0, 0, 0, 0.05)'
  return {
    datasets: [
      {
        data: [totals.value.failed, totals.value.total - totals.value.failed],
        backgroundColor: [failedColor, trackColor],
        borderWidth: 0
      }
    ]
  }
})

const cardItems = computed(() => [
  {
    title: 'Total Deployments',
    value: totals.value.total,
    totalValue: null,
    icon: Boxes,
    iconClass: 'text-violet-400 bg-violet-500/10',
    valueClass: 'text-primary',
    type: 'simple',
    footerText: 'All controllers active',
    footerClass: 'text-emerald-500'
  },
  {
    title: 'Replicas',
    value: totals.value.currentReplicas,
    totalValue: totals.value.desiredReplicas,
    icon: Server,
    iconClass: 'text-blue-400 bg-blue-500/10',
    valueClass: 'text-primary',
    type: 'progress',
    percentValue: Math.round(
      (totals.value.currentReplicas / (totals.value.desiredReplicas || 1)) * 100
    ),
    progressClass: 'bg-blue-500',
    footerText: 'desired'
  },
  {
    title: 'Up-To-Date',
    value: totals.value.upToDateReplicas,
    totalValue: totals.value.currentReplicas,
    icon: TrendingUp,
    iconClass: 'text-cyan-400 bg-cyan-500/10',
    valueClass: 'text-primary',
    type: 'progress',
    percentValue: Math.round(
      (totals.value.upToDateReplicas / (totals.value.currentReplicas || 1)) * 100
    ),
    progressClass: 'bg-cyan-500',
    footerText: 'synchronized'
  },
  {
    title: 'Available',
    value: totals.value.running,
    totalValue: null,
    icon: CheckCircle2,
    iconClass: 'text-emerald-400 bg-emerald-500/10',
    valueClass: 'text-emerald-500',
    type: 'chart',
    chartData: availableChartData.value,
    percentValue: Math.round((totals.value.running / (totals.value.total || 1)) * 100),
    percentColorClass: 'text-emerald-500',
    footerText: `${totals.value.running} of ${totals.value.total} deployments healthy`,
    footerClass: 'text-muted-color'
  },
  {
    title: 'Updating',
    value: totals.value.progressing,
    totalValue: null,
    icon: TrendingUp,
    iconClass: 'text-amber-400 bg-amber-500/10',
    valueClass: 'text-amber-500',
    type: 'chart',
    chartData: progressingChartData.value,
    percentValue: Math.round((totals.value.progressing / (totals.value.total || 1)) * 100),
    percentColorClass: 'text-amber-500',
    footerText: `${totals.value.progressing} deployment rolling out`,
    footerClass: 'text-muted-color'
  },
  {
    title: 'Failed',
    value: totals.value.failed,
    totalValue: null,
    icon: AlertTriangle,
    iconClass: 'text-rose-400 bg-rose-500/10',
    valueClass: 'text-rose-500',
    type: 'chart',
    chartData: failedChartData.value,
    percentValue: Math.round((totals.value.failed / (totals.value.total || 1)) * 100),
    percentColorClass: 'text-rose-500',
    footerText: 'Requires operator attention',
    footerClass: 'text-rose-500'
  }
])
</script>

<template>
  <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-6 gap-6">
    <Card v-for="(card, index) in cardItems" :key="index">
      <template #content>
        <div class="flex items-center gap-4">
          <div
            :class="[
              'w-10 h-10 rounded-xl flex items-center justify-center shrink-0',
              card.iconClass
            ]"
          >
            <component :is="card.icon" class="w-5 h-5" />
          </div>
          <div class="flex-1 min-w-0">
            <div class="text-xs font-bold text-muted-color uppercase tracking-wider">
              {{ card.title }}
            </div>
            <div :class="['text-2xl font-bold mt-1', card.valueClass]">
              {{ card.value }}
              <span v-if="card.totalValue !== null" class="text-sm font-normal text-muted-color">
                / {{ card.totalValue }}
              </span>
            </div>
          </div>
          <div
            v-if="card.type === 'chart' && card.chartData"
            class="w-13 h-13 relative shrink-0 flex items-center justify-center"
          >
            <Chart
              type="doughnut"
              :data="card.chartData"
              :options="miniChartOptions"
              class="w-full h-full"
            />
            <span :class="['absolute text-xs font-bold', card.percentColorClass]">
              {{ card.percentValue }}%
            </span>
          </div>
        </div>
      </template>
      <template #footer>
        <div class="mt-4">
          <template v-if="card.type === 'progress'">
            <div class="flex justify-between text-xs text-muted-color mb-1 font-mono">
              <span>{{ card.percentValue }}% {{ card.footerText }}</span>
            </div>
            <div class="w-full h-1.5 rounded-full bg-(--bg-hover) overflow-hidden">
              <div
                :class="['h-full rounded-full', card.progressClass]"
                :style="{ width: card.percentValue + '%' }"
              ></div>
            </div>
          </template>
          <div v-else :class="['text-xs font-medium', card.footerClass]">
            {{ card.footerText }}
          </div>
        </div>
      </template>
    </Card>
  </div>
</template>
