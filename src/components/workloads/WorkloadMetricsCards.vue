<script setup lang="ts">
import Chart from 'primevue/chart'
import { useKubernetesStore } from '@/stores/kubernetesStore'
import { computed, onMounted, ref } from 'vue'
import { Boxes, Server, CheckCircle2, TrendingUp, AlertTriangle } from '@lucide/vue'

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
</script>

<template>
  <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-6 gap-6">
    <!-- Card 1: Total Deployments -->
    <div
      class="bg-(--bg-card) border border-(--border) rounded-xl p-5 flex flex-col justify-between shadow-sm transition-all duration-200 hover:border-(--border-strong)"
    >
      <div class="flex items-center gap-4">
        <div
          class="w-10 h-10 rounded-xl flex items-center justify-center shrink-0 text-violet-400 bg-violet-500/10"
        >
          <Boxes class="w-5 h-5" />
        </div>
        <div class="flex-1 min-w-0">
          <div class="text-[10px] font-bold text-(--text-muted) uppercase tracking-wider">
            Total Deployments
          </div>
          <div class="text-2xl font-bold mt-1 text-(--text-primary)">
            {{ totals.total }}
          </div>
        </div>
      </div>
      <div class="mt-4 text-[10px] text-emerald-500 font-medium">All controllers active</div>
    </div>

    <!-- Card 2: Replicas Status -->
    <div
      class="bg-(--bg-card) border border-(--border) rounded-xl p-5 flex flex-col justify-between shadow-sm transition-all duration-200 hover:border-(--border-strong)"
    >
      <div class="flex items-center gap-4">
        <div
          class="w-10 h-10 rounded-xl flex items-center justify-center shrink-0 text-blue-400 bg-blue-500/10"
        >
          <Server class="w-5 h-5" />
        </div>
        <div class="flex-1 min-w-0">
          <div class="text-[10px] font-bold text-(--text-muted) uppercase tracking-wider">
            Replicas
          </div>
          <div class="text-2xl font-bold mt-1 text-(--text-primary)">
            {{ totals.currentReplicas }}
            <span class="text-sm font-normal text-(--text-muted)"
              >/ {{ totals.desiredReplicas }}</span
            >
          </div>
        </div>
      </div>
      <div class="mt-4">
        <div class="flex justify-between text-[10px] text-(--text-muted) mb-1 font-mono">
          <span
            >{{ Math.round((totals.currentReplicas / totals.desiredReplicas) * 100) }}%
            desired</span
          >
        </div>
        <div class="w-full h-1.5 rounded-full bg-(--bg-hover) overflow-hidden">
          <div
            class="h-full rounded-full bg-blue-500"
            :style="{ width: (totals.currentReplicas / totals.desiredReplicas) * 100 + '%' }"
          ></div>
        </div>
      </div>
    </div>

    <!-- Card 3: Up to Date Replicas -->
    <div
      class="bg-(--bg-card) border border-(--border) rounded-xl p-5 flex flex-col justify-between shadow-sm transition-all duration-200 hover:border-(--border-strong)"
    >
      <div class="flex items-center gap-4">
        <div
          class="w-10 h-10 rounded-xl flex items-center justify-center shrink-0 text-cyan-400 bg-cyan-500/10"
        >
          <TrendingUp class="w-5 h-5" />
        </div>
        <div class="flex-1 min-w-0">
          <div class="text-[10px] font-bold text-(--text-muted) uppercase tracking-wider">
            Up-To-Date
          </div>
          <div class="text-2xl font-bold mt-1 text-(--text-primary)">
            {{ totals.upToDateReplicas }}
            <span class="text-sm font-normal text-(--text-muted)"
              >/ {{ totals.currentReplicas }}</span
            >
          </div>
        </div>
      </div>
      <div class="mt-4">
        <div class="flex justify-between text-[10px] text-(--text-muted) mb-1 font-mono">
          <span
            >{{ Math.round((totals.upToDateReplicas / totals.currentReplicas) * 100) }}%
            synchronized</span
          >
        </div>
        <div class="w-full h-1.5 rounded-full bg-(--bg-hover) overflow-hidden">
          <div
            class="h-full rounded-full bg-cyan-500"
            :style="{ width: (totals.upToDateReplicas / totals.currentReplicas) * 100 + '%' }"
          ></div>
        </div>
      </div>
    </div>

    <!-- Card 4: Available (Doughnut Chart) -->
    <div
      class="bg-(--bg-card) border border-(--border) rounded-xl p-5 flex flex-col justify-between shadow-sm transition-all duration-200 hover:border-(--border-strong)"
    >
      <div class="flex items-center gap-4">
        <div
          class="w-10 h-10 rounded-xl flex items-center justify-center shrink-0 text-emerald-400 bg-emerald-500/10"
        >
          <CheckCircle2 class="w-5 h-5" />
        </div>
        <div class="flex-1 min-w-0">
          <div class="text-[10px] font-bold text-(--text-muted) uppercase tracking-wider">
            Available
          </div>
          <div class="text-2xl font-bold mt-1 text-emerald-500">
            {{ totals.running }}
          </div>
        </div>
        <div
          class="w-10 h-10 relative shrink-0 flex items-center justify-center"
          v-if="availableChartData"
        >
          <Chart
            type="doughnut"
            :data="availableChartData"
            :options="miniChartOptions"
            class="w-full h-full"
          />
          <span class="absolute text-[9px] font-bold text-emerald-500"
            >{{ Math.round((totals.running / totals.total) * 100) }}%</span
          >
        </div>
      </div>
      <div class="mt-4 text-[10px] text-(--text-muted) font-medium">
        {{ totals.running }} of {{ totals.total }} deployments healthy
      </div>
    </div>

    <!-- Card 5: Progressing (Doughnut Chart) -->
    <div
      class="bg-(--bg-card) border border-(--border) rounded-xl p-5 flex flex-col justify-between shadow-sm transition-all duration-200 hover:border-(--border-strong)"
    >
      <div class="flex items-center gap-4">
        <div
          class="w-10 h-10 rounded-xl flex items-center justify-center shrink-0 text-amber-400 bg-amber-500/10"
        >
          <TrendingUp class="w-5 h-5" />
        </div>
        <div class="flex-1 min-w-0">
          <div class="text-[10px] font-bold text-(--text-muted) uppercase tracking-wider">
            Updating
          </div>
          <div class="text-2xl font-bold mt-1 text-amber-500">
            {{ totals.progressing }}
          </div>
        </div>
        <div
          class="w-10 h-10 relative shrink-0 flex items-center justify-center"
          v-if="progressingChartData"
        >
          <Chart
            type="doughnut"
            :data="progressingChartData"
            :options="miniChartOptions"
            class="w-full h-full"
          />
          <span class="absolute text-[9px] font-bold text-amber-500"
            >{{ Math.round((totals.progressing / totals.total) * 100) }}%</span
          >
        </div>
      </div>
      <div class="mt-4 text-[10px] text-(--text-muted) font-medium">
        {{ totals.progressing }} deployment rolling out
      </div>
    </div>

    <!-- Card 6: Failed (Doughnut Chart) -->
    <div
      class="bg-(--bg-card) border border-(--border) rounded-xl p-5 flex flex-col justify-between shadow-sm transition-all duration-200 hover:border-(--border-strong)"
    >
      <div class="flex items-center gap-4">
        <div
          class="w-10 h-10 rounded-xl flex items-center justify-center shrink-0 text-rose-400 bg-rose-500/10"
        >
          <AlertTriangle class="w-5 h-5" />
        </div>
        <div class="flex-1 min-w-0">
          <div class="text-[10px] font-bold text-(--text-muted) uppercase tracking-wider">
            Failed
          </div>
          <div class="text-2xl font-bold mt-1 text-rose-500">
            {{ totals.failed }}
          </div>
        </div>
        <div
          class="w-10 h-10 relative shrink-0 flex items-center justify-center"
          v-if="failedChartData"
        >
          <Chart
            type="doughnut"
            :data="failedChartData"
            :options="miniChartOptions"
            class="w-full h-full"
          />
          <span class="absolute text-[9px] font-bold text-rose-500"
            >{{ Math.round((totals.failed / totals.total) * 100) }}%</span
          >
        </div>
      </div>
      <div class="mt-4 text-[10px] text-rose-500 font-medium">Requires operator attention</div>
    </div>
  </div>
</template>
