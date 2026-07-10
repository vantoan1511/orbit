<script setup lang="ts">
import { Activity, Box, Cpu, Database, Server } from '@lucide/vue'

const cards = [
  {
    title: 'Total Nodes',
    value: '12',
    subvalue: '/ 12',
    detail: '12 Online (100%)',
    icon: Server,
    iconColor: 'text-violet-400 bg-violet-500/10'
  },
  {
    title: 'CPU Allocation',
    value: '58%',
    detail: '27.8 / 48 cores',
    progress: 58,
    progressColor: 'bg-blue-500',
    icon: Cpu,
    iconColor: 'text-blue-400 bg-blue-500/10'
  },
  {
    title: 'Memory Allocation',
    value: '72%',
    detail: '138.2 / 192 GiB',
    progress: 72,
    progressColor: 'bg-indigo-500',
    icon: Database,
    iconColor: 'text-indigo-400 bg-indigo-500/10'
  },
  {
    title: 'Pods Allocation',
    value: '84',
    subvalue: '/ 120',
    detail: '70% Capacity',
    progress: 70,
    progressColor: 'bg-sky-500',
    icon: Box,
    iconColor: 'text-sky-400 bg-sky-500/10'
  },
  {
    title: 'Node Pressures',
    value: 'Normal',
    isPressure: true,
    pressures: [
      { name: 'Disk', ok: true },
      { name: 'Mem', ok: true },
      { name: 'PID', ok: true }
    ],
    icon: Activity,
    iconColor: 'text-emerald-400 bg-emerald-500/10'
  }
]
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
