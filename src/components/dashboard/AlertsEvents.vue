<script setup lang="ts">
import { useKubernetesStore } from '@/stores/kubernetesStore'
import {
  AlertTriangle,
  CheckCircle2,
  FileText,
  HardDrive,
  HelpCircle,
  Key,
  RefreshCw,
  Server
} from '@lucide/vue'
import { computed } from 'vue'

const kubernetesStore = useKubernetesStore()

function getEventIcon(kind?: string) {
  switch (kind?.toLowerCase()) {
    case 'pod':
      return RefreshCw
    case 'node':
      return Server
    case 'secret':
      return Key
    case 'configmap':
      return FileText
    case 'persistentvolume':
    case 'persistentvolumeclaim':
      return HardDrive
    case 'job':
    case 'cronjob':
      return CheckCircle2
    default:
      return HelpCircle
  }
}

function getEventIconColor(kind?: string) {
  switch (kind?.toLowerCase()) {
    case 'pod':
      return 'text-blue-400 bg-blue-500/10'
    case 'node':
      return 'text-violet-400 bg-violet-500/10'
    case 'secret':
      return 'text-amber-400 bg-amber-500/10'
    case 'configmap':
      return 'text-indigo-400 bg-indigo-500/10'
    case 'persistentvolume':
    case 'persistentvolumeclaim':
      return 'text-sky-400 bg-sky-500/10'
    case 'job':
    case 'cronjob':
      return 'text-emerald-400 bg-emerald-500/10'
    default:
      return 'text-gray-400 bg-gray-500/10'
  }
}

const warnings = computed(() => {
  return kubernetesStore.events
    .filter((e) => e.type === 'Warning')
    .slice(0, 5)
    .map((e) => ({
      uid: e.uid,
      reason: e.reason || 'Warning',
      message: e.message || '',
      time: e.lastSeen || e.firstSeen || e.time || 'unknown'
    }))
})

const normalEvents = computed(() => {
  return kubernetesStore.events
    .filter((e) => e.type === 'Normal')
    .slice(0, 7)
    .map((e) => ({
      uid: e.uid,
      icon: getEventIcon(e.objectKind),
      iconColor: getEventIconColor(e.objectKind),
      message: e.message || '',
      time: e.lastSeen || e.firstSeen || e.time || 'unknown'
    }))
})
</script>

<template>
  <div class="grid grid-cols-1 xl:grid-cols-2 gap-6">
    <!-- Recent Warnings -->
    <div
      class="bg-(--bg-card) border border-(--border) rounded-xl p-5 shadow-sm transition-all duration-200 flex flex-col justify-between"
    >
      <div>
        <div class="flex items-center justify-between mb-4">
          <div class="text-xs font-semibold text-(--text-primary) uppercase tracking-wider">
            Recent Warnings
          </div>
          <button
            class="text-xs text-(--accent) hover:underline flex items-center gap-1 font-medium"
          >
            <span>View all</span>
          </button>
        </div>

        <div class="divide-y divide-(--border)">
          <div
            v-for="(warning, index) in warnings"
            :key="warning.uid || index"
            class="py-3 flex items-start gap-3.5 first:pt-0 last:pb-0"
          >
            <div class="p-2 rounded-lg bg-rose-500/10 text-rose-500 mt-0.5 shrink-0">
              <AlertTriangle class="w-4 h-4" />
            </div>
            <div class="flex-1 min-w-0">
              <div class="flex items-center justify-between gap-2">
                <span class="text-xs font-bold text-(--text-primary) font-mono">{{
                  warning.reason
                }}</span>
                <span class="text-[10px] text-(--text-muted) font-mono whitespace-nowrap">{{
                  warning.time
                }}</span>
              </div>
              <p class="text-xs text-(--text-secondary) mt-1 truncate">
                {{ warning.message }}
              </p>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Recent Events -->
    <div
      class="bg-(--bg-card) border border-(--border) rounded-xl p-5 shadow-sm transition-all duration-200 flex flex-col justify-between"
    >
      <div>
        <div class="flex items-center justify-between mb-4">
          <div class="text-xs font-semibold text-(--text-primary) uppercase tracking-wider">
            Recent Events
          </div>
          <button
            class="text-xs text-(--accent) hover:underline flex items-center gap-1 font-medium"
          >
            <span>View all</span>
          </button>
        </div>

        <div class="divide-y divide-(--border)">
          <div
            v-for="(event, index) in normalEvents"
            :key="event.uid || index"
            class="py-2.5 flex items-center gap-3.5 first:pt-0 last:pb-0"
          >
            <div class="p-1.5 rounded-lg shrink-0" :class="event.iconColor">
              <component :is="event.icon" class="w-4 h-4" />
            </div>
            <div class="flex-1 min-w-0 flex items-center justify-between gap-4">
              <p class="text-xs text-(--text-secondary) truncate font-medium">
                {{ event.message }}
              </p>
              <span class="text-[10px] text-(--text-muted) font-mono whitespace-nowrap">{{
                event.time
              }}</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
