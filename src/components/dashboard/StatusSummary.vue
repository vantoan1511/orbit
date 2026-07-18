<script setup lang="ts">
import { useKubernetesStore } from '@/stores/kubernetesStore'
import { CheckCircle2, Server, LayoutGrid, AlertTriangle, HelpCircle } from '@lucide/vue'
import { computed } from 'vue'

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
      bgClass: 'bg-slate-500/10',
      textClass: 'text-slate-400'
    }
  } else if (ready < total) {
    return {
      text: 'Degraded',
      subtext: `${total - ready} of ${total} nodes not ready`,
      icon: AlertTriangle,
      iconClass: 'text-amber-500',
      bgClass: 'bg-amber-500/10',
      textClass: 'text-amber-500'
    }
  } else {
    return {
      text: 'Healthy',
      subtext: 'All systems normal',
      icon: CheckCircle2,
      iconClass: 'text-emerald-500',
      bgClass: 'bg-emerald-500/10',
      textClass: 'text-emerald-500'
    }
  }
})
</script>

<template>
  <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
    <!-- Card 1: Cluster Status -->
    <div
      class="bg-(--bg-card) border border-(--border) rounded-xl p-5 flex items-center gap-5 shadow-sm transition-all duration-200 hover:border-(--border-strong)"
    >
      <div
        class="w-12 h-12 rounded-xl flex items-center justify-center shrink-0"
        :class="clusterStatusInfo.bgClass"
      >
        <component
          :is="clusterStatusInfo.icon"
          class="w-6 h-6"
          :class="clusterStatusInfo.iconClass"
        />
      </div>
      <div class="flex-1 min-w-0">
        <div class="text-xs font-semibold text-(--text-muted) uppercase tracking-wider">
          Cluster Status
        </div>
        <div class="text-2xl font-bold mt-1" :class="clusterStatusInfo.textClass">
          {{ clusterStatusInfo.text }}
        </div>
        <div class="text-xs text-(--text-muted) mt-0.5 truncate">
          {{ clusterStatusInfo.subtext }}
        </div>
      </div>
    </div>

    <!-- Card 2: Kubernetes Version -->
    <div
      class="bg-(--bg-card) border border-(--border) rounded-xl p-5 flex items-center gap-5 shadow-sm transition-all duration-200 hover:border-(--border-strong)"
    >
      <div
        class="w-12 h-12 rounded-xl bg-sky-500/10 flex items-center justify-center text-sky-500 shrink-0"
      >
        <!-- SVG Kubernetes Icon -->
        <svg class="w-6 h-6" viewBox="0 0 24 24" fill="currentColor">
          <path
            d="M12 2.69l5.66 3.27v6.54L12 15.77 6.34 12.5V5.96L12 2.69m0-1.8L5 4.96v7.88l7 4.04 7-4.04V4.96l-7-4.07zM12 17.5v5.69"
          />
          <path
            d="M5.34 13.5L2 11.58V5.37l7.34-4.24v6.23L5.34 9.48v4.02m0 2l-3.34-1.92V7.37L9.34 3.13v2.23L5.34 7.48v8.02z"
            class="opacity-70"
          />
        </svg>
      </div>
      <div class="flex-1 min-w-0">
        <div class="text-xs font-semibold text-(--text-muted) uppercase tracking-wider">
          Kubernetes Version
        </div>
        <div class="text-2xl font-bold text-(--text-primary) mt-1">
          {{ kubernetesVersion }}
        </div>
        <div class="text-xs text-(--text-muted) mt-0.5 truncate">Active Version</div>
      </div>
    </div>

    <!-- Card 3: Nodes -->
    <div
      class="bg-(--bg-card) border border-(--border) rounded-xl p-5 flex items-center gap-5 shadow-sm transition-all duration-200 hover:border-(--border-strong)"
    >
      <div
        class="w-12 h-12 rounded-xl bg-violet-500/10 flex items-center justify-center text-violet-400 shrink-0"
      >
        <Server class="w-6 h-6" />
      </div>
      <div class="flex-1 min-w-0">
        <div class="text-xs font-semibold text-(--text-muted) uppercase tracking-wider">Nodes</div>
        <div class="text-2xl font-bold text-(--text-primary) mt-1">
          {{ readyNodesCount }}
          <span class="text-base text-(--text-muted) font-normal">/ {{ totalNodesCount }}</span>
        </div>
        <div class="text-xs text-(--text-muted) mt-0.5 truncate">{{ readyNodesCount }} Ready</div>
      </div>
    </div>

    <!-- Card 4: Namespaces -->
    <div
      class="bg-(--bg-card) border border-(--border) rounded-xl p-5 flex items-center gap-5 shadow-sm transition-all duration-200 hover:border-(--border-strong)"
    >
      <div
        class="w-12 h-12 rounded-xl bg-amber-500/10 flex items-center justify-center text-amber-400 shrink-0"
      >
        <LayoutGrid class="w-6 h-6" />
      </div>
      <div class="flex-1 min-w-0">
        <div class="text-xs font-semibold text-(--text-muted) uppercase tracking-wider">
          Namespaces
        </div>
        <div class="text-2xl font-bold text-(--text-primary) mt-1">
          {{ namespacesCount }}
        </div>
        <div class="text-xs text-(--text-muted) mt-0.5 truncate">Active Namespaces</div>
      </div>
    </div>
  </div>
</template>
