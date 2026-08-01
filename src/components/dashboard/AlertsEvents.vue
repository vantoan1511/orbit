<script setup lang="ts">
import EventListCard, { type EventListItem } from '@/components/dashboard/EventListCard.vue'
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

const EVENT_ICONS = {
  pod: RefreshCw,
  node: Server,
  secret: Key,
  configmap: FileText,
  persistentvolume: HardDrive,
  persistentvolumeclaim: HardDrive,
  job: CheckCircle2,
  cronjob: CheckCircle2
} as const

const EVENT_ICON_CLASSES: Record<string, string> = {
  pod: 'text-blue-400 bg-blue-500/10',
  node: 'text-violet-400 bg-violet-500/10',
  secret: 'text-amber-400 bg-amber-500/10',
  configmap: 'text-indigo-400 bg-indigo-500/10',
  persistentvolume: 'text-sky-400 bg-sky-500/10',
  persistentvolumeclaim: 'text-sky-400 bg-sky-500/10',
  job: 'text-emerald-400 bg-emerald-500/10',
  cronjob: 'text-emerald-400 bg-emerald-500/10'
}

function getEventIcon(kind?: string) {
  return EVENT_ICONS[kind?.toLowerCase() as keyof typeof EVENT_ICONS] || HelpCircle
}

function getEventIconClass(kind?: string) {
  return EVENT_ICON_CLASSES[kind?.toLowerCase() || ''] || 'text-gray-400 bg-gray-500/10'
}

const warnings = computed<EventListItem[]>(() => {
  return kubernetesStore.events
    .filter((e) => e.type === 'Warning')
    .slice(0, 5)
    .map((e) => ({
      uid: e.uid,
      icon: AlertTriangle,
      iconClass: 'text-rose-500 bg-rose-500/10',
      label: e.reason || 'Warning',
      message: e.message || '',
      time: e.lastSeen || e.firstSeen || e.time || 'unknown'
    }))
})

const normalEvents = computed<EventListItem[]>(() => {
  return kubernetesStore.events
    .filter((e) => e.type === 'Normal')
    .slice(0, 7)
    .map((e) => ({
      uid: e.uid,
      icon: getEventIcon(e.objectKind),
      iconClass: getEventIconClass(e.objectKind),
      label: e.reason || e.objectKind || 'Event',
      message: e.message || '',
      time: e.lastSeen || e.firstSeen || e.time || 'unknown'
    }))
})
</script>

<template>
  <div class="grid grid-cols-1 xl:grid-cols-2 gap-6">
    <EventListCard title="Recent Warnings" to="/events" :items="warnings" />
    <EventListCard title="Recent Events" to="/events" :items="normalEvents" />
  </div>
</template>
