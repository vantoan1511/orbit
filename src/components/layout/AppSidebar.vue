<script setup lang="ts">
import { useCluster } from '@/composables/useCluster'
import { useTheme } from '@/composables/useTheme'
import { kubernetesService } from '@/services/kubernetesService'
import { os } from '@/services/nativeService'
import { useKubernetesStore } from '@/stores/kubernetesStore'
import { useNotificationStore } from '@/stores/notificationStore'
import { useProfileStore } from '@/stores/profileStore'
import { VERSION } from '@/version'
import {
  Activity,
  Box,
  Boxes,
  Check,
  FileText,
  FolderOpen,
  HardDrive,
  LayoutDashboard,
  Network,
  Plus,
  Server,
  Settings,
  Settings2,
  ShieldCheck
} from '@lucide/vue'
import { Button } from 'primevue'
import { useRoute } from 'vue-router'

const k8sStore = useKubernetesStore()
const notificationStore = useNotificationStore()
const profileStore = useProfileStore()
const { activeCluster, isRefreshing, handleAddCluster } = useCluster()

const handleSwitchCluster = async (clusterId: string) => {
  await kubernetesService.switchCluster(clusterId)
}

// Navigation links
const navLinks = [
  { name: 'Overview', icon: LayoutDashboard, path: '/' },
  { name: 'Logs', icon: FileText, path: '/logs' },
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
  <aside class="w-64 flex flex-col h-screen text-primary select-none">
    <!-- Brand Header -->
    <div class="h-16 px-6 flex items-center gap-3">
      <!-- Orbit Icon Logo -->
      <img src="/logo.png" alt="Orbit Logo" class="w-8 h-8 object-contain" />

      <span class="text-xl font-bold tracking-tight font-ui">Orbit</span>
    </div>

    <!-- Clusters Section -->
    <div class="p-4">
      <div class="text-sm font-bold text-muted-color tracking-wider uppercase mb-2 px-2">
        Clusters
      </div>
      <div class="flex flex-col gap-2">
        <Button
          v-for="cluster in k8sStore.clusters"
          :key="cluster.id"
          :loading="k8sStore.activeClusterId === cluster.id && isRefreshing"
          :severity="
            k8sStore.activeClusterId === cluster.id
              ? activeCluster?.status === 'healthy'
                ? 'success'
                : 'danger'
              : 'secondary'
          "
          fluid
          class="truncate justify-start font-semibold"
          variant="text"
          @click="handleSwitchCluster(cluster.id)"
        >
          <Check v-if="k8sStore.activeClusterId === cluster.id" :size="16" />
          {{ cluster.name }}
        </Button>

        <!-- Empty state when no clusters are configured -->
        <p v-if="k8sStore.clusters.length === 0" class="text-sm text-muted-color px-3 py-1">
          No clusters added yet
        </p>

        <Button fluid severity="contrast" size="small" @click="handleAddCluster">
          <Plus :size="16" />
          <span class="text-sm font-semibold">Add cluster</span>
        </Button>
      </div>
    </div>

    <!-- Navigation Section -->
    <nav class="flex-1 overflow-y-auto p-4 space-y-1">
      <Button
        v-for="link in navLinks"
        :key="link.name"
        v-slot="slotProps"
        as-child
        fluid
        variant="link"
      >
        <router-link
          :to="link.path"
          :class="[
            slotProps.class,
            route.path === link.path
              ? 'bg-primary-200! dark:bg-primary-700! border-l-primary! border-l-3! rounded-l-lg! translate-x-3'
              : 'text-muted-color hover:bg-surface-100 dark:hover:bg-surface-800',
            'flex! items-center! justify-start! transition-all duration-200'
          ]"
        >
          <component :is="link.icon" class="w-4 h-4 shrink-0" />
          <span
            :class="route.path === link.path ? 'font-bold!' : 'font-medium!'"
            class="text-nowrap"
          >
            {{ link.name }}
          </span>
        </router-link>
      </Button>
    </nav>

    <!-- Bottom Footer -->
    <div class="p-4 flex items-center justify-around">
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
        <div class="relative inline-flex">
          <Button
            rounded
            variant="text"
            icon="pi pi-bell"
            badge-severity="danger"
            :aria-label="'Notifications'"
            :badge="notificationStore.unreadCount.toString()"
            @click="notificationStore.toggleDrawer()"
          />
        </div>

        <!-- Profile -->
        <Button
          rounded
          variant="text"
          icon="pi pi-user"
          :aria-label="'User Profile'"
          @click="profileStore.toggleDrawer()"
        />
      </div>
    </div>
    <div class="flex items-center justify-center gap-2">
      <span class="text-xs text-muted-color font-mono">{{ VERSION }}</span>
    </div>
  </aside>
</template>
