<script setup lang="ts">
import AppBreadcrumbs from './AppBreadcrumbs.vue'
import FooterStatusAction from './FooterStatusAction.vue'
import { useCluster } from '@/composables/useCluster'
import { useEngineConnection } from '@/composables/useEngineConnection'
import { useKubernetesStore } from '@/stores/kubernetesStore'
import { useTerminalStore } from '@/stores/terminalStore'
import {
  AlertCircle,
  Loader2,
  RefreshCwIcon,
  RotateCcw,
  Terminal as TerminalIcon
} from '@lucide/vue'
import Button from 'primevue/button'
import { computed, onMounted, onUnmounted, ref } from 'vue'

const k8sStore = useKubernetesStore()
const terminalStore = useTerminalStore()
const { activeCluster, isRefreshing, refreshCluster, reconnectActiveCluster, lastUpdatedAt } =
  useCluster()
const { isReconnecting, isFailed, retryNow, restartApp } = useEngineConnection()

const isReconnectingCluster = ref(false)
const handleReconnectCluster = async () => {
  if (isReconnectingCluster.value) return
  isReconnectingCluster.value = true
  try {
    await reconnectActiveCluster()
  } finally {
    isReconnectingCluster.value = false
  }
}

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
</script>

<template>
  <footer
    data-component="app-footer"
    class="flex items-center justify-between gap-4 select-none bg-(--bg-sidebar) border-t border-(--border) px-3 py-1 text-[11px] font-medium shrink-0 z-20"
  >
    <!-- Left side: Breadcrumb slot -->
    <div class="flex items-center gap-2 min-w-0" data-testid="footer-breadcrumbs-placeholder">
      <AppBreadcrumbs />
    </div>

    <!-- Right side: Engine connection status & Last updated & Refresh -->
    <div class="flex items-center gap-3 shrink-0">
      <!-- Reconnecting indicator -->
      <div
        v-if="isReconnecting"
        class="flex items-center gap-1.5 text-[11px] text-(--warning) font-medium"
        v-tooltip.top="
          'Connection lost (e.g. PC sleep/idle). Re-establishing connection with core engine...'
        "
      >
        <Loader2 :size="12" class="animate-spin text-(--warning)" />
        <span>Reconnecting…</span>
        <Button
          rounded
          variant="text"
          size="small"
          @click="retryNow"
          v-tooltip.top="'Retry connection immediately'"
          class="p-0.5 w-5 h-5 text-(--warning)"
        >
          <template #icon>
            <RotateCcw :size="10" />
          </template>
        </Button>
      </div>

      <!-- Failed state with Restart App action -->
      <div
        v-else-if="isFailed"
        class="flex items-center gap-1.5 text-[11px] text-(--danger) font-medium"
      >
        <AlertCircle :size="12" class="text-(--danger)" />
        <span>Engine disconnected</span>
        <FooterStatusAction
          label="Restart app"
          severity="danger"
          tooltip="Restart application process to re-establish runtime engine link"
          @click="restartApp"
        />
      </div>

      <!-- Cluster watcher connection state (degraded: warning, suspended: danger + Reconnect) -->
      <template v-if="!isReconnecting && !isFailed && activeCluster">
        <div
          v-if="k8sStore.watcherConnectionState === 'degraded'"
          class="flex items-center gap-1.5 text-[11px] text-(--warning) font-medium"
          v-tooltip.top="'Cluster unreachable. Background watchers are backing off and retrying...'"
        >
          <Loader2 :size="12" class="animate-spin text-(--warning)" />
          <span>Cluster unreachable, retrying…</span>
        </div>

        <div
          v-else-if="k8sStore.watcherConnectionState === 'suspended'"
          class="flex items-center gap-1.5 text-[11px] text-(--danger) font-medium"
        >
          <AlertCircle :size="12" class="text-(--danger)" />
          <span>Cluster disconnected</span>
          <FooterStatusAction
            label="Reconnect"
            severity="danger"
            tooltip="Restart watchers and re-establish connection to cluster"
            :loading="isReconnectingCluster"
            @click="handleReconnectCluster"
          >
            <template #icon>
              <RotateCcw :size="10" />
            </template>
          </FooterStatusAction>
        </div>
      </template>

      <!-- Last updated & Refresh -->
      <template v-if="activeCluster">
        <span class="text-[11px] text-muted-color">Updated {{ lastUpdatedDisplay }}</span>
        <Button
          rounded
          variant="text"
          size="small"
          :loading="isRefreshing"
          :disabled="activeCluster === null"
          @click="refreshCluster"
          class="p-1 w-6 h-6"
        >
          <template #icon>
            <RefreshCwIcon :size="12" />
          </template>
        </Button>
      </template>

      <!-- Terminal toggle button -->
      <Button
        rounded
        variant="text"
        size="small"
        v-tooltip.top="'Toggle Terminal'"
        @click="terminalStore.togglePanel()"
        class="p-1 w-6 h-6"
      >
        <template #icon>
          <TerminalIcon :size="12" />
        </template>
      </Button>
    </div>
  </footer>
</template>
