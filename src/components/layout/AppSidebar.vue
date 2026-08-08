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
import { Button, PanelMenu } from 'primevue'
import type { MenuItem } from 'primevue/menuitem'
import { type Component, computed, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'

const k8sStore = useKubernetesStore()
const notificationStore = useNotificationStore()
const profileStore = useProfileStore()
const { activeCluster, isRefreshing, handleAddCluster } = useCluster()
const route = useRoute()
const router = useRouter()
const { isDark, toggleTheme } = useTheme()

type CategoryId =
  'logs' | 'clusters' | 'core' | 'workloads' | 'network' | 'storage' | 'config' | 'security'

const activeTab = ref<CategoryId | null>('workloads')

const categories = [
  { id: 'clusters' as const, name: 'Clusters', icon: Server, defaultPath: null },
  { id: 'core' as const, name: 'Overview & Core', icon: LayoutDashboard, defaultPath: '/' },
  {
    id: 'workloads' as const,
    name: 'Workloads',
    icon: Boxes,
    defaultPath: '/workloads?tab=deployments'
  },
  { id: 'network' as const, name: 'Network', icon: Network, defaultPath: '/network?tab=services' },
  {
    id: 'storage' as const,
    name: 'Storage',
    icon: HardDrive,
    defaultPath: '/storage?tab=overview'
  },
  {
    id: 'config' as const,
    name: 'Config & Secrets',
    icon: Settings2,
    defaultPath: '/config?tab=configmaps'
  },
  { id: 'security' as const, name: 'Security', icon: ShieldCheck, defaultPath: '/policies' },
  { id: 'logs' as const, name: 'Logs', icon: FileText, defaultPath: '/logs' }
]

export interface AppSidebarMenuItem extends Omit<MenuItem, 'icon'> {
  label?: string
  icon?: Component
  customIcon?: Component
  route?: string
  items?: AppSidebarMenuItem[]
  name?: string
  path?: string
}

const panelMenuItems = computed<AppSidebarMenuItem[]>(() => [
  {
    key: 'core',
    label: 'Overview & Core',
    customIcon: LayoutDashboard,
    items: [
      { key: 'core-overview', label: 'Overview', customIcon: LayoutDashboard, route: '/' },
      { key: 'core-nodes', label: 'Nodes', customIcon: Server, route: '/nodes' },
      { key: 'core-namespaces', label: 'Namespaces', customIcon: FolderOpen, route: '/namespaces' },
      { key: 'core-events', label: 'Events', customIcon: Activity, route: '/events' }
    ]
  },
  {
    key: 'workloads',
    label: 'Workloads',
    customIcon: Boxes,
    items: [
      {
        key: 'workloads-overview',
        label: 'Overview',
        customIcon: Boxes,
        route: '/workloads?tab=overview'
      },
      { key: 'workloads-pods', label: 'Pods', customIcon: Box, route: '/pods' },
      {
        key: 'workloads-deployments',
        label: 'Deployments',
        customIcon: Boxes,
        route: '/workloads?tab=deployments'
      },
      {
        key: 'workloads-statefulsets',
        label: 'StatefulSets',
        customIcon: Boxes,
        route: '/workloads?tab=statefulsets'
      },
      {
        key: 'workloads-daemonsets',
        label: 'DaemonSets',
        customIcon: Boxes,
        route: '/workloads?tab=daemonsets'
      },
      {
        key: 'workloads-replicasets',
        label: 'ReplicaSets',
        customIcon: Boxes,
        route: '/workloads?tab=replicasets'
      },
      { key: 'workloads-jobs', label: 'Jobs', customIcon: Boxes, route: '/workloads?tab=jobs' },
      {
        key: 'workloads-cronjobs',
        label: 'CronJobs',
        customIcon: Boxes,
        route: '/workloads?tab=cronjobs'
      }
    ]
  },
  {
    key: 'network',
    label: 'Network',
    customIcon: Network,
    items: [
      {
        key: 'network-services',
        label: 'Services',
        customIcon: Network,
        route: '/network?tab=services'
      },
      {
        key: 'network-ingresses',
        label: 'Ingresses',
        customIcon: Network,
        route: '/network?tab=ingresses'
      }
    ]
  },
  {
    key: 'storage',
    label: 'Storage',
    customIcon: HardDrive,
    items: [
      {
        key: 'storage-overview',
        label: 'Overview',
        customIcon: HardDrive,
        route: '/storage?tab=overview'
      },
      {
        key: 'storage-pvs',
        label: 'PersistentVolumes',
        customIcon: HardDrive,
        route: '/storage?tab=pvs'
      },
      {
        key: 'storage-pvcs',
        label: 'Volume Claims',
        customIcon: HardDrive,
        route: '/storage?tab=pvcs'
      },
      {
        key: 'storage-classes',
        label: 'StorageClasses',
        customIcon: HardDrive,
        route: '/storage?tab=classes'
      }
    ]
  },
  {
    key: 'config',
    label: 'Config & Secrets',
    customIcon: Settings2,
    items: [
      {
        key: 'config-configmaps',
        label: 'ConfigMaps',
        customIcon: Settings2,
        route: '/config?tab=configmaps'
      },
      {
        key: 'config-secrets',
        label: 'Secrets',
        customIcon: Settings2,
        route: '/config?tab=secrets'
      }
    ]
  },
  {
    key: 'security',
    label: 'Security',
    customIcon: ShieldCheck,
    items: [
      { key: 'security-policies', label: 'Policies', customIcon: ShieldCheck, route: '/policies' }
    ]
  },
  {
    key: 'logs',
    label: 'Logs',
    customIcon: FileText,
    items: [{ key: 'logs-main', label: 'Logs', customIcon: FileText, route: '/logs' }]
  }
])

const currentCategoryPanelItems = computed<AppSidebarMenuItem[]>(() => {
  if (!activeTab.value || activeTab.value === 'clusters') return []
  const item = panelMenuItems.value.find((cat) => cat.key === activeTab.value)
  return item ? [item] : []
})

const expandedKeys = ref<Record<string, boolean>>({
  core: true,
  workloads: true,
  network: true,
  storage: true,
  config: true,
  security: true,
  logs: true
})

const getCategoryForRoute = (path: string): CategoryId | null => {
  if (path === '/logs') return 'logs'
  if (path === '/' || path === '/nodes' || path === '/namespaces' || path === '/events')
    return 'core'
  if (path.startsWith('/workloads') || path === '/pods') return 'workloads'
  if (path.startsWith('/network')) return 'network'
  if (path.startsWith('/storage')) return 'storage'
  if (path.startsWith('/config')) return 'config'
  if (path.startsWith('/policies')) return 'security'
  return null
}

watch(
  () => route.path,
  (newPath) => {
    const category = getCategoryForRoute(newPath)
    if (category && activeTab.value !== 'clusters') {
      activeTab.value = category
      expandedKeys.value = { ...expandedKeys.value, [category]: true }
    }
  },
  { immediate: true }
)

const toggleCategory = (cat: (typeof categories)[number]) => {
  if (cat.id === 'clusters') {
    if (activeTab.value === 'clusters') {
      activeTab.value = getCategoryForRoute(route.path) || 'workloads'
    } else {
      activeTab.value = 'clusters'
    }
    return
  }

  activeTab.value = cat.id
  expandedKeys.value = { ...expandedKeys.value, [cat.id]: true }
  if (cat.defaultPath) {
    router.push(cat.defaultPath)
  }
}

const handleSwitchCluster = async (clusterId: string) => {
  await kubernetesService.switchCluster(clusterId)
  activeTab.value = getCategoryForRoute(route.path) || 'core'
}

const isLinkActive = (linkPath: string) => {
  if (linkPath.includes('?')) {
    const [path, queryStr] = linkPath.split('?')
    if (route.path !== path) return false
    if (!queryStr) return false
    const parts = queryStr.split('=')
    const key = parts[0]
    const val = parts[1]
    if (!key || !val) return false
    return route.query[key] === val
  }
  return route.path === linkPath
}

const currentCategoryTitle = computed(() => {
  if (!activeTab.value) return ''
  const cat = categories.find((c) => c.id === activeTab.value)
  return cat ? cat.name : ''
})
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

      <!-- Main Activity Items (Categories) -->
      <div class="flex flex-col gap-1.5 w-full px-1 items-center">
        <Button
          v-for="cat in categories"
          :key="cat.id"
          v-tooltip.right="cat.name"
          rounded
          variant="text"
          :severity="activeTab === cat.id ? 'primary' : 'secondary'"
          :class="[
            'w-10 h-10 flex! items-center! justify-center!',
            activeTab === cat.id
              ? 'bg-primary-100! dark:bg-primary-900/40! text-primary-600! dark:text-primary-400!'
              : 'text-muted-color'
          ]"
          @click="toggleCategory(cat)"
        >
          <component :is="cat.icon" class="w-5 h-5" />
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
        <span class="font-bold text-xs tracking-wider uppercase text-muted-color font-ui truncate">
          {{ activeTab === 'clusters' ? 'Clusters' : currentCategoryTitle }}
        </span>
        <span class="text-[10px] text-muted-color font-mono shrink-0 ml-1">{{ VERSION }}</span>
      </div>

      <!-- Panel Body: Category Context Links -->
      <nav v-if="activeTab !== 'clusters'" class="flex-1 overflow-y-auto p-2">
        <PanelMenu
          v-if="k8sStore.activeClusterId !== null"
          :model="currentCategoryPanelItems as unknown as MenuItem[]"
          v-model:expandedKeys="expandedKeys"
          multiple
          class="w-full border-none bg-transparent"
          :pt="{
            panel: { class: 'border-none bg-transparent mb-1' },
            header: { class: 'border-none bg-transparent p-0' },
            headercontent: { class: 'border-none bg-transparent p-0' },
            menucontent: { class: 'border-none bg-transparent p-0 pl-2' },
            root: { class: 'border-none bg-transparent' }
          }"
        >
          <template #item="{ item, active }">
            <!-- Category Header Group (item with sub-items) -->
            <div
              v-if="item.items && item.items.length > 0"
              class="flex items-center justify-between px-3 py-1.5 text-xs font-bold uppercase tracking-wider text-muted-color hover:text-color hover:bg-surface-100 dark:hover:bg-surface-800/60 rounded-md cursor-pointer transition-colors select-none my-0.5"
            >
              <div class="flex items-center gap-2 min-w-0">
                <component
                  :is="(item as any).customIcon || item.icon"
                  class="w-4 h-4 shrink-0 text-muted-color"
                />
                <span class="truncate">{{ item.label }}</span>
              </div>
              <span
                :class="[
                  'pi text-[10px] text-muted-color transition-transform duration-200 shrink-0 ml-1',
                  active ? 'pi-chevron-down' : 'pi-chevron-right'
                ]"
              ></span>
            </div>

            <!-- Leaf Menu Link (child item with route) -->
            <router-link v-else-if="item.route" v-slot="{ href, navigate }" :to="item.route" custom>
              <a
                :href="href"
                :class="[
                  isLinkActive(item.route as string)
                    ? 'bg-primary-100! dark:bg-primary-900/30! text-primary-600! dark:text-primary-400! font-bold!'
                    : 'text-muted-color hover:bg-surface-100 dark:hover:bg-surface-800 font-medium!',
                  'flex items-center gap-2.5 px-3 py-1.5 rounded-md transition-colors text-sm cursor-pointer no-underline select-none my-0.5'
                ]"
                @click="navigate"
              >
                <component :is="(item as any).customIcon || item.icon" class="w-4 h-4 shrink-0" />
                <span class="truncate">{{ item.label }}</span>
              </a>
            </router-link>
          </template>
        </PanelMenu>
        <p v-else class="text-xs text-muted-color p-2">
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
