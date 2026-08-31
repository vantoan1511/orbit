<script setup lang="ts">
import { useKubernetesStore } from '@/stores/kubernetesStore'
import { KUBERNETES_POD_STATUS } from '@/constants/kubernetes'
import { computed } from 'vue'

const store = useKubernetesStore()

// Data for Pod Health
const podHealth = computed(() => {
  const pods = store.pods
  const total = pods.length || 1 // prevent division by zero

  const runningCount = pods.filter((p) => p.status === KUBERNETES_POD_STATUS.Running).length
  const pendingCount = pods.filter((p) => p.status === KUBERNETES_POD_STATUS.Pending).length
  const failedCount = pods.filter((p) => p.status === KUBERNETES_POD_STATUS.Failed).length
  const crashLoopCount = pods.filter((p) =>
    p.status.includes(KUBERNETES_POD_STATUS.CrashLoopBackOff)
  ).length

  return {
    running: {
      count: runningCount,
      pct: Math.round((runningCount / total) * 100),
      color: 'bg-emerald-500',
      text: 'text-emerald-500',
      dot: 'bg-emerald-500'
    },
    pending: {
      count: pendingCount,
      pct: Math.round((pendingCount / total) * 100),
      color: 'bg-amber-500',
      text: 'text-amber-500',
      dot: 'bg-amber-500'
    },
    failed: {
      count: failedCount,
      pct: Math.round((failedCount / total) * 100),
      color: 'bg-rose-500',
      text: 'text-rose-500',
      dot: 'bg-rose-500'
    },
    crashLoop: {
      count: crashLoopCount,
      pct: Math.round((crashLoopCount / total) * 100),
      color: 'bg-red-600',
      text: 'text-red-600',
      dot: 'bg-red-600'
    }
  }
})

// Data for Node Health
const nodeHealth = computed(() => {
  const nodes = store.nodes
  const total = nodes.length || 1

  const readyCount = nodes.filter((n) => n.status === 'Ready').length
  const notReadyCount = nodes.filter((n) => n.status === 'NotReady').length
  const cordonedCount = nodes.filter((n) => n.isCordoned).length

  return {
    ready: {
      count: readyCount,
      pct: Math.round((readyCount / total) * 100),
      color: 'bg-emerald-500',
      text: 'text-emerald-500',
      dot: 'bg-emerald-500'
    },
    notReady: {
      count: notReadyCount,
      pct: Math.round((notReadyCount / total) * 100),
      color: 'bg-rose-500',
      text: 'text-rose-500',
      dot: 'bg-rose-500'
    },
    cordoned: {
      count: cordonedCount,
      pct: Math.round((cordonedCount / total) * 100),
      color: 'bg-sky-500',
      text: 'text-sky-500',
      dot: 'bg-sky-500'
    }
  }
})

const healthSections = computed(() => [
  {
    title: 'Pod Health',
    gridCols: 'grid-cols-4',
    items: [
      { label: KUBERNETES_POD_STATUS.Running, ...podHealth.value.running },
      { label: KUBERNETES_POD_STATUS.Pending, ...podHealth.value.pending },
      { label: KUBERNETES_POD_STATUS.Failed, ...podHealth.value.failed },
      { label: 'CrashLoop', ...podHealth.value.crashLoop }
    ]
  },
  {
    title: 'Node Health',
    gridCols: 'grid-cols-3',
    items: [
      { label: 'Ready', ...nodeHealth.value.ready },
      { label: 'NotReady', ...nodeHealth.value.notReady },
      { label: 'Cordoned', ...nodeHealth.value.cordoned }
    ]
  }
])
</script>

<template>
  <div class="flex flex-col gap-6">
    <Card v-for="section in healthSections" :key="section.title">
      <template #title>
        <div class="text-sm font-semibold text-primary uppercase tracking-wider">
          {{ section.title }}
        </div>
      </template>
      <template #content>
        <div class="grid gap-2" :class="section.gridCols">
          <div v-for="item in section.items" :key="item.label" class="flex flex-col">
            <div class="flex items-center gap-1.5 text-xs text-muted-color font-medium">
              <span class="w-2.5 h-2.5 rounded-full" :class="item.dot"></span>
              <span class="truncate">{{ item.label }}</span>
            </div>
            <div class="text-2xl font-bold mt-1.5 text-primary">
              {{ item.count }}
            </div>
            <div class="text-xs text-muted-color font-medium mt-0.5">{{ item.pct }}%</div>
          </div>
        </div>
      </template>
      <template #footer>
        <!-- Segmented Progress Bar -->
        <div class="w-full h-3 rounded-full overflow-hidden flex">
          <div
            v-for="item in section.items"
            :key="item.label"
            :style="{ width: item.pct + '%' }"
            :class="item.color"
            :title="item.label"
          ></div>
        </div>
      </template>
    </Card>
  </div>
</template>
