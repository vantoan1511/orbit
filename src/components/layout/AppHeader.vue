<script setup lang="ts">
import Breadcrumb from 'primevue/breadcrumb'
import Button from 'primevue/button'
import { computed } from 'vue'
import { useRoute } from 'vue-router'
import { useSearchStore } from '@/stores/searchStore'
import { Search } from '@lucide/vue'

const route = useRoute()
const searchStore = useSearchStore()

const isMac =
  typeof navigator !== 'undefined' &&
  navigator.platform &&
  navigator.platform.toUpperCase().indexOf('MAC') >= 0

const home = computed(() => ({
  icon: 'pi pi-home',
  route: '/'
}))

const ROUTE_LABEL_MAP: Record<string, string> = {
  nodes: 'Nodes',
  workloads: 'Workloads',
  pods: 'Pods',
  network: 'Network',
  'configmaps-secrets': 'ConfigMaps & Secrets',
  storage: 'Storage',
  namespaces: 'Namespaces',
  events: 'Events',
  settings: 'Settings',
  policies: 'Policies',
  logs: 'Logs'
}

const TAB_LABEL_MAP: Record<string, string> = {
  configmaps: 'ConfigMaps',
  secrets: 'Secrets',
  deployments: 'Deployments',
  statefulsets: 'StatefulSets',
  daemonsets: 'DaemonSets',
  replicasets: 'ReplicaSets',
  jobs: 'Jobs',
  cronjobs: 'CronJobs',
  overview: 'Overview',
  services: 'Services',
  endpoints: 'Endpoints',
  ingresses: 'Ingresses',
  pvcs: 'Persistent Volume Claims',
  pvs: 'Persistent Volumes',
  storageclasses: 'Storage Classes',
  networkpolicies: 'Network Policies'
}

const items = computed(() => {
  const currentRouteName = route.name as string
  if (!currentRouteName || currentRouteName === 'dashboard' || currentRouteName === 'welcome') {
    return []
  }

  const breadcrumbs: Array<{ label: string; route?: string }> = []

  if (currentRouteName === 'edit-workload') {
    breadcrumbs.push({ label: 'Workloads', route: '/workloads' })

    const kind = (route.params.kind as string) || ''
    if (kind) {
      breadcrumbs.push({
        label: kind,
        route: `/workloads?tab=${kind.toLowerCase()}s`
      })
    }

    const namespace = (route.params.namespace as string) || ''
    if (namespace) {
      breadcrumbs.push({ label: namespace })
    }

    const workloadName = (route.params.name as string) || ''
    if (workloadName) {
      breadcrumbs.push({ label: workloadName })
    }

    breadcrumbs.push({
      label: 'Edit',
      route: route.path
    })

    return breadcrumbs
  }

  const label =
    ROUTE_LABEL_MAP[currentRouteName] ||
    currentRouteName
      .split('-')
      .map((word) => word.charAt(0).toUpperCase() + word.slice(1))
      .join(' ')

  const tab = route.query.tab as string
  if (tab) {
    breadcrumbs.push({ label, route: route.path })

    const tabLabel =
      TAB_LABEL_MAP[tab] ||
      tab
        .split('-')
        .map((word) => word.charAt(0).toUpperCase() + word.slice(1))
        .join(' ')

    breadcrumbs.push({ label: tabLabel })
  } else {
    breadcrumbs.push({ label, route: route.path })
  }

  return breadcrumbs
})
</script>

<template>
  <header
    class="flex items-center select-none bg-(--bg-sidebar) border-b border-(--border) px-3 py-2 text-xs shrink-0 z-20"
  >
    <Breadcrumb
      :home="home"
      :model="items"
      class="p-0! bg-transparent! border-none! text-xs min-w-0 flex-1 overflow-hidden"
    >
      <template #item="{ item, props }">
        <router-link v-if="item.route" v-slot="{ href, navigate }" :to="item.route" custom>
          <a
            :href="href"
            v-bind="props.action"
            class="flex items-center gap-1.5 hover:text-primary transition-colors"
            @click="navigate"
          >
            <span v-if="item.icon" :class="item.icon" />
            <span v-if="item.label" class="font-medium">{{ item.label }}</span>
          </a>
        </router-link>
        <span v-else class="flex items-center gap-1.5 text-muted-color">
          <span v-if="item.icon" :class="item.icon" />
          <span v-if="item.label">{{ item.label }}</span>
        </span>
      </template>
    </Breadcrumb>
    <Button
      type="button"
      severity="secondary"
      variant="outlined"
      size="small"
      class="group flex items-center justify-between gap-2 h-7 w-auto sm:w-56 md:w-64 lg:w-72 px-2 sm:px-2.5 text-xs text-muted-color bg-(--bg-hover)/40! dark:bg-(--bg-hover)/30! border-(--border)! hover:bg-(--bg-hover)/80! hover:border-(--border-strong)! hover:text-primary! active:bg-(--bg-active)! rounded-md font-normal cursor-pointer ml-auto shrink-0 transition-all duration-150"
      v-tooltip.bottom="isMac ? 'Search everywhere (⌘⇧P)' : 'Search everywhere (Ctrl+Shift+P)'"
      @click="searchStore.open()"
    >
      <div class="flex items-center gap-2 min-w-0">
        <Search
          class="w-3.5 h-3.5 shrink-0 text-muted-color group-hover:text-primary transition-colors"
        />
        <span
          class="hidden sm:inline truncate text-muted-color group-hover:text-primary transition-colors font-normal"
        >
          Search everywhere...
        </span>
      </div>
      <kbd
        class="hidden md:inline-flex items-center px-1.5 py-0.5 text-[10px] font-mono font-medium leading-none text-muted-color bg-(--bg-card) dark:bg-(--bg-card) border border-(--border) rounded group-hover:border-(--border-strong) group-hover:text-primary transition-colors shrink-0"
      >
        {{ isMac ? '⌘⇧P' : 'Ctrl+Shift+P' }}
      </kbd>
    </Button>
  </header>
</template>
