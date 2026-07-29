<script setup lang="ts">
import KeyValueBadgeList from '@/components/shared/KeyValueBadgeList.vue'
import ReplicasProgressBar from '@/components/shared/ReplicasProgressBar.vue'
import { kubernetesService } from '@/services/kubernetesService'
import { events } from '@/services/nativeService'
import { useKubernetesStore } from '@/stores/kubernetesStore'
import { OrbitEvents } from '@/types/events'
import type {
  CronJobInfo,
  DaemonSetReplicas,
  DeploymentInfo,
  JobInfo,
  WorkloadInfo
} from '@/types/kubernetes'
import { Activity, Check, Clock, Copy, FileCode, Info, Layers, Server, Terminal } from '@lucide/vue'
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

const getTypeBadgeClass = (kind: string) => {
  switch (kind) {
    case 'Deployment':
      return 'bg-blue-500/10 text-blue-400 border-blue-500/20'
    case 'StatefulSet':
      return 'bg-purple-500/10 text-purple-400 border-purple-500/20'
    case 'DaemonSet':
      return 'bg-teal-500/10 text-teal-400 border-teal-500/20'
    case 'ReplicaSet':
      return 'bg-indigo-500/10 text-indigo-400 border-indigo-500/20'
    case 'Job':
      return 'bg-amber-500/10 text-amber-400 border-amber-500/20'
    case 'CronJob':
      return 'bg-cyan-500/10 text-cyan-400 border-cyan-500/20'
    default:
      return 'bg-gray-500/10 text-gray-400 border-gray-500/20'
  }
}

const workloadStatus = computed(() => {
  if (!props.workload) return 'Active'
  if ('status' in props.workload) {
    return (props.workload as Exclude<WorkloadInfo, CronJobInfo>).status
  }
  return 'Active'
})

const getStatusBadgeClass = (status: string) => {
  switch (status) {
    case 'Running':
    case 'Succeeded':
    case 'Active':
      return 'bg-emerald-500'
    case 'Progressing':
    case 'Pending':
      return 'bg-amber-500'
    case 'Failed':
    case 'CrashLoopBackOff':
      return 'bg-rose-500'
    default:
      return 'bg-emerald-500'
  }
}

const workloadNamespace = computed(() => props.workload?.namespace ?? '')
const workloadName = computed(() => props.workload?.name ?? '')
const workloadAge = computed(() => props.workload?.age ?? '')
const workloadLabels = computed(() => props.workload?.labels ?? {})
const workloadAnnotations = computed(() => props.workload?.annotations ?? {})
const workloadImages = computed(() => props.workload?.images ?? [])

const replicas = computed(() => {
  if (!props.workload) return null
  if ('replicas' in props.workload) {
    return props.workload.replicas
  }
  return null
})

const desiredReplicas = computed(() => replicas.value?.desired)
const currentReplicas = computed(() => replicas.value?.current)
const readyReplicas = computed(() => {
  if (replicas.value && 'ready' in replicas.value) {
    return (replicas.value as DaemonSetReplicas).ready
  }
  return undefined
})

const available = computed(() => {
  if (!props.workload) return undefined
  if ('available' in props.workload) {
    return (props.workload as DeploymentInfo).available
  }
  return undefined
})

const availableReplicas = computed(() => {
  if (available.value !== undefined) return available.value
  if (replicas.value && 'available' in replicas.value) {
    return (replicas.value as DaemonSetReplicas).available
  }
  return undefined
})

const completions = computed(() => {
  if (!props.workload) return undefined
  if ('completions' in props.workload) {
    return (props.workload as JobInfo).completions
  }
  return undefined
})

const duration = computed(() => {
  if (!props.workload) return undefined
  if ('duration' in props.workload) {
    return (props.workload as JobInfo).duration
  }
  return undefined
})

const schedule = computed(() => {
  if (!props.workload) return undefined
  if ('schedule' in props.workload) {
    return (props.workload as CronJobInfo).schedule
  }
  return undefined
})

const suspend = computed(() => {
  if (!props.workload) return undefined
  if ('suspend' in props.workload) {
    return (props.workload as CronJobInfo).suspend
  }
  return undefined
})

const active = computed(() => {
  if (!props.workload) return undefined
  if ('active' in props.workload) {
    return (props.workload as CronJobInfo).active
  }
  return undefined
})

const lastSchedule = computed(() => {
  if (!props.workload) return undefined
  if ('lastSchedule' in props.workload) {
    return (props.workload as CronJobInfo).lastSchedule
  }
  return undefined
})

const strategy = computed(() => {
  if (!props.workload) return undefined
  if ('strategy' in props.workload) {
    return (props.workload as DeploymentInfo).strategy
  }
  return undefined
})

const minReadySeconds = computed(() => {
  if (!props.workload) return undefined
  if ('minReadySeconds' in props.workload) {
    return (props.workload as DeploymentInfo).minReadySeconds
  }
  return undefined
})

const revisionHistory = computed(() => {
  if (!props.workload) return undefined
  if ('revisionHistory' in props.workload) {
    return (props.workload as DeploymentInfo).revisionHistory
  }
  return undefined
})

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

// Generate a clean fallback YAML representation
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
    const obj = w as unknown as Record<string, unknown>
    const reps = obj.replicas as Record<string, number> | undefined
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
            <TabPanel value="overview" class="space-y-6">
              <!-- Replicas Progress Bars -->
              <ReplicasProgressBar
                v-if="replicas"
                :desired="desiredReplicas"
                :current="currentReplicas"
                :ready="readyReplicas"
                :available="availableReplicas"
              />

              <!-- Job Status -->
              <div v-if="completions !== undefined">
                <h3 class="text-[10px] font-bold text-(--text-muted) uppercase tracking-wider mb-3">
                  Job Status
                </h3>
                <div
                  class="bg-(--bg-hover)/50 border border-(--border) rounded-xl p-4 text-xs space-y-3"
                >
                  <div class="flex justify-between">
                    <span class="text-(--text-secondary) font-medium">Completions</span>
                    <span class="font-mono font-bold text-(--text-primary)">{{ completions }}</span>
                  </div>
                  <div v-if="duration" class="flex justify-between">
                    <span class="text-(--text-secondary) font-medium">Duration</span>
                    <span class="font-mono text-(--text-primary)">{{ duration }}</span>
                  </div>
                </div>
              </div>

              <!-- CronJob Schedule -->
              <div v-if="schedule">
                <h3 class="text-[10px] font-bold text-(--text-muted) uppercase tracking-wider mb-3">
                  CronJob Schedule
                </h3>
                <div
                  class="bg-(--bg-hover)/50 border border-(--border) rounded-xl p-4 text-xs space-y-3"
                >
                  <div class="flex justify-between">
                    <span class="text-(--text-secondary) font-medium">Schedule</span>
                    <span class="font-mono font-bold text-(--text-primary)">{{ schedule }}</span>
                  </div>
                  <div class="flex justify-between">
                    <span class="text-(--text-secondary) font-medium">Suspend</span>
                    <span class="font-mono text-(--text-primary)">{{
                      suspend ? 'True' : 'False'
                    }}</span>
                  </div>
                  <div class="flex justify-between">
                    <span class="text-(--text-secondary) font-medium">Active Jobs</span>
                    <span class="font-mono text-(--text-primary)">{{ active ?? 0 }}</span>
                  </div>
                  <div v-if="lastSchedule" class="flex justify-between">
                    <span class="text-(--text-secondary) font-medium">Last Schedule</span>
                    <span class="font-mono text-(--text-primary)">{{ lastSchedule }}</span>
                  </div>
                </div>
              </div>

              <!-- Configuration Metadata Grid -->
              <div>
                <h3 class="text-[10px] font-bold text-(--text-muted) uppercase tracking-wider mb-3">
                  Configuration
                </h3>
                <div
                  class="grid grid-cols-2 gap-4 bg-(--bg-hover)/30 border border-(--border) rounded-xl p-4 text-xs"
                >
                  <div>
                    <span class="text-(--text-muted) block mb-0.5">Kind</span>
                    <span class="font-semibold text-(--text-secondary)">{{ workloadKind }}</span>
                  </div>
                  <div>
                    <span class="text-(--text-muted) block mb-0.5">Namespace</span>
                    <span class="font-semibold text-(--text-secondary)">{{
                      workloadNamespace
                    }}</span>
                  </div>
                  <div>
                    <span class="text-(--text-muted) block mb-0.5">Age</span>
                    <span class="font-semibold text-(--text-secondary)">{{ workloadAge }}</span>
                  </div>
                  <div>
                    <span class="text-(--text-muted) block mb-0.5">Status</span>
                    <span class="font-semibold text-(--text-secondary)">{{ workloadStatus }}</span>
                  </div>
                  <div v-if="strategy">
                    <span class="text-(--text-muted) block mb-0.5">Strategy</span>
                    <span
                      class="font-semibold text-(--text-secondary) truncate block"
                      :title="strategy"
                    >
                      {{ strategy }}
                    </span>
                  </div>
                  <div v-if="minReadySeconds !== undefined">
                    <span class="text-(--text-muted) block mb-0.5">Min Ready Seconds</span>
                    <span class="font-mono text-(--text-secondary)">{{ minReadySeconds }}s</span>
                  </div>
                  <div v-if="revisionHistory !== undefined">
                    <span class="text-(--text-muted) block mb-0.5">Revision History Limit</span>
                    <span class="font-mono text-(--text-secondary)">{{ revisionHistory }}</span>
                  </div>
                  <div class="col-span-2" v-if="workloadImages && workloadImages.length">
                    <span class="text-(--text-muted) block mb-0.5">Container Images</span>
                    <div class="flex flex-wrap gap-1.5 mt-1">
                      <span
                        v-for="img in workloadImages"
                        :key="img"
                        class="bg-(--bg-hover) text-(--text-secondary) font-mono text-[10px] px-2 py-0.5 rounded border border-(--border) truncate max-w-full"
                        :title="img"
                      >
                        {{ img }}
                      </span>
                    </div>
                  </div>
                </div>
              </div>

              <!-- Labels & Annotations (Rendered ONCE) -->
              <KeyValueBadgeList :items="workloadLabels" title="Labels" variant="tag" />

              <KeyValueBadgeList :items="workloadAnnotations" title="Annotations" variant="list" />
            </TabPanel>

            <!-- PODS PANEL -->
            <TabPanel value="pods" class="space-y-4">
              <div class="text-[10px] font-bold text-(--text-muted) uppercase tracking-wider mb-1">
                Active Pods ({{ workloadPods.length }})
              </div>
              <div class="space-y-2.5">
                <template v-if="workloadPods.length > 0">
                  <div
                    v-for="pod in workloadPods"
                    :key="pod.name"
                    class="flex items-center justify-between p-3.5 bg-(--bg-hover)/30 border border-(--border) rounded-xl hover:bg-(--bg-hover)/50 transition-colors"
                  >
                    <div class="flex items-center gap-3 min-w-0">
                      <span
                        class="w-2.5 h-2.5 rounded-full shrink-0 animate-pulse"
                        :class="getStatusBadgeClass(pod.status)"
                      ></span>
                      <div class="min-w-0">
                        <span
                          class="text-xs font-semibold text-(--text-primary) font-mono truncate block"
                          :title="pod.name"
                        >
                          {{ pod.name }}
                        </span>
                        <div
                          class="flex items-center gap-3 text-[10px] text-(--text-muted) font-mono mt-0.5"
                        >
                          <span>IP: {{ pod.ip || 'N/A' }}</span>
                          <span>Node: {{ pod.node || 'N/A' }}</span>
                          <span v-if="pod.restarts !== undefined"
                            >Restarts: {{ pod.restarts }}</span
                          >
                          <span>Age: {{ pod.age }}</span>
                        </div>
                      </div>
                    </div>

                    <Button
                      severity="secondary"
                      size="small"
                      variant="text"
                      class="text-xs shrink-0"
                      title="View Logs"
                      @click="viewPodLogs(pod.name)"
                    >
                      <Terminal class="w-3.5 h-3.5" />
                    </Button>
                  </div>
                </template>
                <div
                  v-else
                  class="text-center py-10 text-xs text-(--text-muted) flex flex-col items-center gap-2"
                >
                  <Info class="w-8 h-8 text-(--text-muted)/50" />
                  <span>No active pods found for this workload.</span>
                </div>
              </div>
            </TabPanel>

            <!-- EVENTS PANEL -->
            <TabPanel value="events" class="space-y-4">
              <div class="text-[10px] font-bold text-(--text-muted) uppercase tracking-wider mb-1">
                Recent Events ({{ workloadEvents.length }})
              </div>
              <div
                v-if="workloadEvents.length > 0"
                class="relative pl-4 border-l border-(--border) space-y-4 ml-2"
              >
                <div v-for="(ev, idx) in workloadEvents" :key="idx" class="relative">
                  <span
                    class="absolute -left-5.25 top-1 w-2.5 h-2.5 rounded-full ring-4 ring-(--bg-card)"
                    :class="ev.type === 'Warning' ? 'bg-rose-500' : 'bg-emerald-500'"
                  ></span>
                  <div class="flex items-center gap-2">
                    <span class="text-xs font-semibold text-(--text-primary)">{{ ev.reason }}</span>
                    <span
                      class="text-[9px] px-1.5 py-0.2 rounded font-mono border"
                      :class="
                        ev.type === 'Warning'
                          ? 'bg-rose-500/10 text-rose-400 border-rose-500/20'
                          : 'bg-emerald-500/10 text-emerald-400 border-emerald-500/20'
                      "
                    >
                      {{ ev.type }}
                    </span>
                    <span v-if="ev.count > 1" class="text-[9px] text-(--text-muted) font-mono">
                      (x{{ ev.count }})
                    </span>
                  </div>
                  <div class="text-[10px] text-(--text-muted) mt-0.5">
                    {{ ev.message }}
                  </div>
                  <div
                    class="text-[9px] font-mono text-(--text-muted) mt-1 flex items-center gap-2"
                  >
                    <span>Object: {{ ev.objectKind }}/{{ ev.objectName }}</span>
                    <span>•</span>
                    <span>Source: {{ ev.source }}</span>
                    <span>•</span>
                    <span>{{ ev.lastSeen || ev.time || 'recent' }}</span>
                  </div>
                </div>
              </div>
              <div
                v-else
                class="text-center py-10 text-xs text-(--text-muted) flex flex-col items-center gap-2"
              >
                <Info class="w-8 h-8 text-(--text-muted)/50" />
                <span>No recent events recorded for this workload.</span>
              </div>
            </TabPanel>

            <!-- YAML PANEL -->
            <TabPanel value="yaml" class="h-full flex flex-col space-y-3">
              <div class="flex items-center justify-between">
                <span class="text-[10px] font-bold text-(--text-muted) uppercase tracking-wider">
                  Live Kubernetes Manifest
                </span>
                <Button
                  severity="secondary"
                  size="small"
                  variant="outlined"
                  class="text-xs"
                  @click="copyYaml"
                >
                  <Component :is="copied ? Check : Copy" class="w-3.5 h-3.5 mr-1.5" />
                  <span>{{ copied ? 'Copied!' : 'Copy YAML' }}</span>
                </Button>
              </div>

              <div
                class="flex-1 min-h-64 bg-zinc-950 rounded-lg border border-zinc-800 p-4 overflow-auto font-mono text-[10px] text-zinc-300 leading-relaxed"
              >
                <div v-if="isYamlLoading" class="text-zinc-500 italic py-4 text-center">
                  Loading live YAML manifest...
                </div>
                <pre v-else>{{ displayedYaml }}</pre>
              </div>
            </TabPanel>
          </TabPanels>
        </Tabs>
      </div>
    </div>
  </Drawer>
</template>
