<script setup lang="ts">
import { computed } from 'vue'
import { Card } from 'primevue'
import { FileText, ShieldAlert, Activity, Boxes, FileWarning, Lock } from '@lucide/vue'
import { useKubernetesStore } from '@/stores/kubernetesStore'
import { storeToRefs } from 'pinia'

const props = defineProps<{
  activeTab: 'configmaps' | 'secrets'
}>()

const k8sStore = useKubernetesStore()
const { configMaps, secrets } = storeToRefs(k8sStore)

const parseSizeToBytes = (sizeStr: string): number => {
  const match = sizeStr.match(/^([\d.]+)\s*([A-Za-z]+)?$/)
  if (!match || !match[1]) return 0
  const value = parseFloat(match[1])
  const unit = match[2] ? match[2].toLowerCase() : ''
  if (unit === 'kib' || unit === 'ki') return value * 1024
  if (unit === 'mib' || unit === 'mi') return value * 1024 * 1024
  return value
}

const formatBytes = (bytes: number): string => {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KiB`
  return `${(bytes / (1024 * 1024)).toFixed(1)} MiB`
}

const metrics = computed(() => {
  const items = props.activeTab === 'configmaps' ? configMaps.value : secrets.value
  const uniqueNamespaces = new Set(items.map((item) => item.namespace)).size

  const totalBytes = items.reduce((acc, item) => acc + parseSizeToBytes(item.size), 0)
  const formattedTotalSize = formatBytes(totalBytes)

  const mountedCount = items.reduce((acc, item) => acc + (item.mountedPods > 0 ? 1 : 0), 0)
  const mountedPct = items.length > 0 ? ((mountedCount / items.length) * 100).toFixed(1) : '0'

  const orphanedCount = items.filter((item) => item.mountedPods === 0).length

  if (props.activeTab === 'configmaps') {
    return [
      {
        title: 'ConfigMaps',
        value: String(items.length),
        subtext: `Across ${uniqueNamespaces} ${uniqueNamespaces === 1 ? 'namespace' : 'namespaces'}`,
        icon: FileText,
        iconClass: 'bg-(--bg-hover) text-muted-color'
      },
      {
        title: 'Total Size',
        value: formattedTotalSize,
        subtext: 'Total memory footprint',
        subtextClass: 'text-(--success) font-medium',
        icon: Activity,
        iconClass: 'bg-(--success-soft) text-(--success)'
      },
      {
        title: 'Mounted in Pods',
        value: String(mountedCount),
        subtext: `${mountedPct}% of total`,
        subtextClass: 'text-(--accent) font-medium',
        icon: Boxes,
        iconClass: 'bg-(--accent-soft) text-(--accent)'
      },
      {
        title: 'Orphaned',
        value: String(orphanedCount),
        subtext: 'Not referenced',
        subtextClass: 'text-(--warning) font-medium',
        icon: FileWarning,
        iconClass: 'bg-(--warning-soft) text-(--warning)'
      }
    ]
  } else {
    return [
      {
        title: 'Secrets',
        value: String(items.length),
        subtext: `Across ${uniqueNamespaces} ${uniqueNamespaces === 1 ? 'namespace' : 'namespaces'}`,
        icon: Lock,
        iconClass: 'bg-(--bg-hover) text-muted-color'
      },
      {
        title: 'Total Size',
        value: formattedTotalSize,
        subtext: 'Total memory footprint',
        subtextClass: 'text-(--success) font-medium',
        icon: Activity,
        iconClass: 'bg-(--success-soft) text-(--success)'
      },
      {
        title: 'Mounted in Pods',
        value: String(mountedCount),
        subtext: `${mountedPct}% of total`,
        subtextClass: 'text-(--accent) font-medium',
        icon: Boxes,
        iconClass: 'bg-(--accent-soft) text-(--accent)'
      },
      {
        title: 'Orphaned',
        value: String(orphanedCount),
        subtext: 'Not referenced',
        subtextClass: 'text-(--danger) font-medium',
        icon: ShieldAlert,
        iconClass: 'bg-(--danger-soft) text-(--danger)'
      }
    ]
  }
})
</script>

<template>
  <div
    data-component="config-metrics-cards"
    class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-6"
  >
    <Card v-for="metric in metrics" :key="metric.title">
      <template #content>
        <div class="flex items-center gap-4">
          <div
            class="w-10 h-10 rounded-[var(--ui-radius-control)] flex items-center justify-center shrink-0"
            :class="metric.iconClass"
          >
            <component :is="metric.icon" class="w-5 h-5" />
          </div>
          <div class="flex-1 min-w-0">
            <div class="text-[11px] font-semibold text-muted-color uppercase tracking-wider">
              {{ metric.title }}
            </div>
            <div class="text-2xl font-bold text-primary font-mono mt-0.5">
              {{ metric.value }}
            </div>
            <div class="text-[10px] truncate" :class="metric.subtextClass || 'text-muted-color'">
              {{ metric.subtext }}
            </div>
          </div>
        </div>
      </template>
    </Card>
  </div>
</template>
