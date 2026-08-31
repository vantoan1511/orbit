<script setup lang="ts">
import { useKubernetesStore } from '@/stores/kubernetesStore'
import {
  KUBERNETES_NAMESPACE_STATUS,
  KUBERNETES_POD_STATUS,
  KUBERNETES_VOLUME_STATUS
} from '@/constants/kubernetes'
import { Box, ClipboardList, Clock, Database, HardDrive, Layers } from '@lucide/vue'
import { computed } from 'vue'

const store = useKubernetesStore()

const items = computed(() => {
  return [
    {
      title: 'Deployments',
      count: store.deployments.length,
      icon: Layers,
      iconColor: 'text-deployment bg-(--deployment)/10',
      statusLabel: KUBERNETES_VOLUME_STATUS.Available,
      statusVal: store.deployments.reduce((acc, d) => acc + d.available, 0),
      statusSeverity: 'success'
    },
    {
      title: 'StatefulSets',
      count: store.statefulSets.length,
      icon: Database,
      iconColor: 'text-statefulset bg-(--statefulset)/10',
      statusLabel: 'Current',
      statusVal: store.statefulSets.reduce((acc, s) => acc + s.replicas.current, 0),
      statusSeverity: 'success'
    },
    {
      title: 'DaemonSets',
      count: store.daemonSets.length,
      icon: Box,
      iconColor: 'text-daemonset bg-(--daemonset)/10',
      statusLabel: 'Ready',
      statusVal: store.daemonSets.reduce((acc, d) => acc + d.replicas.ready, 0),
      statusSeverity: 'success'
    },
    {
      title: 'Jobs',
      count: store.jobs.length,
      icon: ClipboardList,
      iconColor: 'text-job bg-(--job)/10',
      statusLabel: 'Completed',
      statusVal: store.jobs.filter(
        (j) => j.status === 'Complete' || j.status === KUBERNETES_POD_STATUS.Completed
      ).length,
      statusSeverity: 'success'
    },
    {
      title: 'Persistent Volumes',
      count: store.persistentVolumes.length,
      icon: HardDrive,
      iconColor: 'text-muted-color bg-(--bg-hover)',
      statusLabel: KUBERNETES_VOLUME_STATUS.Bound,
      statusVal: store.persistentVolumes.filter(
        (pv) => pv.status === KUBERNETES_VOLUME_STATUS.Bound
      ).length,
      statusSeverity: 'success'
    },
    {
      title: 'CronJobs',
      count: store.cronJobs.length,
      icon: Clock,
      iconColor: 'text-rose-500 bg-rose-500/10',
      statusLabel: KUBERNETES_NAMESPACE_STATUS.Active,
      statusVal: store.cronJobs.reduce((acc, c) => acc + c.active, 0),
      statusSeverity: 'warn'
    }
  ]
})
</script>

<template>
  <div class="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-6 gap-6">
    <Card v-for="item in items" :key="item.title">
      <template #title>
        <div class="flex items-center justify-between mb-4">
          <div class="p-2 rounded-lg" :class="item.iconColor">
            <component :is="item.icon" class="w-5 h-5" />
          </div>
        </div>
      </template>
      <template #subtitle>
        <div class="text-sm font-bold text-muted-color tracking-wider uppercase">
          {{ item.title }}
        </div>
      </template>
      <template #content>
        <div class="text-3xl font-bold text-primary mt-1 font-ui">
          {{ item.count }}
        </div>
      </template>
      <template #footer>
        <div class="mt-4 flex items-center justify-between text-xs border-t border-surface pt-3">
          <span class="text-muted-color font-medium">{{ item.statusLabel }}</span>
          <Tag :severity="item.statusSeverity" :value="item.statusVal" />
        </div>
      </template>
    </Card>
  </div>
</template>
