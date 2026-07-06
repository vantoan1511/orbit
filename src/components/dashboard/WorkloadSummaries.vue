<script setup lang="ts">
import { Layers, Database, Box, ClipboardList, HardDrive, Award } from '@lucide/vue'

const items = [
  {
    title: 'Deployments',
    count: 52,
    icon: Layers,
    iconColor: 'text-[var(--deployment)] bg-[var(--deployment)]/10',
    statusLabel: 'Updated',
    statusVal: 3,
    statusColor: 'text-emerald-500 bg-emerald-500/10'
  },
  {
    title: 'StatefulSets',
    count: 12,
    icon: Database,
    iconColor: 'text-[var(--statefulset)] bg-[var(--statefulset)]/10',
    statusLabel: 'Ready',
    statusVal: 12,
    statusColor: 'text-emerald-500 bg-emerald-500/10'
  },
  {
    title: 'DaemonSets',
    count: 8,
    icon: Box,
    iconColor: 'text-[var(--daemonset)] bg-[var(--daemonset)]/10',
    statusLabel: 'Updated',
    statusVal: 8,
    statusColor: 'text-emerald-500 bg-emerald-500/10'
  },
  {
    title: 'Jobs',
    count: 24,
    icon: ClipboardList,
    iconColor: 'text-[var(--job)] bg-[var(--job)]/10',
    statusLabel: 'Active',
    statusVal: 2,
    statusColor: 'text-amber-500 bg-amber-500/10'
  },
  {
    title: 'Persistent Volumes',
    count: 28,
    icon: HardDrive,
    iconColor: 'text-[var(--text-muted)] bg-[var(--text-muted)]/10',
    statusLabel: 'Bound',
    statusVal: 26,
    statusColor: 'text-emerald-500 bg-emerald-500/10'
  },
  {
    title: 'Certificates',
    count: 3,
    icon: Award,
    iconColor: 'text-rose-500 bg-rose-500/10',
    statusLabel: 'Expiring',
    statusVal: 1,
    statusColor: 'text-amber-500 bg-amber-500/10'
  }
]
</script>

<template>
  <div class="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-6 gap-6">
    <div
      v-for="item in items"
      :key="item.title"
      class="bg-[var(--bg-card)] border border-[var(--border)] rounded-xl p-5 shadow-sm hover:border-[var(--border-strong)] transition-all duration-200 flex flex-col justify-between"
    >
      <!-- Top Section -->
      <div class="flex items-center justify-between mb-4">
        <!-- Icon wrapper -->
        <div class="p-2 rounded-lg" :class="item.iconColor">
          <component :is="item.icon" class="w-5 h-5" />
        </div>
      </div>

      <!-- Count and title -->
      <div>
        <div class="text-[10px] font-bold text-[var(--text-muted)] tracking-wider uppercase">
          {{ item.title }}
        </div>
        <div class="text-3xl font-bold text-[var(--text-primary)] mt-1 font-ui">
          {{ item.count }}
        </div>
      </div>

      <!-- Footer Info Badge -->
      <div
        class="mt-4 flex items-center justify-between text-xs border-t border-[var(--border)] pt-3"
      >
        <span class="text-[var(--text-muted)] font-medium">{{ item.statusLabel }}</span>
        <span class="px-2 py-0.5 rounded text-xs font-bold" :class="item.statusColor">
          {{ item.statusVal }}
        </span>
      </div>
    </div>
  </div>
</template>
