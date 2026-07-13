<script setup lang="ts">
import { computed } from 'vue'
import { Layers, Database, Box, ClipboardList, HardDrive, Clock } from '@lucide/vue'
import { useKubernetesStore } from '@/stores/kubernetesStore'

const store = useKubernetesStore()

const items = computed(() => {
  return [
    {
      title: 'Deployments',
      count: store.deployments.length,
      icon: Layers,
      iconColor: 'text-[var(--deployment)] bg-[var(--deployment)]/10',
      statusLabel: 'Available',
      statusVal: store.deployments.reduce((acc, d) => acc + d.available, 0),
      statusColor: 'text-emerald-500 bg-emerald-500/10'
    },
    {
      title: 'StatefulSets',
      count: store.statefulSets.length,
      icon: Database,
      iconColor: 'text-[var(--statefulset)] bg-[var(--statefulset)]/10',
      statusLabel: 'Current',
      statusVal: store.statefulSets.reduce((acc, s) => acc + s.replicas.current, 0),
      statusColor: 'text-emerald-500 bg-emerald-500/10'
    },
    {
      title: 'DaemonSets',
      count: store.daemonSets.length,
      icon: Box,
      iconColor: 'text-[var(--daemonset)] bg-[var(--daemonset)]/10',
      statusLabel: 'Ready',
      statusVal: store.daemonSets.reduce((acc, d) => acc + d.replicas.ready, 0),
      statusColor: 'text-emerald-500 bg-emerald-500/10'
    },
    {
      title: 'Jobs',
      count: store.jobs.length,
      icon: ClipboardList,
      iconColor: 'text-[var(--job)] bg-[var(--job)]/10',
      statusLabel: 'Completed',
      statusVal: store.jobs.filter((j) => j.status === 'Complete' || j.status === 'Completed')
        .length,
      statusColor: 'text-emerald-500 bg-emerald-500/10'
    },
    {
      title: 'Persistent Volumes',
      count: store.persistentVolumes.length,
      icon: HardDrive,
      iconColor: 'text-[var(--text-muted)] bg-[var(--text-muted)]/10',
      statusLabel: 'Bound',
      statusVal: store.persistentVolumes.filter((pv) => pv.status === 'Bound').length,
      statusColor: 'text-emerald-500 bg-emerald-500/10'
    },
    {
      title: 'CronJobs',
      count: store.cronJobs.length,
      icon: Clock,
      iconColor: 'text-rose-500 bg-rose-500/10',
      statusLabel: 'Active',
      statusVal: store.cronJobs.reduce((acc, c) => acc + c.active, 0),
      statusColor: 'text-amber-500 bg-amber-500/10'
    }
  ]
})
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
