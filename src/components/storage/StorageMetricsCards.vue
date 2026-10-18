<script setup lang="ts">
import { computed } from 'vue'
import { Card } from 'primevue'
import { Database, HardDrive, FileSpreadsheet, Layers } from '@lucide/vue'
import { useKubernetesStore } from '@/stores/kubernetesStore'
import { KUBERNETES_VOLUME_STATUS } from '@/constants/kubernetes'

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
    if (pv.status === KUBERNETES_VOLUME_STATUS.Bound) {
      usedCapGiB += cap
    } else if (pv.status === KUBERNETES_VOLUME_STATUS.Available) {
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
      iconClass: 'bg-(--bg-hover) text-muted-color'
    },
    {
      title: 'Used Capacity',
      value: formatCapacity(usedCapGiB),
      subtext: `${usedPct}% of cluster total`,
      subtextClass: 'text-(--accent) font-medium',
      icon: HardDrive,
      iconClass: 'bg-(--accent-soft) text-(--accent)'
    },
    {
      title: 'Available Capacity',
      value: formatCapacity(availableCapGiB),
      subtext: `${availablePct}% remaining`,
      subtextClass: 'text-(--success) font-medium',
      icon: HardDrive,
      iconClass: 'bg-(--success-soft) text-(--success)'
    },
    {
      title: 'Claims (PVCs)',
      value: totalPVCs.toString(),
      subtext: 'Bound to volumes',
      subtextClass: 'text-(--accent) font-medium',
      icon: FileSpreadsheet,
      iconClass: 'bg-(--accent-soft) text-(--accent)'
    },
    {
      title: 'Storage Classes',
      value: totalStorageClasses.toString(),
      subtext: 'Provisioners configured',
      icon: Layers,
      iconClass: 'bg-(--warning-soft) text-(--warning)'
    }
  ]
})
</script>

<template>
  <div
    data-component="storage-metrics-cards"
    class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-5 gap-4"
  >
    <Card v-for="metric in metrics" :key="metric.title">
      <template #content>
        <div class="flex items-center gap-3">
          <div
            class="w-9 h-9 rounded-[var(--ui-radius-control)] flex items-center justify-center shrink-0"
            :class="metric.iconClass"
          >
            <component :is="metric.icon" class="w-4.5 h-4.5" />
          </div>
          <div class="flex-1 min-w-0">
            <div
              class="text-[10px] font-semibold text-muted-color uppercase tracking-wider truncate"
            >
              {{ metric.title }}
            </div>
            <div class="text-xl font-bold text-primary font-mono mt-0.5 truncate">
              {{ metric.value }}
            </div>
            <div class="text-[9px] truncate" :class="metric.subtextClass || 'text-muted-color'">
              {{ metric.subtext }}
            </div>
          </div>
        </div>
      </template>
    </Card>
  </div>
</template>
