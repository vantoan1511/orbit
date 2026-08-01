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

const clusterStatusInfo = computed(() => {
  const total = totalNodesCount.value
  const ready = readyNodesCount.value

  if (total === 0) {
    return {
      text: 'Unknown',
      subtext: 'No nodes found',
      icon: HelpCircle,
      iconClass: 'text-slate-400',
      bgClass: 'bg-slate-500/10'
    }
  } else if (ready < total) {
    return {
      text: 'Degraded',
      subtext: `${total - ready} of ${total} nodes not ready`,
      icon: AlertTriangle,
      iconClass: 'text-amber-500',
      bgClass: 'bg-amber-500/10'
    }
  } else {
    return {
      text: 'Healthy',
      subtext: 'All systems normal',
      icon: CheckCircle2,
      iconClass: 'text-emerald-500',
      bgClass: 'bg-emerald-500/10'
    }
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
    valueClass: 'text-(--text-primary)'
  },
  {
    label: 'Nodes',
    value: readyNodesCount.value,
    valueExtra: `/ ${totalNodesCount.value}`,
    subtext: `${readyNodesCount.value} Ready`,
    icon: Server,
    iconBgClass: 'bg-violet-500/10',
    iconClass: 'text-violet-400',
    valueClass: 'text-(--text-primary)'
  },
  {
    label: 'Namespaces',
    value: namespacesCount.value,
    subtext: 'Active Namespaces',
    icon: LayoutGrid,
    iconBgClass: 'bg-amber-500/10',
    iconClass: 'text-amber-400',
    valueClass: 'text-(--text-primary)'
  }
])
</script>

<template>
  <div class="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-4 gap-6">
    <StatusCard v-for="card in cards" :key="card.label" v-bind="card" />
  </div>
</template>
