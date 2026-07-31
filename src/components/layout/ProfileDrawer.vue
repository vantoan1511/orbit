<script setup lang="ts">
import { useCluster } from '@/composables/useCluster'
import { kubernetesService } from '@/services/kubernetesService'
import { useKubernetesStore } from '@/stores/kubernetesStore'
import { useProfileStore } from '@/stores/profileStore'
import {
  CircleCheck,
  ExternalLink,
  FileText,
  Plus,
  RefreshCw,
  Server,
  Settings,
  ShieldCheck,
  UserCheck
} from '@lucide/vue'
import Button from 'primevue/button'
import Drawer from 'primevue/drawer'
import { useRouter } from 'vue-router'

const profileStore = useProfileStore()
const k8sStore = useKubernetesStore()
const { activeCluster, isRefreshing, handleAddCluster } = useCluster()
const router = useRouter()

const handleSwitchCluster = async (clusterId: string) => {
  await kubernetesService.switchCluster(clusterId)
}

const handleOpenSettings = () => {
  profileStore.closeDrawer()
  router.push('/settings')
}

const getAuthTypeColorClass = (authType: string) => {
  switch (authType) {
    case 'Certificate':
      return 'text-emerald-400 bg-emerald-500/10 border-emerald-500/20'
    case 'Token':
      return 'text-sky-400 bg-sky-500/10 border-sky-500/20'
    case 'Exec Plugin':
      return 'text-amber-400 bg-amber-500/10 border-amber-500/20'
    case 'OIDC':
      return 'text-purple-400 bg-purple-500/10 border-purple-500/20'
    default:
      return 'text-(--text-muted) bg-(--bg-hover) border-(--border)'
  }
}
</script>

<template>
  <Drawer
    v-model:visible="profileStore.isDrawerOpen"
    position="right"
    class="w-full sm:max-w-md border-l border-(--border) bg-(--bg-card) p-0"
    :style="{ width: '28rem' }"
  >
    <template #header>
      <div class="flex items-center justify-between w-full pr-2">
        <div class="flex items-center gap-2">
          <span class="font-bold text-lg text-(--text-primary) font-ui">User Profile</span>
          <span
            v-if="profileStore.profile?.k8sVersion"
            class="text-xs font-semibold px-2 py-0.5 rounded-full border"
            :class="
              activeCluster?.status === 'healthy'
                ? 'bg-emerald-500/10 text-emerald-400 border-emerald-500/20'
                : 'bg-amber-500/10 text-amber-400 border-amber-500/20'
            "
          >
            {{ activeCluster?.status === 'healthy' ? 'Connected' : 'Offline' }}
          </span>
        </div>
      </div>
    </template>

    <div class="flex flex-col h-full overflow-y-auto p-6 space-y-6">
      <!-- Active Identity Card -->
      <div class="p-4 rounded-xl bg-(--bg-sidebar)/60 border border-(--border) space-y-4">
        <div class="flex items-center gap-3">
          <div class="p-2.5 rounded-lg bg-(--bg-card) border border-(--border) text-(--primary)">
            <UserCheck :size="20" />
          </div>
          <div class="flex-1 min-w-0">
            <div class="text-xs font-medium text-(--text-muted) uppercase tracking-wider">
              Kubernetes Identity
            </div>
            <div class="text-base font-bold text-(--text-primary) truncate">
              {{ profileStore.profile?.userName || 'Default Identity' }}
            </div>
          </div>
          <span
            v-if="profileStore.profile?.authType"
            class="text-xs px-2.5 py-1 rounded-md border font-mono font-medium"
            :class="getAuthTypeColorClass(profileStore.profile.authType)"
          >
            {{ profileStore.profile.authType }}
          </span>
        </div>

        <div class="space-y-2.5 pt-2 border-t border-(--border)/60 text-xs">
          <!-- Active Context -->
          <div class="flex items-center justify-between">
            <span class="text-(--text-muted) flex items-center gap-1.5">
              <ShieldCheck :size="14" />
              Active Context
            </span>
            <span class="font-mono font-medium text-(--text-primary) truncate max-w-48">
              {{ profileStore.profile?.activeContext || 'None' }}
            </span>
          </div>

          <!-- Active Cluster -->
          <div class="flex items-center justify-between">
            <span class="text-(--text-muted) flex items-center gap-1.5">
              <Server :size="14" />
              Cluster Name
            </span>
            <span class="font-mono font-medium text-(--text-primary) truncate max-w-48">
              {{ profileStore.profile?.clusterName || 'None' }}
            </span>
          </div>

          <!-- Server Endpoint -->
          <div v-if="profileStore.profile?.serverUrl" class="space-y-1">
            <span class="text-(--text-muted) flex items-center gap-1.5">
              <ExternalLink :size="14" />
              API Server Endpoint
            </span>
            <div
              class="p-2 rounded bg-(--bg-card) border border-(--border) font-mono text-[11px] text-(--text-secondary) truncate select-all"
            >
              {{ profileStore.profile.serverUrl }}
            </div>
          </div>
        </div>
      </div>

      <!-- Kubeconfig Sources -->
      <div class="space-y-2">
        <div class="text-xs font-bold text-(--text-muted) uppercase tracking-wider px-1">
          Loaded Kubeconfig Paths
        </div>
        <div
          v-if="
            profileStore.profile?.kubeconfigPaths && profileStore.profile.kubeconfigPaths.length > 0
          "
          class="space-y-1.5"
        >
          <div
            v-for="path in profileStore.profile.kubeconfigPaths"
            :key="path"
            class="flex items-center gap-2 p-2.5 rounded-lg bg-(--bg-sidebar)/40 border border-(--border) text-xs font-mono text-(--text-secondary) truncate"
          >
            <FileText :size="14" class="shrink-0 text-(--text-muted)" />
            <span class="truncate">{{ path }}</span>
          </div>
        </div>
        <div v-else class="text-xs text-(--text-muted) px-2 py-1">
          No custom kubeconfig paths loaded.
        </div>
      </div>

      <!-- Cluster Context Switcher -->
      <div class="space-y-3">
        <div class="flex items-center justify-between px-1">
          <span class="text-xs font-bold text-(--text-muted) uppercase tracking-wider">
            Available Clusters
          </span>
          <span class="text-xs text-(--text-muted) font-mono">
            {{ k8sStore.clusters.length }} total
          </span>
        </div>

        <div class="space-y-2 max-h-48 overflow-y-auto pr-1">
          <Button
            v-for="cluster in k8sStore.clusters"
            :key="cluster.id"
            :loading="k8sStore.activeClusterId === cluster.id && isRefreshing"
            :severity="k8sStore.activeClusterId === cluster.id ? 'primary' : 'secondary'"
            fluid
            class="truncate justify-between text-xs font-medium"
            :variant="k8sStore.activeClusterId === cluster.id ? 'filled' : 'outlined'"
            size="small"
            @click="handleSwitchCluster(cluster.id)"
          >
            <div class="flex items-center gap-2 truncate">
              <CircleCheck
                v-if="k8sStore.activeClusterId === cluster.id"
                :size="14"
                class="text-emerald-400"
              />
              <span class="truncate">{{ cluster.name }}</span>
            </div>
            <span
              v-if="k8sStore.activeClusterId === cluster.id"
              class="text-[10px] uppercase font-bold text-emerald-400"
            >
              Active
            </span>
          </Button>
        </div>

        <Button
          fluid
          severity="secondary"
          size="small"
          class="text-xs font-medium"
          @click="handleAddCluster"
        >
          <Plus :size="14" />
          <span>Add Cluster</span>
        </Button>
      </div>

      <!-- Quick Action Controls -->
      <div class="pt-4 border-t border-(--border) flex items-center gap-2">
        <Button
          fluid
          severity="secondary"
          size="small"
          class="text-xs font-medium"
          :loading="profileStore.isLoading"
          @click="profileStore.fetchProfile()"
        >
          <RefreshCw :size="14" />
          <span>Refresh</span>
        </Button>

        <Button
          fluid
          severity="secondary"
          size="small"
          class="text-xs font-medium"
          @click="handleOpenSettings"
        >
          <Settings :size="14" />
          <span>Settings</span>
        </Button>
      </div>
    </div>
  </Drawer>
</template>
