<script setup lang="ts">
import { useKubernetesStore } from '@/stores/kubernetesStore'
import { Activity, Box, Cpu, Database, Server } from '@lucide/vue'
import { Card } from 'primevue'
import { computed } from 'vue'

const k8sStore = useKubernetesStore()

const totalNodes = computed(() => k8sStore.nodes.length)
const readyNodes = computed(() => k8sStore.nodes.filter((n) => n.status === 'Ready').length)
const onlinePct = computed(() =>
  totalNodes.value > 0 ? Math.round((readyNodes.value / totalNodes.value) * 100) : 0
)

const cpuMetrics = computed(() => {
  let used = 0
  let total = 0
  k8sStore.nodes.forEach((n) => {
    used += parseFloat(n.cpuUsed) || 0
    total += parseFloat(n.cpuTotal) || 0
  })
  const pct = total > 0 ? Math.round((used / total) * 100) : 0
  return { used, total, pct }
})

const memMetrics = computed(() => {
  let used = 0
  let total = 0
  k8sStore.nodes.forEach((n) => {
    used += parseFloat(n.memUsed) || 0
    total += parseFloat(n.memTotal) || 0
  })
  const pct = total > 0 ? Math.round((used / total) * 100) : 0
  return { used, total, pct }
})

const podMetrics = computed(() => {
  let count = 0
  let limit = 0
  k8sStore.nodes.forEach((n) => {
    count += n.podsCount || 0
    limit += n.podsLimit || 0
  })
  const pct = limit > 0 ? Math.round((count / limit) * 100) : 0
  return { count, limit, pct }
})

const allReady = computed(() => totalNodes.value > 0 && readyNodes.value === totalNodes.value)

const pressures = computed(() => [
  { name: 'Disk', ok: allReady.value },
  { name: 'Mem', ok: allReady.value },
  { name: 'PID', ok: allReady.value }
])
</script>

<template>
  <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-5 gap-6">
    <!-- Total Nodes -->
    <Card class="flex flex-col justify-between">
      <template #content>
        <div class="flex items-center gap-5">
          <div
            class="w-12 h-12 rounded-xl flex items-center justify-center shrink-0 bg-node/10 text-node"
          >
            <Server class="w-6 h-6" />
          </div>
          <div class="flex-1 min-w-0">
            <div class="text-xs font-semibold text-muted-color uppercase tracking-wider">
              Total Nodes
            </div>
            <div class="text-2xl font-bold mt-1 text-primary">
              {{ totalNodes }}
              <span class="text-base text-muted-color font-normal"> / {{ totalNodes }} </span>
            </div>
          </div>
        </div>
        <div class="mt-4 text-xs text-emerald-500 font-medium">
          {{ readyNodes }} Online ({{ onlinePct }}%)
        </div>
      </template>
    </Card>

    <!-- CPU Allocation -->
    <Card class="flex flex-col justify-between">
      <template #content>
        <div class="flex items-center gap-5">
          <div
            class="w-12 h-12 rounded-xl flex items-center justify-center shrink-0 bg-blue-500/10 text-blue-400"
          >
            <Cpu class="w-6 h-6" />
          </div>
          <div class="flex-1 min-w-0">
            <div class="text-xs font-semibold text-muted-color uppercase tracking-wider">
              CPU Allocation
            </div>
            <div class="text-2xl font-bold mt-1 text-primary">{{ cpuMetrics.pct }}%</div>
          </div>
        </div>
        <div class="mt-4">
          <div class="flex justify-between text-xs text-muted-color mb-1 font-mono">
            <span>{{ cpuMetrics.used.toFixed(1) }} / {{ cpuMetrics.total.toFixed(0) }} cores</span>
          </div>
          <div class="w-full h-1.5 rounded-full bg-(--bg-hover) overflow-hidden">
            <div
              class="h-full rounded-full bg-blue-500"
              :style="{ width: cpuMetrics.pct + '%' }"
            ></div>
          </div>
        </div>
      </template>
    </Card>

    <!-- Memory Allocation -->
    <Card class="flex flex-col justify-between">
      <template #content>
        <div class="flex items-center gap-5">
          <div
            class="w-12 h-12 rounded-xl flex items-center justify-center shrink-0 bg-indigo-500/10 text-indigo-400"
          >
            <Database class="w-6 h-6" />
          </div>
          <div class="flex-1 min-w-0">
            <div class="text-xs font-semibold text-muted-color uppercase tracking-wider">
              Memory Allocation
            </div>
            <div class="text-2xl font-bold mt-1 text-primary">{{ memMetrics.pct }}%</div>
          </div>
        </div>
        <div class="mt-4">
          <div class="flex justify-between text-xs text-muted-color mb-1 font-mono">
            <span>{{ memMetrics.used.toFixed(1) }} / {{ memMetrics.total.toFixed(0) }} GiB</span>
          </div>
          <div class="w-full h-1.5 rounded-full bg-(--bg-hover) overflow-hidden">
            <div
              class="h-full rounded-full bg-indigo-500"
              :style="{ width: memMetrics.pct + '%' }"
            ></div>
          </div>
        </div>
      </template>
    </Card>

    <!-- Pods Allocation -->
    <Card class="flex flex-col justify-between">
      <template #content>
        <div class="flex items-center gap-5">
          <div
            class="w-12 h-12 rounded-xl flex items-center justify-center shrink-0 bg-sky-500/10 text-sky-400"
          >
            <Box class="w-6 h-6" />
          </div>
          <div class="flex-1 min-w-0">
            <div class="text-xs font-semibold text-muted-color uppercase tracking-wider">
              Pods Allocation
            </div>
            <div class="text-2xl font-bold mt-1 text-primary">
              {{ podMetrics.count }}
              <span class="text-base text-muted-color font-normal"> / {{ podMetrics.limit }} </span>
            </div>
          </div>
        </div>
        <div class="mt-4">
          <div class="flex justify-between text-xs text-muted-color mb-1 font-mono">
            <span>{{ podMetrics.pct }}% Capacity</span>
          </div>
          <div class="w-full h-1.5 rounded-full bg-(--bg-hover) overflow-hidden">
            <div
              class="h-full rounded-full bg-sky-500"
              :style="{ width: podMetrics.pct + '%' }"
            ></div>
          </div>
        </div>
      </template>
    </Card>

    <!-- Node Pressures -->
    <Card class="flex flex-col justify-between">
      <template #content>
        <div class="flex items-center gap-5">
          <div
            class="w-12 h-12 rounded-xl flex items-center justify-center shrink-0"
            :class="
              allReady ? 'bg-emerald-500/10 text-emerald-400' : 'bg-amber-500/10 text-amber-400'
            "
          >
            <Activity class="w-6 h-6" />
          </div>
          <div class="flex-1 min-w-0">
            <div class="text-xs font-semibold text-muted-color uppercase tracking-wider">
              Node Pressures
            </div>
            <div
              class="text-2xl font-bold mt-1"
              :class="allReady ? 'text-emerald-500' : 'text-amber-500'"
            >
              {{ allReady ? 'Normal' : 'Warning' }}
            </div>
          </div>
        </div>
        <div class="mt-4">
          <div class="text-xs text-muted-color flex gap-2">
            <span v-for="p in pressures" :key="p.name" class="flex items-center gap-1">
              <span
                class="w-1.5 h-1.5 rounded-full"
                :class="p.ok ? 'bg-emerald-500' : 'bg-amber-500'"
              ></span>
              {{ p.name }}
            </span>
          </div>
        </div>
      </template>
    </Card>
  </div>
</template>
