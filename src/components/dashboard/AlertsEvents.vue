<script setup lang="ts">
import EventListCard, { type EventListItem } from '@/components/dashboard/EventListCard.vue'
import { KUBERNETES_EVENT_TYPE } from '@/constants/kubernetes'
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
  cronjob: CheckCircle2,
  deployment: RefreshCw,
  statefulset: RefreshCw,
  daemonset: RefreshCw,
  service: RefreshCw,
  ingress: RefreshCw
} as const

const EVENT_ICON_CLASSES: Record<string, string> = {
  pod: 'text-pod bg-(--accent-soft)',
  node: 'text-node bg-(--success-soft)',
  secret: 'text-secret bg-(--warning-soft)',
  configmap: 'text-configmap bg-(--accent-soft)',
  persistentvolume: 'text-primary bg-(--bg-hover)',
  persistentvolumeclaim: 'text-primary bg-(--bg-hover)',
  job: 'text-job bg-(--success-soft)',
  cronjob: 'text-job bg-(--success-soft)',
  deployment: 'text-deployment bg-(--accent-soft)',
  statefulset: 'text-statefulset bg-(--accent-soft)',
  daemonset: 'text-daemonset bg-(--accent-soft)',
  service: 'text-service bg-(--success-soft)',
  ingress: 'text-ingress bg-(--success-soft)'
}

function getEventIcon(kind?: string) {
  return EVENT_ICONS[kind?.toLowerCase() as keyof typeof EVENT_ICONS] || HelpCircle
}

function getEventIconClass(kind?: string) {
  return EVENT_ICON_CLASSES[kind?.toLowerCase() || ''] || 'text-muted-color bg-(--bg-hover)'
}

const warnings = computed<EventListItem[]>(() => {
  return kubernetesStore.events
    .filter((e) => e.type === KUBERNETES_EVENT_TYPE.Warning)
    .slice(0, 5)
    .map((e) => ({
      uid: e.uid,
      icon: AlertTriangle,
      iconClass: 'text-(--danger) bg-(--danger-soft)',
      label: e.reason || KUBERNETES_EVENT_TYPE.Warning,
      message: e.message || '',
      time: e.lastSeen || e.firstSeen || e.time || 'unknown'
    }))
})

const normalEvents = computed<EventListItem[]>(() => {
  return kubernetesStore.events
    .filter((e) => e.type === KUBERNETES_EVENT_TYPE.Normal)
    .slice(0, 5)
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
