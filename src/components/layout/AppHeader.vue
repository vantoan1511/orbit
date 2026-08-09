<script setup lang="ts">
import Breadcrumb from 'primevue/breadcrumb'
import { computed } from 'vue'
import { useRoute } from 'vue-router'

const route = useRoute()

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
    class="flex items-center h-12 px-6 border-b border-[var(--p-surface-200)] dark:border-[var(--p-surface-800)] bg-[var(--p-surface-0)] dark:bg-[var(--p-surface-900)] shrink-0"
  >
    <Breadcrumb :home="home" :model="items" class="!p-0 !bg-transparent !border-none text-xs">
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
  </header>
</template>
