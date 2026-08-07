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
import { ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'

const k8sStore = useKubernetesStore()
const notificationStore = useNotificationStore()
const profileStore = useProfileStore()
const { activeCluster, isRefreshing, handleAddCluster } = useCluster()
const route = useRoute()
const router = useRouter()
const { isDark, toggleTheme } = useTheme()

const activeTab = ref<'resources' | 'clusters' | null>('resources')

const toggleTab = (tab: 'resources' | 'clusters') => {
  activeTab.value = activeTab.value === tab ? null : tab
}

const handleSwitchCluster = async (clusterId: string) => {
  await kubernetesService.switchCluster(clusterId)
  activeTab.value = 'resources'
}

// Navigation links for Resources
const navLinks = [
  { name: 'Overview', icon: LayoutDashboard, path: '/' },
  { name: 'Logs', icon: FileText, path: '/logs' },
  { name: 'Nodes', icon: Server, path: '/nodes' },
  { name: 'Workloads', icon: Boxes, path: '/workloads' },
  { name: 'Pods', icon: Box, path: '/pods' },
  { name: 'Network', icon: Network, path: '/network' },
  { name: 'ConfigMaps & Secrets', icon: Settings2, path: '/config' },
  { name: 'Storage', icon: HardDrive, path: '/storage' },
  { name: 'Namespaces', icon: FolderOpen, path: '/namespaces' },
  { name: 'Events', icon: Activity, path: '/events' },
  { name: 'Policies', icon: ShieldCheck, path: '/policies' }
]
</script>

<template>
  <aside class="flex h-screen text-primary select-none bg-surface-0 dark:bg-surface-900 shadow-sm">
    <!-- Activity Bar (Far Left Strip) -->
    <div
      class="w-14 flex flex-col items-center py-3 border-r border-surface-200 dark:border-surface-700 bg-surface-50 dark:bg-surface-950 shrink-0"
    >
      <!-- Orbit Brand Logo -->
      <div class="mb-4 flex items-center justify-center p-1">
        <img src="/logo.png" alt="Orbit Logo" class="w-7 h-7 object-contain" />
      </div>

      <!-- Main Activity Items -->
      <div class="flex flex-col gap-2 w-full px-1 items-center">
        <!-- Resources / Explorer Tab -->
        <Button
          v-tooltip.right="'Resources'"
          rounded
          variant="text"
          :severity="activeTab === 'resources' ? 'primary' : 'secondary'"
          :class="[
            'w-10 h-10 flex! items-center! justify-center!',
            activeTab === 'resources'
              ? 'bg-primary-100! dark:bg-primary-900/40! text-primary-600! dark:text-primary-400!'
              : 'text-muted-color'
          ]"
          @click="toggleTab('resources')"
        >
          <LayoutDashboard class="w-5 h-5" />
        </Button>

        <!-- Clusters Tab -->
        <Button
          v-tooltip.right="'Clusters'"
          rounded
          variant="text"
          :severity="activeTab === 'clusters' ? 'primary' : 'secondary'"
          :class="[
            'w-10 h-10 flex! items-center! justify-center!',
            activeTab === 'clusters'
              ? 'bg-primary-100! dark:bg-primary-900/40! text-primary-600! dark:text-primary-400!'
              : 'text-muted-color'
          ]"
          @click="toggleTab('clusters')"
        >
          <Server class="w-5 h-5" />
        </Button>
      </div>

      <!-- Bottom Actions Spacer -->
      <div class="flex-1"></div>

      <!-- Bottom Activity Actions -->
      <div class="flex flex-col gap-2 items-center w-full px-1">
        <!-- Settings Link -->
        <Button
          v-tooltip.right="'Settings'"
          rounded
          variant="text"
          :class="[
            'w-10 h-10 flex! items-center! justify-center!',
            route.path === '/settings'
              ? 'bg-primary-100! dark:bg-primary-900/40! text-primary-600! dark:text-primary-400!'
              : 'text-muted-color'
          ]"
          @click="router.push('/settings')"
        >
          <Settings class="w-5 h-5" />
        </Button>

        <!-- Theme Toggle -->
        <Button
          v-tooltip.right="isDark ? 'Light Mode' : 'Dark Mode'"
          rounded
          variant="text"
          class="w-10 h-10 text-muted-color"
          :icon="isDark ? 'pi pi-sun' : 'pi pi-moon'"
          @click="toggleTheme"
        />

        <!-- Notifications -->
        <div class="relative inline-flex">
          <Button
            v-tooltip.right="'Notifications'"
            rounded
            variant="text"
            class="w-10 h-10 text-muted-color"
            icon="pi pi-bell"
            badge-severity="danger"
            :badge="
              notificationStore.unreadCount > 0
                ? notificationStore.unreadCount.toString()
                : undefined
            "
            @click="notificationStore.toggleDrawer()"
          />
        </div>

        <!-- User Profile -->
        <Button
          v-tooltip.right="'User Profile'"
          rounded
          variant="text"
          class="w-10 h-10 text-muted-color"
          icon="pi pi-user"
          @click="profileStore.toggleDrawer()"
        />

        <!-- GitHub Docs -->
        <Button
          v-tooltip.right="'GitHub Repository'"
          rounded
          variant="text"
          class="w-10 h-10 text-muted-color"
          icon="pi pi-github"
          @click="os.open('https://github.com/vantoan1511/orbit')"
        />
      </div>
    </div>

    <!-- Sidebar Panel (Contextual View Panel) -->
    <div
      v-if="activeTab"
      class="w-52 flex flex-col h-full overflow-hidden border-r border-surface-200 dark:border-surface-700 bg-surface-0 dark:bg-surface-900"
    >
      <!-- Panel Header -->
      <div
        class="h-14 px-4 flex items-center justify-between border-b border-surface-100 dark:border-surface-800"
      >
        <span class="font-bold text-xs tracking-wider uppercase text-muted-color font-ui">
          {{ activeTab === 'resources' ? activeCluster?.name || 'Resources' : 'Clusters' }}
        </span>
        <span class="text-[10px] text-muted-color font-mono">{{ VERSION }}</span>
      </div>

      <!-- Panel Body: Resources Context -->
      <nav v-if="activeTab === 'resources'" class="flex-1 overflow-y-auto p-3 space-y-1">
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
                ? 'bg-primary-100! dark:bg-primary-900/30! text-primary-600! dark:text-primary-400! font-bold!'
                : 'text-muted-color hover:bg-surface-100 dark:hover:bg-surface-800 font-medium!',
              'flex! items-center! justify-start! gap-2.5 px-3 py-2 rounded-md transition-colors text-sm',
              k8sStore.activeClusterId === null ? 'hidden!' : ''
            ]"
          >
            <component :is="link.icon" class="w-4 h-4 shrink-0" />
            <span class="truncate">{{ link.name }}</span>
          </router-link>
        </Button>
        <p v-if="k8sStore.activeClusterId === null" class="text-xs text-muted-color p-2">
          Select or add a cluster to view resources.
        </p>
      </nav>

      <!-- Panel Body: Clusters Context -->
      <div
        v-else-if="activeTab === 'clusters'"
        class="flex-1 overflow-y-auto p-3 flex flex-col gap-2"
      >
        <div class="flex flex-col gap-1.5 flex-1">
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
            class="truncate justify-start font-semibold text-xs py-2"
            variant="text"
            @click="handleSwitchCluster(cluster.id)"
          >
            <Check
              v-if="k8sStore.activeClusterId === cluster.id"
              :size="14"
              class="shrink-0 mr-1"
            />
            <span class="truncate">{{ cluster.name }}</span>
          </Button>

          <!-- Empty state -->
          <p v-if="k8sStore.clusters.length === 0" class="text-xs text-muted-color px-2 py-1">
            No clusters added yet
          </p>
        </div>

        <!-- Add Cluster Button -->
        <Button fluid severity="contrast" size="small" class="mt-auto" @click="handleAddCluster">
          <Plus :size="14" />
          <span class="text-xs font-semibold">Add cluster</span>
        </Button>
      </div>
    </div>
  </aside>
</template>
