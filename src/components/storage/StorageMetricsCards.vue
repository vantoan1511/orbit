<script setup lang="ts">
import { computed } from 'vue'
import { Database, HardDrive, FileSpreadsheet, Layers } from '@lucide/vue'
import { mockStorageMetrics } from './mockStorage'

const metrics = computed(() => {
  return [
    {
      title: 'Total Capacity',
      value: mockStorageMetrics.totalCapacity,
      subtext: `${mockStorageMetrics.totalPVs} Total Volumes`,
      icon: Database,
      iconClass: 'bg-(--bg-hover) text-(--text-muted)'
    },
    {
      title: 'Used Capacity',
      value: mockStorageMetrics.usedCapacity,
      subtext: '72% of cluster total',
      subtextClass: 'text-violet-400 font-medium',
      icon: HardDrive,
      iconClass: 'bg-violet-500/10 text-violet-400'
    },
    {
      title: 'Available Capacity',
      value: mockStorageMetrics.availableCapacity,
      subtext: '28% remaining',
      subtextClass: 'text-emerald-400 font-medium',
      icon: HardDrive,
      iconClass: 'bg-emerald-500/10 text-emerald-400'
    },
    {
      title: 'Claims (PVCs)',
      value: mockStorageMetrics.totalPVCs.toString(),
      subtext: 'Bound to volumes',
      subtextClass: 'text-blue-400 font-medium',
      icon: FileSpreadsheet,
      iconClass: 'bg-blue-500/10 text-blue-400'
    },
    {
      title: 'Storage Classes',
      value: mockStorageMetrics.totalStorageClasses.toString(),
      subtext: 'Provisioners configured',
      icon: Layers,
      iconClass: 'bg-amber-500/10 text-amber-400'
    }
  ]
})
</script>

<template>
  <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-5 gap-4">
    <div
      v-for="metric in metrics"
      :key="metric.title"
      class="bg-(--bg-card) border border-(--border) rounded-xl p-4 flex items-center gap-3 shadow-sm transition-all duration-200 hover:border-(--border-strong)"
    >
      <div
        class="w-9 h-9 rounded-lg flex items-center justify-center shrink-0"
        :class="metric.iconClass"
      >
        <component :is="metric.icon" class="w-4.5 h-4.5" />
      </div>
      <div class="min-w-0">
        <div
          class="text-[10px] font-semibold text-(--text-muted) uppercase tracking-wider truncate"
        >
          {{ metric.title }}
        </div>
        <div class="text-xl font-bold text-(--text-primary) font-mono mt-0.5 truncate">
          {{ metric.value }}
        </div>
        <div class="text-[9px] truncate" :class="metric.subtextClass || 'text-(--text-muted)'">
          {{ metric.subtext }}
        </div>
      </div>
    </div>
  </div>
</template>
