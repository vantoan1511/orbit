<script setup lang="ts">
import { computed } from 'vue'
import { Database, HardDrive, FileSpreadsheet, Layers } from '@lucide/vue'
import { useKubernetesStore } from '@/stores/kubernetesStore'

const k8sStore = useKubernetesStore()

const parseCapacityToGiB = (cap: string): number => {
  const num = parseFloat(cap)
  if (isNaN(num)) return 0
  if (cap.toLowerCase().includes('tib')) return num * 1024
  if (cap.toLowerCase().includes('gib')) return num
  if (cap.toLowerCase().includes('mib')) return num / 1024
  return num
}

const formatCapacity = (gib: number): string => {
  if (gib >= 1024) {
    return `${(gib / 1024).toFixed(1)} TiB`
  }
  return `${gib.toFixed(1)} GiB`
}

const metrics = computed(() => {
  const pvs = k8sStore.persistentVolumes
  const pvcs = k8sStore.persistentVolumeClaims
  const scs = k8sStore.storageClasses

  const totalPVs = pvs.length
  const totalPVCs = pvcs.length
  const totalStorageClasses = scs.length

  let totalCapGiB = 0
  let usedCapGiB = 0
  let availableCapGiB = 0

  pvs.forEach((pv) => {
    const cap = parseCapacityToGiB(pv.capacity)
    totalCapGiB += cap
    if (pv.status === 'Bound') {
      usedCapGiB += cap
    } else if (pv.status === 'Available') {
      availableCapGiB += cap
    }
  })

  const usedPct = totalCapGiB > 0 ? Math.round((usedCapGiB / totalCapGiB) * 100) : 0
  const availablePct = totalCapGiB > 0 ? Math.round((availableCapGiB / totalCapGiB) * 100) : 0

  return [
    {
      title: 'Total Capacity',
      value: formatCapacity(totalCapGiB),
      subtext: `${totalPVs} Total Volumes`,
      icon: Database,
      iconClass: 'bg-(--bg-hover) text-(--text-muted)'
    },
    {
      title: 'Used Capacity',
      value: formatCapacity(usedCapGiB),
      subtext: `${usedPct}% of cluster total`,
      subtextClass: 'text-violet-400 font-medium',
      icon: HardDrive,
      iconClass: 'bg-violet-500/10 text-violet-400'
    },
    {
      title: 'Available Capacity',
      value: formatCapacity(availableCapGiB),
      subtext: `${availablePct}% remaining`,
      subtextClass: 'text-emerald-400 font-medium',
      icon: HardDrive,
      iconClass: 'bg-emerald-500/10 text-emerald-400'
    },
    {
      title: 'Claims (PVCs)',
      value: totalPVCs.toString(),
      subtext: 'Bound to volumes',
      subtextClass: 'text-blue-400 font-medium',
      icon: FileSpreadsheet,
      iconClass: 'bg-blue-500/10 text-blue-400'
    },
    {
      title: 'Storage Classes',
      value: totalStorageClasses.toString(),
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
