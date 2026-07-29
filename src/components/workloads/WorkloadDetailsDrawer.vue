<script setup lang="ts">
import WorkloadEventsTab from '@/components/workloads/WorkloadEventsTab.vue'
import WorkloadOverviewTab from '@/components/workloads/WorkloadOverviewTab.vue'
import WorkloadPodsTab from '@/components/workloads/WorkloadPodsTab.vue'
import WorkloadYamlTab from '@/components/workloads/WorkloadYamlTab.vue'
import { kubernetesService } from '@/services/kubernetesService'
import { events } from '@/services/nativeService'
import { useKubernetesStore } from '@/stores/kubernetesStore'
import { OrbitEvents } from '@/types/events'
import type { CronJobInfo, DaemonSetReplicas, JobInfo, WorkloadInfo } from '@/types/kubernetes'
import { Activity, Clock, FileCode, Layers, Server, Terminal } from '@lucide/vue'
import { storeToRefs } from 'pinia'
import Button from 'primevue/button'
import Drawer from 'primevue/drawer'
import Tab from 'primevue/tab'
import TabList from 'primevue/tablist'
import TabPanel from 'primevue/tabpanel'
import TabPanels from 'primevue/tabpanels'
import Tabs from 'primevue/tabs'
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

const TYPE_BADGE_CLASSES: Record<string, string> = {
  Deployment: 'bg-blue-500/10 text-blue-400 border-blue-500/20',
  StatefulSet: 'bg-purple-500/10 text-purple-400 border-purple-500/20',
  DaemonSet: 'bg-teal-500/10 text-teal-400 border-teal-500/20',
  ReplicaSet: 'bg-indigo-500/10 text-indigo-400 border-indigo-500/20',
  Job: 'bg-amber-500/10 text-amber-400 border-amber-500/20',
  CronJob: 'bg-cyan-500/10 text-cyan-400 border-cyan-500/20'
}

const getTypeBadgeClass = (kind: string) =>
  TYPE_BADGE_CLASSES[kind] ?? 'bg-gray-500/10 text-gray-400 border-gray-500/20'

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

// Live Raw YAML Fetching
const rawYamlData = ref<string | null>(null)
const isYamlLoading = ref<boolean>(false)
const copied = ref<boolean>(false)

const fetchRawYaml = async () => {
  if (!props.workload || !props.visible) return
  isYamlLoading.value = true
  try {
    await kubernetesService.getResourceRaw({
      namespace: workloadNamespace.value || 'default',
      kind: workloadKind.value,
      name: workloadName.value
    })
  } catch (e) {
    console.error('Failed to fetch raw YAML:', e)
    isYamlLoading.value = false
  }
}

const handleRawData = (data: { kind?: string; name?: string; data?: unknown }) => {
  if (data && data.kind === workloadKind.value && data.name === workloadName.value) {
    if (data.data) {
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
  () => [props.visible, props.workload, activeTab.value],
  ([visible]) => {
    if (visible && props.workload) {
      if (activeTab.value === 'yaml' && !rawYamlData.value) {
        fetchRawYaml()
      }
    } else if (!visible) {
      rawYamlData.value = null
      copied.value = false
    }
  }
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
  <Drawer
    :visible="props.visible"
    @update:visible="emit('update:visible', $event)"
    position="right"
    class="w-full sm:max-w-lg border-l border-(--border) bg-(--bg-card) p-0"
    :header="workloadName || 'Workload Details'"
    :style="{ width: '36rem' }"
  >
    <template #header>
      <div class="flex items-center gap-3 w-full" v-if="props.workload">
        <div class="flex items-center gap-1.5">
          <span
            class="w-2.5 h-2.5 rounded-full animate-pulse"
            :class="getStatusBadgeClass(workloadStatus)"
          ></span>
          <span class="text-xs font-bold uppercase tracking-wider text-(--text-muted)">
            {{ workloadStatus || 'Active' }}
          </span>
        </div>
        <div
          class="text-xs text-(--text-muted) font-mono bg-(--bg-hover) px-2 py-0.5 rounded border border-(--border)"
        >
          ns/{{ workloadNamespace }}
        </div>
        <div
          class="text-[10px] font-semibold uppercase tracking-wider font-ui border px-2 py-0.5 rounded"
          :class="getTypeBadgeClass(workloadKind)"
        >
          {{ workloadKind }}
        </div>
      </div>
    </template>

    <div v-if="props.workload" class="h-full flex flex-col">
      <!-- Title Section & Actions -->
      <div
        class="p-6 border-b border-(--border) bg-(--bg-hover)/50 flex items-start justify-between gap-4"
      >
        <div>
          <h2
            class="text-xl font-bold text-(--text-primary) font-ui truncate mb-1"
            :title="workloadName"
          >
            {{ workloadName }}
          </h2>
          <div class="text-xs text-(--text-muted) flex items-center gap-2">
            <Clock class="w-3.5 h-3.5" />
            <span>Age: {{ workloadAge }}</span>
          </div>
        </div>

        <div class="flex items-center gap-2 shrink-0">
          <Button
            severity="secondary"
            size="small"
            variant="outlined"
            class="text-xs"
            @click="viewLogs"
          >
            <Terminal class="w-3.5 h-3.5 mr-1.5" />
            <span>Logs</span>
          </Button>

          <Button
            severity="secondary"
            size="small"
            variant="outlined"
            class="text-xs"
            @click="editYaml"
          >
            <FileCode class="w-3.5 h-3.5 mr-1.5" />
            <span>Edit</span>
          </Button>
        </div>
      </div>

      <!-- Tab Layout -->
      <div class="flex-1 flex flex-col min-h-0">
        <Tabs v-model:value="activeTab" class="flex-1 flex flex-col">
          <TabList class="border-b border-(--border) px-6 bg-(--bg-card)">
            <Tab
              value="overview"
              class="py-3 px-4 text-xs font-bold uppercase tracking-wider flex items-center gap-1.5"
            >
              <Server class="w-3.5 h-3.5" />
              <span>Overview</span>
            </Tab>

            <Tab
              value="pods"
              class="py-3 px-4 text-xs font-bold uppercase tracking-wider flex items-center gap-1.5"
            >
              <Layers class="w-3.5 h-3.5" />
              <span>Pods ({{ workloadPods.length }})</span>
            </Tab>

            <Tab
              value="events"
              class="py-3 px-4 text-xs font-bold uppercase tracking-wider flex items-center gap-1.5"
            >
              <Activity class="w-3.5 h-3.5" />
              <span>Events ({{ workloadEvents.length }})</span>
            </Tab>

            <Tab
              value="yaml"
              class="py-3 px-4 text-xs font-bold uppercase tracking-wider flex items-center gap-1.5"
            >
              <FileCode class="w-3.5 h-3.5" />
              <span>YAML</span>
            </Tab>
          </TabList>

          <TabPanels class="p-6 flex-1 overflow-y-auto min-h-0">
            <!-- OVERVIEW PANEL -->
            <TabPanel value="overview">
              <WorkloadOverviewTab
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
              />
            </TabPanel>

            <!-- PODS PANEL -->
            <TabPanel value="pods">
              <WorkloadPodsTab
                :pods="workloadPods"
                :get-status-badge-class="getStatusBadgeClass"
                @view-pod-logs="viewPodLogs"
              />
            </TabPanel>

            <!-- EVENTS PANEL -->
            <TabPanel value="events">
              <WorkloadEventsTab :events="workloadEvents" />
            </TabPanel>

            <!-- YAML PANEL -->
            <TabPanel value="yaml" class="h-full">
              <WorkloadYamlTab
                :displayed-yaml="displayedYaml"
                :is-yaml-loading="isYamlLoading"
                :copied="copied"
                @copy-yaml="copyYaml"
              />
            </TabPanel>
          </TabPanels>
        </Tabs>
      </div>
    </div>
  </Drawer>
</template>
