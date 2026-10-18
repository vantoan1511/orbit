<script setup lang="ts">
import { useCluster } from '@/composables/useCluster'
import { useKubernetesStore } from '@/stores/kubernetesStore'
import { useSearchStore } from '@/stores/searchStore'
import { useShortcutStore } from '@/stores/shortcutStore'
import { useWindowControls } from '@/composables/useWindowControls'
import { detectCloudProvider } from '@/utils/cloudProvider'
import { Clock, Cloud, Search } from '@lucide/vue'
import Button from 'primevue/button'
import { computed } from 'vue'
import { KubernetesIcon } from 'vue3-simple-icons'
import WindowControls from './WindowControls.vue'

const { toggleMaximize } = useWindowControls()
const searchStore = useSearchStore()
const shortcutStore = useShortcutStore()
const kubernetesStore = useKubernetesStore()
const { activeCluster } = useCluster()
const isDev = import.meta.env.DEV

const searchTooltip = computed(() => {
  const shortcut = shortcutStore.getDisplayShortcut('quickOpen')
  return shortcut ? `Search everywhere (${shortcut})` : 'Search everywhere'
})

const kubernetesVersion = computed(() => {
  return kubernetesStore.nodes[0]?.version || 'Unknown'
})

const clusterUptime = computed(() => {
  return kubernetesStore.nodes[0]?.uptime || 'Unknown'
})

const cloudProvider = computed(() => {
  return detectCloudProvider(kubernetesStore.nodes[0]?.labels)
})

const clusterTooltip = computed(() => {
  if (!activeCluster.value) return 'No active cluster'
  const parts = [
    `${activeCluster.value.name} (${activeCluster.value.status || 'unknown'})`,
    `K8S: ${kubernetesVersion.value}`,
    `Cloud: ${cloudProvider.value.provider} / ${cloudProvider.value.platform}`,
    `Uptime: ${clusterUptime.value}`
  ]
  return parts.join(' • ')
})
</script>

<template>
  <header
    class="relative flex items-center justify-between h-8 w-full shrink-0 select-none bg-(--bg-sidebar) border-b border-(--border) text-xs z-30"
    data-tauri-drag-region
    @dblclick="toggleMaximize"
  >
    <!-- Left: Cluster Info & Inline Metadata (replaces static Orbit logo) -->
    <div class="flex items-center gap-2.5 px-3 shrink-0 pointer-events-auto" @dblclick.stop>
      <template v-if="activeCluster !== null">
        <div class="flex items-center gap-1.5 cursor-default" v-tooltip.bottom="clusterTooltip">
          <!-- Status dot -->
          <span
            class="w-2 h-2 rounded-full shrink-0"
            :class="
              activeCluster.status === 'healthy'
                ? 'bg-(--success)'
                : activeCluster.status === 'warning'
                  ? 'bg-(--warning)'
                  : 'bg-(--danger)'
            "
          />
          <!-- Cluster name -->
          <span
            class="font-semibold text-xs text-primary tracking-tight truncate max-w-[160px] sm:max-w-[200px] md:max-w-[240px]"
          >
            {{ activeCluster.name }}
          </span>
        </div>

        <!-- Inline metadata on wide screens -->
        <div
          v-if="activeCluster.status === 'healthy'"
          class="hidden xl:flex items-center gap-2.5 text-[11px] font-medium tracking-wider text-muted-color uppercase shrink-0"
        >
          <div class="h-3 w-px bg-(--border)" />
          <div class="flex items-center gap-1.5">
            <KubernetesIcon :size="12" />
            <span>K8S {{ kubernetesVersion }}</span>
          </div>
          <div class="flex items-center gap-1.5">
            <Cloud :size="12" />
            <span>{{ cloudProvider.provider }}</span>
            <span>/</span>
            <span>{{ cloudProvider.platform }}</span>
          </div>
          <div class="flex items-center gap-1.5">
            <Clock :size="12" />
            <span>{{ clusterUptime }}</span>
          </div>
        </div>
      </template>

      <span v-else class="font-semibold text-muted-color text-xs tracking-tight">
        No active cluster
      </span>

      <!-- Development mode indicator -->
      <span
        v-if="isDev"
        class="px-1.5 py-0.5 rounded text-[10px] font-mono font-bold tracking-wider uppercase bg-amber-500/15 text-amber-600 dark:text-amber-400 border border-amber-500/30 shrink-0 select-none cursor-help"
        v-tooltip.bottom="'Orbit Development Instance (config: ~/.orbit-dev)'"
      >DEV</span>
    </div>

    <!-- Center: Search Everywhere Trigger (Centered to Screen/Window) -->
    <div
      class="absolute left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 z-40 pointer-events-auto"
    >
      <Button
        type="button"
        severity="secondary"
        variant="outlined"
        size="small"
        class="group flex items-center justify-start gap-2 h-6 w-56 sm:w-64 md:w-80 px-2 text-xs text-muted-color bg-(--bg-hover)/40! dark:bg-(--bg-hover)/30! border-(--border)! hover:bg-(--bg-hover)/80! hover:border-(--border-strong)! hover:text-primary! active:bg-(--bg-active)! rounded-md font-normal cursor-pointer transition-all duration-150"
        v-tooltip.bottom="searchTooltip"
        @click.stop="searchStore.open()"
        @dblclick.stop
      >
        <div class="flex items-center gap-1.5 min-w-0">
          <Search
            class="w-3 h-3 shrink-0 text-muted-color group-hover:text-primary transition-colors"
          />
          <span
            class="hidden sm:inline truncate text-muted-color group-hover:text-primary transition-colors font-normal text-[11px]"
          >
            Search everywhere...
          </span>
        </div>
      </Button>
    </div>

    <!-- Center: Window Drag Strip -->
    <div class="flex-1 h-full" data-tauri-drag-region />

    <!-- Right: Window Controls -->
    <WindowControls />
  </header>
</template>
