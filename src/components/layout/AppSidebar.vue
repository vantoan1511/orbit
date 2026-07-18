<script setup lang="ts">
import { kubernetesService } from '@/services/kubernetesService'
import { os } from '@/services/nativeService'
import { useKubernetesStore } from '@/stores/kubernetesStore'
import { VERSION } from '@/version'
import {
  Activity,
  Bell,
  BookOpen,
  Box,
  Boxes,
  FolderOpen,
  HardDrive,
  LayoutDashboard,
  Moon,
  Network,
  Plus,
  Server,
  Settings,
  Settings2,
  ShieldCheck,
  Sun,
  User
} from '@lucide/vue'
import { ref } from 'vue'
import { useRoute } from 'vue-router'

const k8sStore = useKubernetesStore()

const handleSwitchCluster = async (clusterId: string) => {
  await kubernetesService.switchCluster(clusterId)
}

const handleAddCluster = async () => {
  try {
    const selectedFiles = await os.showOpenDialog('Select Kubeconfig File', {
      filters: [{ name: 'Kubeconfig', extensions: ['*', 'yaml', 'yml', 'conf'] }]
    })
    if (selectedFiles && selectedFiles.length > 0 && selectedFiles[0]) {
      await kubernetesService.addCluster(selectedFiles[0])
    }
  } catch (error) {
    console.error('Failed to add cluster:', error)
  }
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

// Theme state
const isDark = ref(true)

const toggleTheme = () => {
  isDark.value = !isDark.value
  const html = document.documentElement
  if (isDark.value) {
    html.classList.add('my-app-dark')
    html.setAttribute('data-theme', 'dark')
  } else {
    html.classList.remove('my-app-dark')
    html.setAttribute('data-theme', 'light')
  }
}
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
      <div class="flex flex-col gap-1">
        <button
          v-for="cluster in k8sStore.clusters"
          :key="cluster.id"
          @click="handleSwitchCluster(cluster.id)"
          class="w-full text-left px-3 py-2 rounded-lg text-sm flex items-center justify-between transition-all duration-200"
          :class="[
            k8sStore.activeClusterId === cluster.id
              ? 'bg-(--bg-active) font-medium text-(--text-primary)'
              : 'text-(--text-secondary) hover:bg-(--bg-hover)'
          ]"
        >
          <div class="flex items-center gap-2 truncate">
            <span
              class="w-2 h-2 rounded-full shrink-0"
              :class="cluster.status === 'healthy' ? 'bg-emerald-500' : 'bg-zinc-500'"
            ></span>
            <span class="truncate">{{ cluster.name }}</span>
          </div>
        </button>

        <!-- Empty state when no clusters are configured -->
        <p v-if="k8sStore.clusters.length === 0" class="text-xs text-(--text-muted) px-3 py-1">
          No clusters added yet
        </p>

        <button
          @click="handleAddCluster"
          class="w-full text-left px-3 py-2 rounded-lg text-sm text-blue-500 hover:bg-(--bg-hover) flex items-center gap-2 transition-all duration-200 font-medium"
        >
          <Plus class="w-4 h-4" />
          <span>Add Cluster</span>
        </button>
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
        <button
          @click="toggleTheme"
          class="p-2 rounded-lg hover:bg-(--bg-hover) text-(--text-secondary) transition-all duration-200"
          title="Toggle Theme"
        >
          <Sun v-if="isDark" class="w-4.5 h-4.5" />
          <Moon v-else class="w-4.5 h-4.5" />
        </button>

        <!-- Docs -->
        <a
          href="#"
          class="p-2 rounded-lg hover:bg-(--bg-hover) text-(--text-secondary) transition-all duration-200"
          title="Documentation"
        >
          <BookOpen class="w-4.5 h-4.5" />
        </a>

        <!-- Notifications -->
        <button
          class="p-2 rounded-lg hover:bg-(--bg-hover) text-(--text-secondary) transition-all duration-200 relative"
          title="Notifications"
        >
          <Bell class="w-4.5 h-4.5" />
          <span class="absolute top-1 right-1 w-2 h-2 rounded-full bg-rose-500"></span>
        </button>

        <!-- Profile -->
        <button
          class="w-8 h-8 rounded-full bg-(--bg-hover) flex items-center justify-center text-(--text-secondary) hover:bg-(--bg-active) transition-all duration-200"
        >
          <User class="w-4 h-4" />
        </button>
      </div>
    </div>
    <div class="flex items-center justify-center gap-2">
      <span class="text-[10px] text-(--text-muted) font-mono">{{ VERSION }}</span>
    </div>
  </aside>
</template>
