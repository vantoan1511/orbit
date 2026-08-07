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
    class="px-6 py-3 flex items-center justify-between gap-4 select-none backdrop-blur-sm bg-surface-0/50 dark:bg-surface-950/50 border-b border-surface-200/60 dark:border-surface-800/60 sticky top-0 z-20"
  >
    <!-- Left side: Cluster info & inline metadata -->
    <div class="flex items-center gap-6 min-w-0">
      <div class="flex items-center gap-4 shrink-0">
        <template v-if="activeCluster !== null">
          <OverlayBadge :severity="activeCluster.status === 'healthy' ? 'success' : 'error'">
            <h1 class="pr-3 text-lg font-bold font-ui tracking-tight truncate">
              {{ activeCluster.name }}
            </h1>
          </OverlayBadge>
        </template>
        <h1 v-else class="text-lg font-bold text-muted-color font-ui tracking-tight">
          No active cluster
        </h1>
      </div>

      <!-- Metadata inline — only shown when cluster is active and healthy -->
      <div
        v-if="activeCluster !== null && activeCluster.status === 'healthy'"
        class="flex items-center gap-4 text-xs font-semibold tracking-wider text-muted-color shrink-0"
      >
        <div class="h-3 w-px bg-surface-300 dark:bg-surface-700"></div>

        <!-- Kubernetes Version -->
        <div class="flex items-center gap-1.5">
          <KubernetesIcon :size="13" />
          <span>K8s {{ kubernetesVersion }}</span>
        </div>

        <!-- Cloud Provider -->
        <div class="flex items-center gap-1.5">
          <Cloud :size="13" />
          <span class="uppercase">{{ cloudProvider.provider }}</span>
          <span>/</span>
          <span>{{ cloudProvider.platform }}</span>
        </div>

        <!-- Uptime -->
        <div class="flex items-center gap-1.5">
          <Clock :size="13" />
          <span>{{ clusterUptime }}</span>
        </div>
      </div>
    </div>

    <!-- Right side: Last updated & Refresh -->
    <div v-if="activeCluster" class="flex items-center gap-3 shrink-0">
      <span class="text-xs text-muted-color">Updated {{ lastUpdatedDisplay }}</span>
      <Button
        rounded
        variant="text"
        :loading="isRefreshing"
        :disabled="activeCluster === null"
        @click="refreshCluster"
      >
        <template #icon>
          <RefreshCwIcon :size="13" />
        </template>
      </Button>
    </div>
  </header>
</template>
