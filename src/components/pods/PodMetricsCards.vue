<script setup lang="ts">
import { useKubernetesStore } from '@/stores/kubernetesStore'
import { AlertTriangle, Box, CheckCircle2, HelpCircle, Loader2 } from '@lucide/vue'
import Chart from 'primevue/chart'
import { computed, onMounted, ref } from 'vue'

const k8sStore = useKubernetesStore()

const totalPods = computed(() => k8sStore.pods.length)
const maxPods = computed(() => Math.max(k8sStore.nodes.length * 110, 110))

const runningCount = computed(() => k8sStore.pods.filter((p) => p.status === 'Running').length)
const pendingCount = computed(() => k8sStore.pods.filter((p) => p.status === 'Pending').length)
const failedCount = computed(() => k8sStore.pods.filter((p) => p.status === 'Failed').length)
const unknownCount = computed(() => {
  const recognized = ['Running', 'Pending', 'Failed']
  return k8sStore.pods.filter((p) => !recognized.includes(p.status)).length
})

// Chart config and options
const sparklineOptions = ref()

onMounted(() => {
  sparklineOptions.value = {
    responsive: true,
    maintainAspectRatio: false,
    plugins: {
      legend: { display: false },
      tooltip: { enabled: false }
    },
    scales: {
      x: { display: false },
      y: { display: false }
    },
    elements: {
      point: { radius: 0, hoverRadius: 0 },
      line: { tension: 0.4, borderWidth: 1.5 }
    }
  }
})

const runningChartData = computed(() => {
  const current = runningCount.value
  const base = current > 10 ? current - 5 : current
  const data = [base - 2, base + 2, base - 1, base + 3, base, base - 2, current]
  return {
    labels: ['1', '2', '3', '4', '5', '6', '7'],
    datasets: [
      {
        data,
        borderColor: '#10b981', // emerald-500
        backgroundColor: 'transparent',
        fill: false
      }
    ]
  }
})

const pendingChartData = computed(() => {
  const current = pendingCount.value
  const base = current > 5 ? current - 2 : current
  const data = [base - 1, base + 1, base, base + 2, base - 1, base + 1, current]
  return {
    labels: ['1', '2', '3', '4', '5', '6', '7'],
    datasets: [
      {
        data,
        borderColor: '#f59e0b', // amber-500
        backgroundColor: 'transparent',
        fill: false
      }
    ]
  }
})

const failedChartData = computed(() => {
  const current = failedCount.value
  const base = current > 3 ? current - 1 : current
  const data = [base, base + 1, base - 1, base + 2, base, base - 1, current]
  return {
    labels: ['1', '2', '3', '4', '5', '6', '7'],
    datasets: [
      {
        data,
        borderColor: '#ef4444', // red-500
        backgroundColor: 'transparent',
        fill: false
      }
    ]
  }
})

const unknownChartData = computed(() => {
  const current = unknownCount.value
  const base = current > 2 ? current - 1 : current
  const data = [base, base + 1, base, base - 1, base + 1, base, current]
  return {
    labels: ['1', '2', '3', '4', '5', '6', '7'],
    datasets: [
      {
        data,
        borderColor: '#9ca3af', // gray-400
        backgroundColor: 'transparent',
        fill: false
      }
    ]
  }
})
</script>

<template>
  <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-5 gap-6">
    <!-- Card 1: Total Pods -->
    <div
      class="bg-(--bg-card) border border-(--border) rounded-xl p-5 flex flex-col justify-between shadow-sm transition-all duration-200 hover:border-(--border-strong)"
    >
      <div class="flex items-center gap-4">
        <div
          class="w-10 h-10 rounded-xl flex items-center justify-center shrink-0 text-violet-400 bg-violet-500/10"
        >
          <Box class="w-5 h-5" />
        </div>
        <div class="flex-1 min-w-0">
          <div class="text-[10px] font-bold text-(--text-muted) uppercase tracking-wider">
            Total Pods
          </div>
          <div class="text-2xl font-bold mt-1 text-(--text-primary)">
            {{ totalPods }}
            <span class="text-sm font-normal text-(--text-muted)">/ {{ maxPods }} [■]</span>
          </div>
        </div>
      </div>
      <div class="mt-4">
        <div class="flex justify-between text-[10px] text-(--text-muted) mb-1 font-mono">
          <span>{{ Math.round((totalPods / maxPods) * 100) }}% Limit</span>
        </div>
        <div class="w-full h-1.5 rounded-full bg-(--bg-hover) overflow-hidden">
          <div
            class="h-full rounded-full bg-violet-500"
            :style="{ width: (totalPods / maxPods) * 100 + '%' }"
          ></div>
        </div>
      </div>
    </div>

    <!-- Card 2: Running -->
    <div
      class="bg-(--bg-card) border border-(--border) rounded-xl p-5 flex flex-col justify-between shadow-sm transition-all duration-200 hover:border-(--border-strong)"
    >
      <div class="flex items-center justify-between gap-4">
        <div class="flex items-center gap-4">
          <div
            class="w-10 h-10 rounded-xl flex items-center justify-center shrink-0 text-emerald-400 bg-emerald-500/10"
          >
            <CheckCircle2 class="w-5 h-5" />
          </div>
          <div class="flex-1 min-w-0">
            <div class="text-[10px] font-bold text-(--text-muted) uppercase tracking-wider">
              Running
            </div>
            <div class="text-2xl font-bold mt-1 text-emerald-500">
              {{ runningCount }}
            </div>
          </div>
        </div>
        <div class="w-16 h-8 shrink-0" v-if="runningChartData">
          <Chart
            type="line"
            :data="runningChartData"
            :options="sparklineOptions"
            class="w-full h-full"
          />
        </div>
      </div>
      <div class="mt-4 text-[10px] text-(--text-muted) font-medium">
        {{ Math.round((runningCount / totalPods) * 1000) / 10 }}% of total workloads active
      </div>
    </div>

    <!-- Card 3: Pending -->
    <div
      class="bg-(--bg-card) border border-(--border) rounded-xl p-5 flex flex-col justify-between shadow-sm transition-all duration-200 hover:border-(--border-strong)"
    >
      <div class="flex items-center justify-between gap-4">
        <div class="flex items-center gap-4">
          <div
            class="w-10 h-10 rounded-xl flex items-center justify-center shrink-0 text-amber-400 bg-amber-500/10"
          >
            <Loader2 class="w-5 h-5 animate-spin" />
          </div>
          <div class="flex-1 min-w-0">
            <div class="text-[10px] font-bold text-(--text-muted) uppercase tracking-wider">
              Pending
            </div>
            <div class="text-2xl font-bold mt-1 text-amber-500">
              {{ pendingCount }}
            </div>
          </div>
        </div>
        <div class="w-16 h-8 shrink-0" v-if="pendingChartData">
          <Chart
            type="line"
            :data="pendingChartData"
            :options="sparklineOptions"
            class="w-full h-full"
          />
        </div>
      </div>
      <div class="mt-4 text-[10px] text-(--text-muted) font-medium">
        {{ Math.round((pendingCount / totalPods) * 1000) / 10 }}% scheduler queues occupied
      </div>
    </div>

    <!-- Card 4: Failed -->
    <div
      class="bg-(--bg-card) border border-(--border) rounded-xl p-5 flex flex-col justify-between shadow-sm transition-all duration-200 hover:border-(--border-strong)"
    >
      <div class="flex items-center justify-between gap-4">
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
              {{ failedCount }}
            </div>
          </div>
        </div>
        <div class="w-16 h-8 shrink-0" v-if="failedChartData">
          <Chart
            type="line"
            :data="failedChartData"
            :options="sparklineOptions"
            class="w-full h-full"
          />
        </div>
      </div>
      <div class="mt-4 text-[10px] text-rose-500 font-medium">Requires operator intervention</div>
    </div>

    <!-- Card 5: Unknown -->
    <div
      class="bg-(--bg-card) border border-(--border) rounded-xl p-5 flex flex-col justify-between shadow-sm transition-all duration-200 hover:border-(--border-strong)"
    >
      <div class="flex items-center justify-between gap-4">
        <div class="flex items-center gap-4">
          <div
            class="w-10 h-10 rounded-xl flex items-center justify-center shrink-0 text-gray-400 bg-gray-500/10"
          >
            <HelpCircle class="w-5 h-5" />
          </div>
          <div class="flex-1 min-w-0">
            <div class="text-[10px] font-bold text-(--text-muted) uppercase tracking-wider">
              Unknown
            </div>
            <div class="text-2xl font-bold mt-1 text-gray-400">
              {{ unknownCount }}
            </div>
          </div>
        </div>
        <div class="w-16 h-8 shrink-0" v-if="unknownChartData">
          <Chart
            type="line"
            :data="unknownChartData"
            :options="sparklineOptions"
            class="w-full h-full"
          />
        </div>
      </div>
      <div class="mt-4 text-[10px] text-(--text-muted) font-medium">
        Unreachable or lost nodes status
      </div>
    </div>
  </div>
</template>
