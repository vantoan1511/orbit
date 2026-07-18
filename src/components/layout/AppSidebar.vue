<script setup lang="ts">
import { useCluster } from '@/composables/useCluster'
import { useTheme } from '@/composables/useTheme'
import { kubernetesService } from '@/services/kubernetesService'
import { os } from '@/services/nativeService'
import { useKubernetesStore } from '@/stores/kubernetesStore'
import { VERSION } from '@/version'
import {
  Activity,
  Box,
  Boxes,
  FolderOpen,
  HardDrive,
  LayoutDashboard,
  Network,
  Server,
  Settings,
  Settings2,
  ShieldCheck
} from '@lucide/vue'
import { Button } from 'primevue'
import { useRoute } from 'vue-router'

const k8sStore = useKubernetesStore()
const { activeCluster, isRefreshing, handleAddCluster } = useCluster()

const handleSwitchCluster = async (clusterId: string) => {
  await kubernetesService.switchCluster(clusterId)
}

// Navigation links
const navLinks = [
  { name: 'Overview', icon: LayoutDashboard, path: '/' },
  { name: 'Nodes', icon: Server, path: '/nodes' },
  { name: 'Workloads', icon: Boxes, path: '/workloads' },
  { name: 'Pods', icon: Box, path: '/pods' },
  { name: 'Services', icon: Network, path: '/services' },
  { name: 'ConfigMaps & Secrets', icon: Settings2, path: '/config' },
  { name: 'Storage', icon: HardDrive, path: '/storage' },
  { name: 'Namespaces', icon: FolderOpen, path: '/namespaces' },
  { name: 'Events', icon: Activity, path: '/events' },
  { name: 'Policies', icon: ShieldCheck, path: '/policies' },
  { name: 'Settings', icon: Settings, path: '/settings' }
]

const route = useRoute()

const { isDark, toggleTheme } = useTheme()
</script>

<template>
  <aside
    class="w-64 bg-(--bg-sidebar) border-r border-(--border) flex flex-col h-screen text-(--text-primary) select-none"
  >
    <!-- Brand Header -->
    <div class="h-16 px-6 flex items-center gap-3 border-b border-(--border)">
      <!-- Orbit Icon Logo -->
      <img src="/logo.png" alt="Orbit Logo" class="w-8 h-8 object-contain" />

      <span class="text-xl font-bold tracking-tight font-ui">Orbit</span>
    </div>

    <!-- Clusters Section -->
    <div class="p-4 border-b border-(--border)">
      <div class="text-[10px] font-bold text-(--text-muted) tracking-wider uppercase mb-2 px-2">
        Clusters
      </div>
      <div class="flex flex-col gap-2">
        <Button
          v-for="cluster in k8sStore.clusters"
          :key="cluster.id"
          :label="cluster.name"
          :loading="k8sStore.activeClusterId === cluster.id && isRefreshing"
          :icon="k8sStore.activeClusterId === cluster.id ? 'pi pi-circle-fill' : ''"
          :severity="
            k8sStore.activeClusterId === cluster.id
              ? activeCluster?.status === 'healthy'
                ? 'success'
                : 'danger'
              : 'secondary'
          "
          fluid
          class="truncate justify-start text-medium"
          variant="text"
          @click="handleSwitchCluster(cluster.id)"
        />

        <!-- Empty state when no clusters are configured -->
        <p v-if="k8sStore.clusters.length === 0" class="text-xs text-(--text-muted) px-3 py-1">
          No clusters added yet
        </p>

        <Button
          fluid
          icon="pi pi-plus"
          label="Add Cluster"
          severity="secondary"
          size="small"
          @click="handleAddCluster"
        />
      </div>
    </div>

    <!-- Navigation Section -->
    <nav class="flex-1 overflow-y-auto p-4 space-y-1">
      <router-link
        v-for="link in navLinks"
        :key="link.name"
        :to="link.path"
        class="flex items-center gap-3 px-3 py-2 rounded-lg text-sm transition-all duration-200"
        :class="[
          route.path === link.path
            ? 'bg-(--bg-active) font-medium text-(--text-primary)'
            : 'text-(--text-secondary) hover:bg-(--bg-hover)',
          k8sStore.activeClusterId === null && link.path !== '/settings'
            ? 'opacity-40 pointer-events-none'
            : ''
        ]"
      >
        <component :is="link.icon" class="w-4 h-4 shrink-0" />
        <span>{{ link.name }}</span>
      </router-link>
    </nav>

    <!-- Bottom Footer -->
    <div class="p-4 border-t border-(--border) flex items-center justify-around bg-(--bg-sidebar)">
      <div class="flex items-center gap-3">
        <!-- Theme Toggle -->
        <Button
          rounded
          variant="text"
          :icon="isDark ? 'pi pi-sun' : 'pi pi-moon'"
          @click="toggleTheme"
        />

        <!-- Docs -->
        <Button
          rounded
          variant="text"
          icon="pi pi-github"
          @click="os.open('https://github.com/vantoan1511/orbit')"
        />

        <!-- Notifications -->
        <Button rounded variant="text" icon="pi pi-bell" />

        <!-- Profile -->
        <Button rounded variant="text" icon="pi pi-user" />
      </div>
    </div>
    <div class="flex items-center justify-center gap-2">
      <span class="text-[10px] text-(--text-muted) font-mono">{{ VERSION }}</span>
    </div>
  </aside>
</template>
