<script setup lang="ts">
import Breadcrumb from 'primevue/breadcrumb'
import { computed } from 'vue'
import { useRoute } from 'vue-router'
import { getResourceBreadcrumbs, TAB_LABEL_MAP } from '@/utils/breadcrumb'

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

const items = computed(() => {
  const currentRouteName = route.name as string
  if (!currentRouteName || currentRouteName === 'dashboard' || currentRouteName === 'welcome') {
    return []
  }

  const breadcrumbs: Array<{ label: string; route?: string }> = []

  if (currentRouteName === 'edit-workload') {
    return getResourceBreadcrumbs({
      kind: route.params.kind as string,
      namespace: route.params.namespace as string,
      name: route.params.name as string
    })
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
  <nav aria-label="Breadcrumb" class="flex items-center min-w-0" data-testid="footer-breadcrumbs">
    <Breadcrumb
      :home="home"
      :model="items"
      class="p-0 bg-transparent border-none text-[11px] font-medium min-w-0 flex-1 overflow-hidden"
    >
      <template #item="{ item, props }">
        <router-link v-if="item.route" v-slot="{ href, navigate }" :to="item.route" custom>
          <a
            :href="href"
            v-bind="props.action"
            class="flex items-center gap-1.5 hover:text-primary text-muted-color transition-colors text-[11px]"
            @click="navigate"
          >
            <span v-if="item.icon" :class="[item.icon, 'text-[11px]']" />
            <span v-if="item.label" class="font-medium truncate max-w-[140px] sm:max-w-[200px]">{{
              item.label
            }}</span>
          </a>
        </router-link>
        <span v-else class="flex items-center gap-1.5 text-muted-color text-[11px]">
          <span v-if="item.icon" :class="[item.icon, 'text-[11px]']" />
          <span v-if="item.label" class="truncate max-w-[140px] sm:max-w-[200px]">{{
            item.label
          }}</span>
        </span>
      </template>
    </Breadcrumb>
  </nav>
</template>
