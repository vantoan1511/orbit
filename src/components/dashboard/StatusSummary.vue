<script setup lang="ts">
import { useKubernetesStore } from '@/stores/kubernetesStore'
import { AlertTriangle, CheckCircle2, HelpCircle, LayoutGrid, Server } from '@lucide/vue'
import { computed } from 'vue'
import { KubernetesIcon } from 'vue3-simple-icons'
import StatusCard from './StatusCard.vue'

const store = useKubernetesStore()

const totalNodesCount = computed(() => store.nodes.length)
const readyNodesCount = computed(() => store.nodes.filter((n) => n.status === 'Ready').length)
const namespacesCount = computed(() => store.namespaceList.length)
const kubernetesVersion = computed(() => store.nodes[0]?.version || 'Unknown')

type ClusterStatusKey = 'unknown' | 'degraded' | 'healthy'

const statusRecord: Record<
  ClusterStatusKey,
  { text: string; icon: typeof HelpCircle; iconClass: string; bgClass: string }
> = {
  unknown: {
    text: 'Unknown',
    icon: HelpCircle,
    iconClass: 'text-slate-400',
    bgClass: 'bg-slate-500/10'
  },
  degraded: {
    text: 'Degraded',
    icon: AlertTriangle,
    iconClass: 'text-amber-500',
    bgClass: 'bg-amber-500/10'
  },
  healthy: {
    text: 'Healthy',
    icon: CheckCircle2,
    iconClass: 'text-emerald-500',
    bgClass: 'bg-emerald-500/10'
  }
}

const clusterStatusInfo = computed(() => {
  const total = totalNodesCount.value
  const ready = readyNodesCount.value

  const statusKey: ClusterStatusKey =
    total === 0 ? 'unknown' : ready < total ? 'degraded' : 'healthy'

  const info = statusRecord[statusKey]
  const subtext =
    statusKey === 'unknown'
      ? 'No nodes found'
      : statusKey === 'degraded'
        ? `${total - ready} of ${total} nodes not ready`
        : 'All systems normal'

  return {
    ...info,
    subtext
  }
})

const cards = computed(() => [
  {
    label: 'Cluster Status',
    value: clusterStatusInfo.value.text,
    subtext: clusterStatusInfo.value.subtext,
    icon: clusterStatusInfo.value.icon,
    iconBgClass: clusterStatusInfo.value.bgClass,
    iconClass: clusterStatusInfo.value.iconClass
  },
  {
    label: 'Kubernetes Version',
    value: kubernetesVersion.value,
    subtext: 'Active Version',
    icon: KubernetesIcon,
    iconBgClass: 'bg-sky-500/10',
    iconClass: 'text-sky-500',
    valueClass: 'text-primary'
  },
  {
    label: 'Nodes',
    value: readyNodesCount.value,
    valueExtra: `/ ${totalNodesCount.value}`,
    subtext: `${readyNodesCount.value} Ready`,
    icon: Server,
    iconBgClass: 'bg-violet-500/10',
    iconClass: 'text-violet-400',
    valueClass: 'text-primary'
  },
  {
    label: 'Namespaces',
    value: namespacesCount.value,
    subtext: 'Active Namespaces',
    icon: LayoutGrid,
    iconBgClass: 'bg-amber-500/10',
    iconClass: 'text-amber-400',
    valueClass: 'text-primary'
  }
])
</script>

<template>
  <div class="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-4 gap-6">
    <StatusCard v-for="card in cards" :key="card.label" v-bind="card" />
  </div>
</template>
