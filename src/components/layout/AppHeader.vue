<script setup lang="ts">
import { useKubernetesStore } from '@/stores/kubernetesStore'
import { ChevronDown, Clock, Cloud, RefreshCw } from '@lucide/vue'
import { computed, ref } from 'vue'

const kubernetesStore = useKubernetesStore()
const isRefreshing = ref(false)

const activeCluster = computed(() => {
  return kubernetesStore.clusters.find((c) => c.id === kubernetesStore.activeClusterId) || null
})

const kubernetesVersion = computed(() => {
  return kubernetesStore.nodes[0]?.version || 'Unknown'
})

const clusterUptime = computed(() => {
  return kubernetesStore.nodes[0]?.uptime || 'Unknown'
})

const cloudProvider = computed(() => {
  const node = kubernetesStore.nodes[0]
  if (!node || !node.labels) {
    return { provider: 'Local', platform: 'Custom' }
  }
  const labels = node.labels
  let provider = 'Local'
  let platform = 'Custom'

  for (const label of labels) {
    const l = label.toLowerCase()
    if (l.includes('eks.amazonaws.com') || l.includes('aws')) {
      provider = 'AWS'
      platform = 'EKS'
      break
    } else if (l.includes('google.com') || l.includes('gke')) {
      provider = 'GCP'
      platform = 'GKE'
      break
    } else if (l.includes('azure') || l.includes('aks')) {
      provider = 'Azure'
      platform = 'AKS'
      break
    } else if (l.includes('minikube')) {
      provider = 'Minikube'
      platform = 'Local'
      break
    } else if (l.includes('k3s')) {
      provider = 'K3s'
      platform = 'Local'
      break
    } else if (l.includes('microk8s')) {
      provider = 'MicroK8s'
      platform = 'Local'
      break
    }
  }
  return { provider, platform }
})

const handleRefresh = async () => {
  if (isRefreshing.value) return
  isRefreshing.value = true
  try {
    await kubernetesStore.loadInitialData()
  } catch (error) {
    console.error('Failed to refresh cluster data:', error)
  } finally {
    isRefreshing.value = false
  }
}
</script>

<template>
  <header
    class="bg-(--bg-app) border-b border-(--border) px-8 py-4 flex flex-col gap-3 select-none"
  >
    <!-- Top Row -->
    <div class="flex items-center justify-between">
      <!-- Left side: Cluster info & status -->
      <div class="flex items-center gap-4">
        <h1 class="text-2xl font-bold text-(--text-primary) font-ui tracking-tight">
          {{ activeCluster?.name || 'Unknown Cluster' }}
        </h1>
        <div
          class="flex items-center gap-1.5 px-2 py-0.5 rounded-full text-xs font-semibold border transition-all duration-200"
          :class="
            activeCluster?.status === 'healthy'
              ? 'bg-emerald-500/10 text-emerald-500 border-emerald-500/20'
              : 'bg-rose-500/10 text-rose-500 border-rose-500/20'
          "
        >
          <span
            class="w-1.5 h-1.5 rounded-full"
            :class="activeCluster?.status === 'healthy' ? 'bg-emerald-500' : 'bg-rose-500'"
          ></span>
          <span>{{ activeCluster?.status === 'healthy' ? 'Healthy' : 'Offline' }}</span>
        </div>
      </div>

      <!-- Right side: Last updated, Refresh, Actions -->
      <div class="flex items-center gap-3">
        <span class="text-xs text-(--text-muted)"> Last updated: 1m ago </span>
        <button
          @click="handleRefresh"
          class="p-2 rounded-lg hover:bg-(--bg-hover) text-(--text-secondary) border border-(--border) transition-all duration-200"
          :class="{ 'animate-spin': isRefreshing }"
          title="Refresh"
        >
          <RefreshCw class="w-4 h-4" />
        </button>
        <button
          class="px-4 py-2 rounded-lg bg-(--bg-card) hover:bg-(--bg-hover) text-(--text-primary) border border-(--border) text-sm font-medium flex items-center gap-2 transition-all duration-200"
        >
          <span>Actions</span>
          <ChevronDown class="w-4 h-4 text-(--text-muted)" />
        </button>
      </div>
    </div>

    <!-- Bottom Row (Sub-metadata) -->
    <div class="flex items-center gap-6 text-xs text-(--text-secondary) font-medium">
      <!-- Kubernetes Version -->
      <div class="flex items-center gap-2">
        <!-- SVG Kubernetes Icon -->
        <svg class="w-4 h-4 text-sky-500" viewBox="0 0 24 24" fill="currentColor">
          <path
            d="M12 2.69l5.66 3.27v6.54L12 15.77 6.34 12.5V5.96L12 2.69m0-1.8L5 4.96v7.88l7 4.04 7-4.04V4.96l-7-4.07zM12 17.5v5.69"
          />
          <path
            d="M5.34 13.5L2 11.58V5.37l7.34-4.24v6.23L5.34 9.48v4.02m0 2l-3.34-1.92V7.37L9.34 3.13v2.23L5.34 7.48v8.02z"
            class="opacity-70"
          />
        </svg>
        <span>Kubernetes {{ kubernetesVersion }}</span>
      </div>

      <!-- Cloud Provider -->
      <div class="flex items-center gap-2 border-l border-(--border) pl-6">
        <Cloud class="w-4 h-4 text-orange-400" />
        <span class="font-semibold text-(--text-muted) uppercase text-[10px] tracking-wider">{{
          cloudProvider.provider
        }}</span>
        <span class="text-(--text-muted)">/</span>
        <span>{{ cloudProvider.platform }}</span>
      </div>

      <!-- Uptime -->
      <div class="flex items-center gap-2 border-l border-(--border) pl-6">
        <Clock class="w-4 h-4 text-indigo-400" />
        <span>Uptime: {{ clusterUptime }}</span>
      </div>
    </div>
  </header>
</template>
