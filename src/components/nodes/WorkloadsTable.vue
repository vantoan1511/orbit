<script setup lang="ts">
import Chart from 'primevue/chart'
import { computed, onMounted, ref } from 'vue'
import { mockWorkloads } from './mockWorkloads'

const activeSubTab = ref('all')

const workloads = ref(mockWorkloads)

const filteredWorkloads = computed(() => {
  if (activeSubTab.value === 'all') return workloads.value
  return workloads.value.filter((w) => w.type.toLowerCase() === activeSubTab.value.toLowerCase())
})

// Chart.js data config
const chartData = ref()
const chartOptions = ref()

onMounted(() => {
  const isDark = document.documentElement.classList.contains('my-app-dark')

  // Custom colors matching the CSS theme variables
  const runningColor = isDark ? '#46d16e' : '#28a745'
  const pendingColor = isDark ? '#ffc54d' : '#f4a100'
  const failedColor = isDark ? '#ff6b6b' : '#d64545'
  const hoverRunning = isDark ? 'rgba(70, 209, 110, 0.8)' : 'rgba(40, 167, 69, 0.8)'
  const hoverPending = isDark ? 'rgba(255, 197, 77, 0.8)' : 'rgba(244, 161, 0, 0.8)'
  const hoverFailed = isDark ? 'rgba(255, 107, 107, 0.8)' : 'rgba(214, 69, 69, 0.8)'

  // Count workload statuses
  const counts = workloads.value.reduce(
    (acc, cur) => {
      if (cur.status === 'Running') acc.running++
      else if (cur.status === 'Pending') acc.pending++
      else if (cur.status === 'Failed') acc.failed++
      return acc
    },
    { running: 0, pending: 0, failed: 0 }
  )

  chartData.value = {
    labels: ['Running', 'Pending', 'Failed'],
    datasets: [
      {
        data: [counts.running, counts.pending, counts.failed],
        backgroundColor: [runningColor, pendingColor, failedColor],
        hoverBackgroundColor: [hoverRunning, hoverPending, hoverFailed],
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
            size: 11,
            weight: '500'
          },
          padding: 15,
          usePointStyle: true,
          pointStyle: 'circle'
        }
      }
    }
  }
})
</script>

<template>
  <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
    <!-- Workloads List (Left 2/3) -->
    <div
      class="lg:col-span-2 bg-(--bg-card) border border-(--border) rounded-xl p-6 shadow-sm flex flex-col justify-between"
    >
      <div>
        <div class="flex items-center justify-between mb-5 flex-wrap gap-3">
          <div class="text-sm font-semibold text-(--text-primary) uppercase tracking-wider">
            Workloads Distribution
          </div>

          <!-- Sub-Tab Selectors -->
          <div class="flex rounded-lg bg-(--bg-hover) p-0.5 border border-(--border)">
            <button
              @click="activeSubTab = 'all'"
              class="px-3 py-1.5 text-xs font-bold rounded-md transition-all duration-200"
              :class="
                activeSubTab === 'all'
                  ? 'bg-(--bg-card) text-(--text-primary) shadow-sm'
                  : 'text-(--text-secondary) hover:text-(--text-primary)'
              "
            >
              All
            </button>
            <button
              @click="activeSubTab = 'deployment'"
              class="px-3 py-1.5 text-xs font-bold rounded-md transition-all duration-200"
              :class="
                activeSubTab === 'deployment'
                  ? 'bg-(--bg-card) text-(--text-primary) shadow-sm'
                  : 'text-(--text-secondary) hover:text-(--text-primary)'
              "
            >
              Deployments
            </button>
            <button
              @click="activeSubTab = 'statefulset'"
              class="px-3 py-1.5 text-xs font-bold rounded-md transition-all duration-200"
              :class="
                activeSubTab === 'statefulset'
                  ? 'bg-(--bg-card) text-(--text-primary) shadow-sm'
                  : 'text-(--text-secondary) hover:text-(--text-primary)'
              "
            >
              StatefulSets
            </button>
            <button
              @click="activeSubTab = 'daemonset'"
              class="px-3 py-1.5 text-xs font-bold rounded-md transition-all duration-200"
              :class="
                activeSubTab === 'daemonset'
                  ? 'bg-(--bg-card) text-(--text-primary) shadow-sm'
                  : 'text-(--text-secondary) hover:text-(--text-primary)'
              "
            >
              DaemonSets
            </button>
          </div>
        </div>

        <!-- DataTable -->
        <DataTable
          :value="filteredWorkloads"
          class="p-datatable-sm border border-(--border) rounded-lg overflow-hidden"
          tableClass="w-full text-left text-xs border-collapse"
        >
          <!-- Name Column -->
          <Column field="name" header="Name" class="font-medium p-3 text-(--text-primary)">
            <template #body="{ data }">
              <span class="font-semibold">{{ data.name }}</span>
            </template>
          </Column>

          <!-- Type Column -->
          <Column field="type" header="Type" class="p-3">
            <template #body="{ data }">
              <span
                class="px-2 py-0.5 rounded text-[10px] font-bold uppercase tracking-wider"
                :class="
                  data.type === 'Deployment'
                    ? 'bg-blue-500/10 text-blue-400 border border-blue-500/20'
                    : data.type === 'StatefulSet'
                      ? 'bg-cyan-500/10 text-cyan-400 border border-cyan-500/20'
                      : 'bg-orange-500/10 text-orange-400 border border-orange-500/20'
                "
              >
                {{ data.type }}
              </span>
            </template>
          </Column>

          <!-- Namespace Column -->
          <Column
            field="namespace"
            header="Namespace"
            class="p-3 font-medium text-(--text-secondary)"
          ></Column>

          <!-- Node Column -->
          <Column field="node" header="Node" class="p-3 font-mono text-(--text-muted)"></Column>

          <!-- Status Column -->
          <Column field="status" header="Status" class="p-3">
            <template #body="{ data }">
              <div class="flex items-center gap-1.5">
                <span
                  class="w-1.5 h-1.5 rounded-full"
                  :class="
                    data.status === 'Running'
                      ? 'bg-emerald-500'
                      : data.status === 'Pending'
                        ? 'bg-amber-500'
                        : 'bg-rose-500'
                  "
                ></span>
                <span
                  class="font-medium"
                  :class="
                    data.status === 'Running'
                      ? 'text-emerald-500'
                      : data.status === 'Pending'
                        ? 'text-amber-500'
                        : 'text-rose-500'
                  "
                >
                  {{ data.status }}
                </span>
              </div>
            </template>
          </Column>

          <!-- Resource Allocation -->
          <Column
            field="cpu"
            header="CPU Allocation"
            class="p-3 font-mono text-(--text-secondary)"
          ></Column>
          <Column
            field="memory"
            header="Memory Allocation"
            class="p-3 font-mono text-(--text-secondary)"
          ></Column>
        </DataTable>
      </div>
    </div>

    <!-- Workloads Status Donut (Right 1/3) -->
    <div
      class="bg-(--bg-card) border border-(--border) rounded-xl p-6 shadow-sm flex flex-col justify-between"
    >
      <div>
        <div class="text-sm font-semibold text-(--text-primary) uppercase tracking-wider mb-5">
          Workloads Status
        </div>

        <div class="relative h-60 w-full flex items-center justify-center mt-2">
          <!-- Donut chart component -->
          <Chart type="doughnut" :data="chartData" :options="chartOptions" class="w-full h-full" />

          <!-- Middle Total Label -->
          <div class="absolute flex flex-col items-center justify-center">
            <span class="text-3xl font-bold text-(--text-primary)">8</span>
            <span class="text-[10px] uppercase font-bold text-(--text-muted) tracking-wider"
              >Total Pods</span
            >
          </div>
        </div>
      </div>

      <div class="border-t border-(--border) pt-4 mt-4 grid grid-cols-3 text-center">
        <div>
          <div class="text-xs text-(--text-muted) font-medium">Running</div>
          <div class="text-lg font-bold text-emerald-500 mt-0.5">6</div>
        </div>
        <div class="border-l border-(--border)">
          <div class="text-xs text-(--text-muted) font-medium">Pending</div>
          <div class="text-lg font-bold text-amber-500 mt-0.5">1</div>
        </div>
        <div class="border-l border-(--border)">
          <div class="text-xs text-(--text-muted) font-medium">Failed</div>
          <div class="text-lg font-bold text-rose-500 mt-0.5">1</div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
:deep(.p-datatable) {
  background: transparent;
}
:deep(.p-datatable-thead > tr > th) {
  background: var(--bg-hover);
  color: var(--text-secondary);
  border-bottom: 1px solid var(--border);
  font-weight: 600;
  font-size: 0.75rem;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}
:deep(.p-datatable-tbody > tr) {
  background: transparent;
  color: var(--text-secondary);
  border-bottom: 1px solid rgba(var(--border), 0.5);
  transition: background-color 0.2s;
}
:deep(.p-datatable-tbody > tr:hover) {
  background: var(--bg-hover) !important;
}
</style>
