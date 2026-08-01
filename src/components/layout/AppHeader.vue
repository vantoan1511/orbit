<script setup lang="ts">
import { useCluster } from '@/composables/useCluster'
import { useKubernetesStore } from '@/stores/kubernetesStore'
import { Clock, Cloud, RefreshCwIcon } from '@lucide/vue'
import { computed } from 'vue'
import { KubernetesIcon } from 'vue3-simple-icons'

const kubernetesStore = useKubernetesStore()
const { activeCluster, isRefreshing, refreshCluster, lastUpdatedAt } = useCluster()

const kubernetesVersion = computed(() => {
  return kubernetesStore.nodes[0]?.version || 'Unknown'
})

const lastUpdatedDisplay = computed(() => {
  if (!lastUpdatedAt.value) return 'just now'
  const diffMs = Date.now() - lastUpdatedAt.value.getTime()
  const minutes = Math.round(diffMs / 60000)
  return minutes > 0 ? `${minutes}m ago` : 'just now'
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
</script>

<template>
  <header
    class="px-8 py-4 flex flex-col gap-3 select-none backdrop-blur-sm bg-surface-0/50 dark:bg-surface-950/50 border-b border-surface-200/60 dark:border-surface-800/60 sticky top-0 z-20 transition-colors duration-200"
  >
    <!-- Top Row -->
    <div class="flex items-center justify-between">
      <!-- Left side: Cluster info & status -->
      <div class="flex items-center gap-4">
        <template v-if="activeCluster !== null">
          <OverlayBadge :severity="activeCluster.status === 'healthy' ? 'success' : 'error'">
            <h1 class="pr-3 text-2xl font-bold font-ui tracking-tight">
              {{ activeCluster.name }}
            </h1>
          </OverlayBadge>
        </template>
        <h1 v-else class="text-2xl font-bold text-(--text-muted) font-ui tracking-tight">
          No active cluster
        </h1>
      </div>

      <!-- Right side: Last updated, Refresh, Actions -->
      <div v-if="activeCluster" class="flex items-center gap-3">
        <span class="text-sm text-muted-color">Last updated: {{ lastUpdatedDisplay }}</span>
        <Button
          rounded
          variant="text"
          size="small"
          :loading="isRefreshing"
          :disabled="activeCluster === null"
          @click="refreshCluster"
        >
          <template #icon>
            <RefreshCwIcon :size="16" />
          </template>
        </Button>
      </div>
    </div>

    <!-- Bottom Row (Sub-metadata) — only shown when a cluster is active and healthy -->
    <div
      v-if="activeCluster !== null && activeCluster.status === 'healthy'"
      class="flex items-center gap-6 text-xs font-semibold tracking-wider"
    >
      <!-- Kubernetes Version -->
      <div class="flex items-center gap-2">
        <KubernetesIcon :size="13" />
        <span>Kubernetes {{ kubernetesVersion }}</span>
      </div>

      <!-- Cloud Provider -->
      <div class="flex items-center gap-2 border-l pl-6">
        <Cloud :size="13" />
        <span class="uppercase">{{ cloudProvider.provider }}</span>
        <span>/</span>
        <span>{{ cloudProvider.platform }}</span>
      </div>

      <!-- Uptime -->
      <div class="flex items-center gap-2 border-l pl-6">
        <Clock :size="13" />
        <span>Uptime: {{ clusterUptime }}</span>
      </div>
    </div>
  </header>
</template>
