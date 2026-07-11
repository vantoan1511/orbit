<script setup lang="ts">
import { computed } from 'vue'
import { useKubernetesStore } from '@/stores/kubernetesStore'
import { Activity, Box, Cpu, Database, Server } from '@lucide/vue'

const k8sStore = useKubernetesStore()

const cards = computed(() => {
  const nodes = k8sStore.nodes

  // Total Nodes
  const totalNodes = nodes.length
  const readyNodes = nodes.filter((n) => n.status === 'Ready').length
  const onlinePct = totalNodes > 0 ? Math.round((readyNodes / totalNodes) * 100) : 0

  // CPU
  let cpuUsed = 0
  let cpuTotal = 0
  // Memory
  let memUsed = 0
  let memTotal = 0
  // Pods
  let podsCount = 0
  let podsLimit = 0

  nodes.forEach((n) => {
    cpuUsed += parseFloat(n.cpuUsed) || 0
    cpuTotal += parseFloat(n.cpuTotal) || 0
    memUsed += parseFloat(n.memUsed) || 0
    memTotal += parseFloat(n.memTotal) || 0
    podsCount += n.podsCount || 0
    podsLimit += n.podsLimit || 0
  })

  const cpuPct = cpuTotal > 0 ? Math.round((cpuUsed / cpuTotal) * 100) : 0
  const memPct = memTotal > 0 ? Math.round((memUsed / memTotal) * 100) : 0
  const podsPct = podsLimit > 0 ? Math.round((podsCount / podsLimit) * 100) : 0

  const allReady = totalNodes > 0 && readyNodes === totalNodes

  return [
    {
      title: 'Total Nodes',
      value: String(totalNodes),
      subvalue: `/ ${totalNodes}`,
      detail: `${readyNodes} Online (${onlinePct}%)`,
      icon: Server,
      iconColor: 'text-violet-400 bg-violet-500/10'
    },
    {
      title: 'CPU Allocation',
      value: `${cpuPct}%`,
      detail: `${cpuUsed.toFixed(1)} / ${cpuTotal.toFixed(0)} cores`,
      progress: cpuPct,
      progressColor: 'bg-blue-500',
      icon: Cpu,
      iconColor: 'text-blue-400 bg-blue-500/10'
    },
    {
      title: 'Memory Allocation',
      value: `${memPct}%`,
      detail: `${memUsed.toFixed(1)} / ${memTotal.toFixed(0)} GiB`,
      progress: memPct,
      progressColor: 'bg-indigo-500',
      icon: Database,
      iconColor: 'text-indigo-400 bg-indigo-500/10'
    },
    {
      title: 'Pods Allocation',
      value: String(podsCount),
      subvalue: `/ ${podsLimit}`,
      detail: `${podsPct}% Capacity`,
      progress: podsPct,
      progressColor: 'bg-sky-500',
      icon: Box,
      iconColor: 'text-sky-400 bg-sky-500/10'
    },
    {
      title: 'Node Pressures',
      value: allReady ? 'Normal' : 'Warning',
      isPressure: true,
      pressures: [
        { name: 'Disk', ok: allReady },
        { name: 'Mem', ok: allReady },
        { name: 'PID', ok: allReady }
      ],
      icon: Activity,
      iconColor: allReady ? 'text-emerald-400 bg-emerald-500/10' : 'text-amber-400 bg-amber-500/10'
    }
  ]
})
</script>

<template>
  <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-5 gap-6">
    <div
      v-for="card in cards"
      :key="card.title"
      class="bg-(--bg-card) border border-(--border) rounded-xl p-5 flex flex-col justify-between shadow-sm transition-all duration-200 hover:border-(--border-strong)"
    >
      <!-- Top Section: Icon & Value -->
      <div class="flex items-center gap-5">
        <div
          class="w-12 h-12 rounded-xl flex items-center justify-center shrink-0"
          :class="card.iconColor"
        >
          <component :is="card.icon" class="w-6 h-6" />
        </div>
        <div class="flex-1 min-w-0">
          <div class="text-[10px] font-bold text-(--text-muted) uppercase tracking-wider">
            {{ card.title }}
          </div>
          <div
            class="text-2xl font-bold mt-1"
            :class="
              card.title === 'Node Pressures' ? 'text-emerald-500 text-lg' : 'text-(--text-primary)'
            "
          >
            {{ card.value }}
            <span v-if="card.subvalue" class="text-sm text-(--text-muted) font-normal">
              {{ card.subvalue }}
            </span>
          </div>
        </div>
      </div>

      <!-- Bottom Section: Progress or Detail -->
      <div class="mt-4">
        <template v-if="card.progress !== undefined">
          <div class="flex justify-between text-[10px] text-(--text-muted) mb-1 font-mono">
            <span>{{ card.detail }}</span>
          </div>
          <div class="w-full h-1.5 rounded-full bg-(--bg-hover) overflow-hidden">
            <div
              class="h-full rounded-full"
              :class="card.progressColor"
              :style="{ width: card.progress + '%' }"
            ></div>
          </div>
        </template>
        <template v-else-if="card.isPressure">
          <div class="text-[10px] text-(--text-muted) flex gap-2">
            <span v-for="p in card.pressures" :key="p.name" class="flex items-center gap-1">
              <span class="w-1.5 h-1.5 rounded-full bg-emerald-500"></span>{{ p.name }}
            </span>
          </div>
        </template>
        <template v-else>
          <div class="text-xs text-emerald-500 font-medium">{{ card.detail }}</div>
        </template>
      </div>
    </div>
  </div>
</template>
