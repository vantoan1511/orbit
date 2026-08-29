<script setup lang="ts">
import WorkloadEventsTab from '@/components/workloads/WorkloadEventsTab.vue'
import WorkloadOverviewTab from '@/components/workloads/WorkloadOverviewTab.vue'
import WorkloadPodsTab from '@/components/workloads/WorkloadPodsTab.vue'
import ResourceYamlTab from '@/components/shared/ResourceYamlTab.vue'
import { kubernetesService } from '@/services/kubernetesService'
import { events } from '@/services/nativeService'
import { useKubernetesStore } from '@/stores/kubernetesStore'
import { OrbitEvents } from '@/types/events'
import type { CronJobInfo, DaemonSetReplicas, JobInfo, WorkloadInfo } from '@/types/kubernetes'
import { Activity, FileCode, Layers, Terminal } from '@lucide/vue'
import { storeToRefs } from 'pinia'
import BaseResourceDrawer from '@/components/shared/BaseResourceDrawer.vue'
import Button from 'primevue/button'
import Tab from 'primevue/tab'
import TabPanel from 'primevue/tabpanel'
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import * as yaml from 'yaml'

const props = defineProps<{
  visible: boolean
  workload: WorkloadInfo | null
}>()

const emit = defineEmits<{
  (e: 'update:visible', value: boolean): void
}>()

const router = useRouter()
const k8sStore = useKubernetesStore()
const { pods, events: clusterEvents } = storeToRefs(k8sStore)

const activeTab = ref('overview')

const getWorkloadKind = (w: WorkloadInfo): string => {
  if ('schedule' in w) return 'CronJob'
  if ('completions' in w) return 'Job'
  if ('strategy' in w) return 'Deployment'
  if ('replicas' in w && w.replicas) {
    const reps = w.replicas as unknown as Record<string, unknown>
    if ('ready' in reps && 'upToDate' in reps) return 'DaemonSet'
    if (w.name.includes('stateful')) return 'StatefulSet'
    return 'ReplicaSet'
  }
  return 'Workload'
}

const workloadKind = computed(() => {
  return props.workload ? getWorkloadKind(props.workload) : 'Workload'
})

const getWorkloadSeverity = (kind: string) => {
  switch (kind) {
    case 'Deployment':
    case 'StatefulSet':
    case 'DaemonSet':
    case 'ReplicaSet':
      return 'info'
    case 'Job':
    case 'CronJob':
      return 'warn'
    default:
      return 'secondary'
  }
}

const workloadStatus = computed(() => {
  if (!props.workload) return 'Active'
  if ('status' in props.workload) {
    return (props.workload as Exclude<WorkloadInfo, CronJobInfo>).status
  }
  return 'Active'
})

const STATUS_BADGE_CLASSES: Record<string, string> = {
  Running: 'bg-emerald-500',
  Succeeded: 'bg-emerald-500',
  Active: 'bg-emerald-500',
  Progressing: 'bg-amber-500',
  Pending: 'bg-amber-500',
  Failed: 'bg-rose-500',
  CrashLoopBackOff: 'bg-rose-500'
}

const getStatusBadgeClass = (status: string) => STATUS_BADGE_CLASSES[status] ?? 'bg-emerald-500'

const workloadNamespace = computed(() => props.workload?.namespace ?? '')
const workloadName = computed(() => props.workload?.name ?? '')
const workloadAge = computed(() => props.workload?.age ?? '')
const workloadLabels = computed(() => props.workload?.labels ?? {})
const workloadAnnotations = computed(() => props.workload?.annotations ?? {})
const workloadImages = computed(() => props.workload?.images ?? [])

const getProp = <T,>(key: string): T | undefined => {
  if (props.workload && key in props.workload) {
    return (props.workload as unknown as Record<string, unknown>)[key] as T
  }
  return undefined
}

const replicas = computed(() => getProp<DaemonSetReplicas>('replicas') ?? null)
const desiredReplicas = computed(() => replicas.value?.desired)
const currentReplicas = computed(() => replicas.value?.current)
const readyReplicas = computed(() =>
  replicas.value && 'ready' in replicas.value
    ? (replicas.value as DaemonSetReplicas).ready
    : undefined
)

const available = computed(() => getProp<number>('available'))
const availableReplicas = computed(() =>
  available.value !== undefined
    ? available.value
    : replicas.value && 'available' in replicas.value
      ? (replicas.value as DaemonSetReplicas).available
      : undefined
)

const completions = computed(() => getProp<string>('completions'))
const duration = computed(() => getProp<string>('duration'))
const schedule = computed(() => getProp<string>('schedule'))
const suspend = computed(() => getProp<boolean>('suspend'))
const active = computed(() => getProp<number>('active'))
const lastSchedule = computed(() => getProp<string>('lastSchedule'))
const strategy = computed(() => getProp<string>('strategy'))
const minReadySeconds = computed(() => getProp<number>('minReadySeconds'))
const revisionHistory = computed(() => getProp<number>('revisionHistory'))

// Real Pods matching workload
const workloadPods = computed(() => {
  if (!props.workload) return []
  const ns = workloadNamespace.value
  const name = workloadName.value
  return pods.value.filter((p) => {
    if (p.namespace !== ns) return false
    return p.name.startsWith(name + '-') || p.name === name
  })
})

// Real Events matching workload
const workloadEvents = computed(() => {
  if (!props.workload) return []
  const ns = workloadNamespace.value
  const name = workloadName.value
  return clusterEvents.value.filter((ev) => {
    if (ev.namespace !== ns) return false
    return ev.objectName === name || ev.objectName.startsWith(name + '-')
  })
})

// Quick Actions
const viewLogs = () => {
  if (!props.workload) return
  router.push({
    name: 'logs',
    query: {
      namespace: workloadNamespace.value || 'default',
      workload: workloadName.value,
      kind: workloadKind.value
    }
  })
}

const viewPodLogs = (podName: string) => {
  router.push({
    name: 'logs',
    query: {
      namespace: workloadNamespace.value || 'default',
      workload: workloadName.value,
      kind: workloadKind.value,
      pod: podName
    }
  })
}

const editYaml = () => {
  if (!props.workload) return
  router.push({
    name: 'edit-workload',
    params: {
      kind: workloadKind.value,
      namespace: workloadNamespace.value || 'default',
      name: workloadName.value
    }
  })
}

// Live Raw YAML & JSON Fetching
const rawResourceData = ref<Record<string, unknown> | null>(null)
const rawYamlData = ref<string | null>(null)
const isYamlLoading = ref<boolean>(false)
const copied = ref<boolean>(false)

const fetchRawData = async () => {
  if (!props.workload || !props.visible) return
  isYamlLoading.value = true
  try {
    await kubernetesService.getResourceRaw({
      namespace: workloadNamespace.value || 'default',
      kind: workloadKind.value,
      name: workloadName.value
    })
  } catch (e) {
    console.error('Failed to fetch raw resource data:', e)
    isYamlLoading.value = false
  }
}

const handleRawData = (data: { kind?: string; name?: string; data?: unknown }) => {
  if (data && data.kind === workloadKind.value && data.name === workloadName.value) {
    if (data.data && typeof data.data === 'object') {
      rawResourceData.value = data.data as Record<string, unknown>
      rawYamlData.value = yaml.stringify(data.data)
    }
    isYamlLoading.value = false
  }
}

onMounted(() => {
  events.on(OrbitEvents.ResourceRawData, handleRawData)
})

onUnmounted(() => {
  events.off(OrbitEvents.ResourceRawData, handleRawData)
})

watch(
  () => [props.visible, props.workload],
  ([visible, workload]) => {
    if (visible && workload) {
      rawResourceData.value = null
      rawYamlData.value = null
      fetchRawData()
    } else if (!visible) {
      rawResourceData.value = null
      rawYamlData.value = null
      copied.value = false
    }
  },
  { immediate: true }
)

const generateYaml = (w: WorkloadInfo) => {
  const kind = getWorkloadKind(w)
  const labelsStr = w.labels
    ? Object.entries(w.labels)
        .map(([k, v]) => `    ${k}: ${v}`)
        .join('\n')
    : ''

  let specSection = ''
  if (kind === 'CronJob') {
    const cj = w as CronJobInfo
    specSection = `spec:
  schedule: "${cj.schedule}"
  suspend: ${cj.suspend}
  jobTemplate:
    spec:
      template:
        spec:
          containers:
          - name: ${cj.name}
            image: ${cj.images?.[0] || 'unknown'}`
  } else if (kind === 'Job') {
    const j = w as JobInfo
    specSection = `spec:
  template:
    spec:
      containers:
      - name: ${j.name}
        image: ${j.images?.[0] || 'unknown'}
      restartPolicy: OnFailure`
  } else {
    const reps = (w as unknown as Record<string, unknown>).replicas as
      Record<string, number> | undefined
    specSection = `spec:
  replicas: ${reps?.desired ?? 1}
  selector:
    matchLabels:
      app: ${w.name}
  template:
    metadata:
      labels:
        app: ${w.name}
    spec:
      containers:
      - name: ${w.name}
        image: ${w.images?.[0] || 'unknown'}`
  }

  return `apiVersion: apps/v1
kind: ${kind}
metadata:
  name: ${w.name}
  namespace: ${w.namespace}
  labels:
${labelsStr}
${specSection}
`
}

const displayedYaml = computed(() => {
  if (rawYamlData.value) return rawYamlData.value
  return props.workload ? generateYaml(props.workload) : ''
})

const copyYaml = async () => {
  if (!displayedYaml.value) return
  try {
    await navigator.clipboard.writeText(displayedYaml.value)
    copied.value = true
    setTimeout(() => {
      copied.value = false
    }, 2000)
  } catch (e) {
    console.error('Failed to copy YAML:', e)
  }
}
</script>

<template>
  <BaseResourceDrawer
    v-model:active-tab="activeTab"
    :visible="props.visible"
    :has-resource="!!props.workload"
    :title="workloadName"
    :kind="workloadKind"
    :kind-severity="getWorkloadSeverity(workloadKind)"
    :status-badge-class="getStatusBadgeClass(workloadStatus)"
    :namespace="workloadNamespace"
    :age="workloadAge"
    @update:visible="emit('update:visible', $event)"
  >
    <template #actions>
      <Button
        severity="secondary"
        size="small"
        variant="outlined"
        class="text-xs flex items-center gap-1.5"
        title="View Logs"
        @click="viewLogs"
      >
        <Terminal class="w-3.5 h-3.5" />
        <span>Logs</span>
      </Button>

      <Button
        severity="secondary"
        size="small"
        variant="outlined"
        class="text-xs flex items-center gap-1.5"
        title="Edit YAML"
        @click="editYaml"
      >
        <FileCode class="w-3.5 h-3.5" />
        <span>Edit</span>
      </Button>
    </template>

    <!-- Extra Tabs -->
    <template #extra-tabs>
      <Tab value="pods" class="text-xs! flex items-center gap-1.5 py-2.5 px-3">
        <Layers class="w-3.5 h-3.5" />
        <span>Pods ({{ workloadPods.length }})</span>
      </Tab>

      <Tab value="events" class="text-xs! flex items-center gap-1.5 py-2.5 px-3">
        <Activity class="w-3.5 h-3.5" />
        <span>Events ({{ workloadEvents.length }})</span>
      </Tab>
    </template>

    <!-- Overview Panel -->
    <template #overview>
      <WorkloadOverviewTab
        v-if="props.workload"
        :workload="props.workload"
        :workload-kind="workloadKind"
        :workload-namespace="workloadNamespace"
        :workload-age="workloadAge"
        :workload-status="workloadStatus"
        :replicas="replicas"
        :desired-replicas="desiredReplicas"
        :current-replicas="currentReplicas"
        :ready-replicas="readyReplicas"
        :available-replicas="availableReplicas"
        :completions="completions"
        :duration="duration"
        :schedule="schedule"
        :suspend="suspend"
        :active="active"
        :last-schedule="lastSchedule"
        :strategy="strategy"
        :min-ready-seconds="minReadySeconds"
        :revision-history="revisionHistory"
        :workload-images="workloadImages"
        :workload-labels="workloadLabels"
        :workload-annotations="workloadAnnotations"
        :raw-resource-data="rawResourceData"
      />
    </template>

    <!-- Extra Panels -->
    <template #extra-panels>
      <TabPanel value="pods">
        <WorkloadPodsTab
          :pods="workloadPods"
          :get-status-badge-class="getStatusBadgeClass"
          @view-pod-logs="viewPodLogs"
        />
      </TabPanel>

      <TabPanel value="events">
        <WorkloadEventsTab :events="workloadEvents" />
      </TabPanel>
    </template>

    <!-- YAML Panel -->
    <template #yaml>
      <ResourceYamlTab
        :displayed-yaml="displayedYaml"
        :is-yaml-loading="isYamlLoading"
        :copied="copied"
        @copy-yaml="copyYaml"
      />
    </template>
  </BaseResourceDrawer>
</template>
