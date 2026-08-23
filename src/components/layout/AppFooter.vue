<script setup lang="ts">
import { useCluster } from '@/composables/useCluster'
import { useKubernetesStore } from '@/stores/kubernetesStore'
import { detectCloudProvider } from '@/utils/cloudProvider'
import { Clock, Cloud, RefreshCwIcon } from '@lucide/vue'
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { KubernetesIcon } from 'vue3-simple-icons'

const kubernetesStore = useKubernetesStore()
const { activeCluster, isRefreshing, refreshCluster, lastUpdatedAt } = useCluster()

const now = ref(Date.now())
let nowTimer: number | undefined

onMounted(() => {
  nowTimer = window.setInterval(() => {
    now.value = Date.now()
  }, 10000)
})

onUnmounted(() => {
  if (nowTimer) {
    window.clearInterval(nowTimer)
  }
})

const kubernetesVersion = computed(() => {
  return kubernetesStore.nodes[0]?.version || 'Unknown'
})

const lastUpdatedDisplay = computed(() => {
  if (!lastUpdatedAt.value) return 'just now'
  const diffMs = Math.max(0, now.value - lastUpdatedAt.value.getTime())
  const seconds = Math.floor(diffMs / 1000)
  if (seconds < 60) return 'just now'
  const minutes = Math.floor(seconds / 60)
  if (minutes < 60) return `${minutes}m ago`
  const hours = Math.floor(minutes / 60)
  if (hours < 24) return `${hours}h ago`
  const days = Math.floor(hours / 24)
  return `${days}d ago`
})

const clusterUptime = computed(() => {
  return kubernetesStore.nodes[0]?.uptime || 'Unknown'
})

const cloudProvider = computed(() => {
  return detectCloudProvider(kubernetesStore.nodes[0]?.labels)
})
</script>

<template>
  <footer
    class="flex items-center justify-between gap-4 select-none bg-(--bg-sidebar) border-t border-(--border) px-3 py-1 text-[11px] font-medium shrink-0 z-20"
  >
    <!-- Left side: Cluster info & inline metadata -->
    <div class="flex items-center gap-4 min-w-0">
      <div class="flex items-center gap-2 shrink-0">
        <template v-if="activeCluster !== null">
          <span class="font-semibold tracking-tight truncate">
            {{ activeCluster.name }}
          </span>
        </template>
        <span v-else class="font-semibold text-muted-color tracking-tight">No active cluster</span>
      </div>

      <!-- Metadata inline — only shown when cluster is active and healthy -->
      <div
        v-if="activeCluster !== null && activeCluster.status === 'healthy'"
        class="flex items-center gap-3 text-[11px] font-medium tracking-wider text-muted-color uppercase shrink-0"
      >
        <div class="h-3 w-px bg-(--border)"></div>

        <!-- Kubernetes Version -->
        <div class="flex items-center gap-1.5">
          <KubernetesIcon :size="12" />
          <span>K8S {{ kubernetesVersion }}</span>
        </div>

        <!-- Cloud Provider -->
        <div class="flex items-center gap-1.5">
          <Cloud :size="12" />
          <span>{{ cloudProvider.provider }}</span>
          <span>/</span>
          <span>{{ cloudProvider.platform }}</span>
        </div>

        <!-- Uptime -->
        <div class="flex items-center gap-1.5">
          <Clock :size="12" />
          <span>{{ clusterUptime }}</span>
        </div>
      </div>
    </div>

    <!-- Right side: Last updated & Refresh -->
    <div v-if="activeCluster" class="flex items-center gap-2 shrink-0">
      <span class="text-[11px] text-muted-color">Updated {{ lastUpdatedDisplay }}</span>
      <Button
        rounded
        variant="text"
        size="small"
        :loading="isRefreshing"
        :disabled="activeCluster === null"
        @click="refreshCluster"
        class="p-1! w-6! h-6!"
      >
        <template #icon>
          <RefreshCwIcon :size="12" />
        </template>
      </Button>
    </div>
  </footer>
</template>
