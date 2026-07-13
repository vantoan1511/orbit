<script setup lang="ts">
import { computed } from 'vue'
import { useKubernetesStore } from '@/stores/kubernetesStore'

const store = useKubernetesStore()

// Data for Pod Health
const podHealth = computed(() => {
  const pods = store.pods
  const total = pods.length || 1 // prevent division by zero

  const runningCount = pods.filter((p) => p.status === 'Running').length
  const pendingCount = pods.filter((p) => p.status === 'Pending').length
  const failedCount = pods.filter((p) => p.status === 'Failed').length
  const crashLoopCount = pods.filter((p) => p.status.includes('CrashLoop')).length

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
</script>

<template>
  <div class="flex flex-col gap-6">
    <!-- Pod Health Card -->
    <div
      class="bg-[var(--bg-card)] border border-[var(--border)] rounded-xl p-5 shadow-sm transition-all duration-200 flex flex-col justify-between h-[13.5rem]"
    >
      <div>
        <div class="text-xs font-semibold text-[var(--text-primary)] uppercase tracking-wider mb-4">
          Pod Health
        </div>

        <!-- Metrics Grid -->
        <div class="grid grid-cols-4 gap-2 mb-6">
          <div class="flex flex-col">
            <div class="flex items-center gap-1.5 text-xs text-[var(--text-secondary)] font-medium">
              <span class="w-2.5 h-2.5 rounded-full" :class="podHealth.running.dot"></span>
              <span>Running</span>
            </div>
            <div class="text-xl font-bold mt-1.5 text-[var(--text-primary)]">
              {{ podHealth.running.count }}
            </div>
            <div class="text-[10px] text-[var(--text-muted)] font-medium mt-0.5">
              {{ podHealth.running.pct }}%
            </div>
          </div>

          <div class="flex flex-col">
            <div class="flex items-center gap-1.5 text-xs text-[var(--text-secondary)] font-medium">
              <span class="w-2.5 h-2.5 rounded-full" :class="podHealth.pending.dot"></span>
              <span>Pending</span>
            </div>
            <div class="text-xl font-bold mt-1.5 text-[var(--text-primary)]">
              {{ podHealth.pending.count }}
            </div>
            <div class="text-[10px] text-[var(--text-muted)] font-medium mt-0.5">
              {{ podHealth.pending.pct }}%
            </div>
          </div>

          <div class="flex flex-col">
            <div class="flex items-center gap-1.5 text-xs text-[var(--text-secondary)] font-medium">
              <span class="w-2.5 h-2.5 rounded-full" :class="podHealth.failed.dot"></span>
              <span>Failed</span>
            </div>
            <div class="text-xl font-bold mt-1.5 text-[var(--text-primary)]">
              {{ podHealth.failed.count }}
            </div>
            <div class="text-[10px] text-[var(--text-muted)] font-medium mt-0.5">
              {{ podHealth.failed.pct }}%
            </div>
          </div>

          <div class="flex flex-col">
            <div class="flex items-center gap-1.5 text-xs text-[var(--text-secondary)] font-medium">
              <span class="w-2.5 h-2.5 rounded-full" :class="podHealth.crashLoop.dot"></span>
              <span class="truncate">CrashLoop</span>
            </div>
            <div class="text-xl font-bold mt-1.5 text-[var(--text-primary)]">
              {{ podHealth.crashLoop.count }}
            </div>
            <div class="text-[10px] text-[var(--text-muted)] font-medium mt-0.5">
              {{ podHealth.crashLoop.pct }}%
            </div>
          </div>
        </div>
      </div>

      <!-- Segmented Progress Bar -->
      <div class="w-full h-3 rounded-full bg-[var(--bg-hover)] overflow-hidden flex">
        <div
          :style="{ width: podHealth.running.pct + '%' }"
          :class="podHealth.running.color"
          title="Running"
        ></div>
        <div
          :style="{ width: podHealth.pending.pct + '%' }"
          :class="podHealth.pending.color"
          title="Pending"
        ></div>
        <div
          :style="{ width: podHealth.failed.pct + '%' }"
          :class="podHealth.failed.color"
          title="Failed"
        ></div>
        <div
          :style="{ width: podHealth.crashLoop.pct + '%' }"
          :class="podHealth.crashLoop.color"
          title="CrashLoop"
        ></div>
      </div>
    </div>

    <!-- Node Health Card -->
    <div
      class="bg-[var(--bg-card)] border border-[var(--border)] rounded-xl p-5 shadow-sm transition-all duration-200 flex flex-col justify-between h-[13.5rem]"
    >
      <div>
        <div class="text-xs font-semibold text-[var(--text-primary)] uppercase tracking-wider mb-4">
          Node Health
        </div>

        <!-- Metrics Grid -->
        <div class="grid grid-cols-3 gap-4 mb-6">
          <div class="flex flex-col">
            <div class="flex items-center gap-1.5 text-xs text-[var(--text-secondary)] font-medium">
              <span class="w-2.5 h-2.5 rounded-full" :class="nodeHealth.ready.dot"></span>
              <span>Ready</span>
            </div>
            <div class="text-xl font-bold mt-1.5 text-[var(--text-primary)]">
              {{ nodeHealth.ready.count }}
            </div>
            <div class="text-[10px] text-[var(--text-muted)] font-medium mt-0.5">
              {{ nodeHealth.ready.pct }}%
            </div>
          </div>

          <div class="flex flex-col">
            <div class="flex items-center gap-1.5 text-xs text-[var(--text-secondary)] font-medium">
              <span class="w-2.5 h-2.5 rounded-full" :class="nodeHealth.notReady.dot"></span>
              <span>NotReady</span>
            </div>
            <div class="text-xl font-bold mt-1.5 text-[var(--text-primary)]">
              {{ nodeHealth.notReady.count }}
            </div>
            <div class="text-[10px] text-[var(--text-muted)] font-medium mt-0.5">
              {{ nodeHealth.notReady.pct }}%
            </div>
          </div>

          <div class="flex flex-col">
            <div class="flex items-center gap-1.5 text-xs text-[var(--text-secondary)] font-medium">
              <span class="w-2.5 h-2.5 rounded-full" :class="nodeHealth.cordoned.dot"></span>
              <span>Cordoned</span>
            </div>
            <div class="text-xl font-bold mt-1.5 text-[var(--text-primary)]">
              {{ nodeHealth.cordoned.count }}
            </div>
            <div class="text-[10px] text-[var(--text-muted)] font-medium mt-0.5">
              {{ nodeHealth.cordoned.pct }}%
            </div>
          </div>
        </div>
      </div>

      <!-- Segmented Progress Bar -->
      <div class="w-full h-3 rounded-full bg-[var(--bg-hover)] overflow-hidden flex">
        <div
          :style="{ width: nodeHealth.ready.pct + '%' }"
          :class="nodeHealth.ready.color"
          title="Ready"
        ></div>
        <div
          :style="{ width: nodeHealth.cordoned.pct + '%' }"
          :class="nodeHealth.cordoned.color"
          title="Cordoned"
        ></div>
      </div>
    </div>
  </div>
</template>
