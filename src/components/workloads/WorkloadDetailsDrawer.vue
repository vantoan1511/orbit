<script setup lang="ts">
import { ref } from 'vue'
import Drawer from 'primevue/drawer'
import Tabs from 'primevue/tabs'
import TabList from 'primevue/tablist'
import Tab from 'primevue/tab'
import TabPanels from 'primevue/tabpanels'
import TabPanel from 'primevue/tabpanel'
import { Clock, Tag } from '@lucide/vue'

import type { WorkloadInfo, CronJobInfo, JobInfo } from '../../types/kubernetes'

const props = defineProps<{
  visible: boolean
  workload: WorkloadInfo | null
}>()

const emit = defineEmits<{
  (e: 'update:visible', value: boolean): void
}>()

const activeTab = ref('overview')

const getWorkloadKind = (w: WorkloadInfo): string => {
  if ('schedule' in w) return 'CronJob'
  if ('completions' in w) return 'Job'
  if ('strategy' in w) return 'Deployment'
  if ('replicas' in w && w.replicas) {
    const reps = w.replicas as Record<string, unknown>
    if ('ready' in reps && 'upToDate' in reps) return 'DaemonSet'
    if (w.name.includes('stateful')) return 'StatefulSet' // fallback
    return 'ReplicaSet' // fallback
  }
  return 'Workload'
}

// Generate a dummy YAML representation for the YAML tab
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
</script>

<template>
  <Drawer
    :visible="props.visible"
    @update:visible="emit('update:visible', $event)"
    position="right"
    class="w-full sm:max-w-lg border-l border-(--border) bg-(--bg-card) p-0"
    :header="props.workload?.name || 'Workload Details'"
    :style="{ width: '32rem' }"
  >
    <template #header>
      <div class="flex items-center gap-3 w-full" v-if="props.workload">
        <div class="flex items-center gap-1.5">
          <span
            class="w-2.5 h-2.5 rounded-full animate-pulse"
            :class="
              props.workload.status === 'Running' || props.workload.status === 'Succeeded'
                ? 'bg-emerald-500'
                : props.workload.status === 'Progressing' || props.workload.status === 'Active'
                  ? 'bg-amber-500'
                  : 'bg-rose-500'
            "
          ></span>
          <span class="text-xs font-bold uppercase tracking-wider text-(--text-muted)">
            {{ props.workload.status || 'Active' }}
          </span>
        </div>
        <div
          class="text-xs text-(--text-muted) font-mono bg-(--bg-hover) px-2 py-0.5 rounded border border-(--border)"
        >
          ns/{{ props.workload.namespace }}
        </div>
      </div>
    </template>

    <div v-if="props.workload" class="h-full flex flex-col">
      <!-- Title Section -->
      <div class="p-6 border-b border-(--border) bg-(--bg-hover)/50">
        <h2 class="text-xl font-bold text-(--text-primary) font-ui truncate mb-1">
          {{ props.workload.name }}
        </h2>
        <div class="text-xs text-(--text-muted) flex items-center gap-2">
          <Clock class="w-3.5 h-3.5" />
          <span>Age: {{ props.workload.age }}</span>
        </div>
      </div>

      <!-- Tab Layout -->
      <div class="flex-1 flex flex-col min-h-0">
        <Tabs v-model:value="activeTab" class="flex-1 flex flex-col">
          <TabList class="border-b border-(--border) px-6 bg-(--bg-card)">
            <Tab value="overview" class="py-3 px-4 text-xs font-bold uppercase tracking-wider"
              >Overview</Tab
            >
            <Tab value="pods" class="py-3 px-4 text-xs font-bold uppercase tracking-wider"
              >Pods</Tab
            >
            <Tab value="events" class="py-3 px-4 text-xs font-bold uppercase tracking-wider"
              >Events</Tab
            >
            <Tab value="yaml" class="py-3 px-4 text-xs font-bold uppercase tracking-wider"
              >YAML</Tab
            >
          </TabList>

          <TabPanels class="p-6 flex-1 overflow-y-auto min-h-0">
            <!-- OVERVIEW PANEL -->
            <TabPanel value="overview" class="space-y-6">
              <!-- Replicas Progress Bars -->
              <div v-if="props.workload.replicas">
                <h3 class="text-[10px] font-bold text-(--text-muted) uppercase tracking-wider mb-3">
                  Replicas Status
                </h3>
                <div class="bg-(--bg-hover)/50 border border-(--border) rounded-xl p-4 space-y-4">
                  <div>
                    <div class="flex justify-between text-xs mb-1">
                      <span class="text-(--text-secondary) font-medium">Desired Replicas</span>
                      <span class="font-mono font-bold text-(--text-primary)">{{
                        props.workload.replicas.desired
                      }}</span>
                    </div>
                    <div class="w-full h-1.5 rounded-full bg-(--bg-hover) overflow-hidden">
                      <div class="h-full rounded-full bg-blue-500" style="width: 100%"></div>
                    </div>
                  </div>

                  <div>
                    <div class="flex justify-between text-xs mb-1">
                      <span class="text-(--text-secondary) font-medium">Current Replicas</span>
                      <span class="font-mono font-bold text-(--text-primary)">{{
                        props.workload.replicas.current
                      }}</span>
                    </div>
                    <div class="w-full h-1.5 rounded-full bg-(--bg-hover) overflow-hidden">
                      <div
                        class="h-full rounded-full bg-indigo-500"
                        :style="{
                          width:
                            (props.workload.replicas.desired
                              ? (props.workload.replicas.current /
                                  props.workload.replicas.desired) *
                                100
                              : 0) + '%'
                        }"
                      ></div>
                    </div>
                  </div>

                  <div v-if="props.workload.replicas.ready !== undefined">
                    <div class="flex justify-between text-xs mb-1">
                      <span class="text-(--text-secondary) font-medium">Ready Replicas</span>
                      <span class="font-mono font-bold text-(--text-primary)">{{
                        props.workload.replicas.ready
                      }}</span>
                    </div>
                    <div class="w-full h-1.5 rounded-full bg-(--bg-hover) overflow-hidden">
                      <div
                        class="h-full rounded-full bg-emerald-500"
                        :style="{
                          width:
                            (props.workload.replicas.desired
                              ? (props.workload.replicas.ready / props.workload.replicas.desired) *
                                100
                              : 0) + '%'
                        }"
                      ></div>
                    </div>
                  </div>

                  <div
                    v-if="
                      props.workload.available !== undefined ||
                      props.workload.replicas.available !== undefined
                    "
                  >
                    <div class="flex justify-between text-xs mb-1">
                      <span class="text-(--text-secondary) font-medium">Available Replicas</span>
                      <span class="font-mono font-bold text-(--text-primary)">{{
                        props.workload.available !== undefined
                          ? props.workload.available
                          : props.workload.replicas.available
                      }}</span>
                    </div>
                    <div class="w-full h-1.5 rounded-full bg-(--bg-hover) overflow-hidden">
                      <div
                        class="h-full rounded-full bg-emerald-500"
                        :style="{
                          width:
                            (props.workload.replicas.desired
                              ? ((props.workload.available !== undefined
                                  ? props.workload.available
                                  : props.workload.replicas.available) /
                                  props.workload.replicas.desired) *
                                100
                              : 0) + '%'
                        }"
                      ></div>
                    </div>
                  </div>
                </div>
              </div>

              <!-- Job Status -->
              <div v-if="props.workload.completions">
                <h3 class="text-[10px] font-bold text-(--text-muted) uppercase tracking-wider mb-3">
                  Job Status
                </h3>
                <div
                  class="bg-(--bg-hover)/50 border border-(--border) rounded-xl p-4 text-xs space-y-3"
                >
                  <div class="flex justify-between">
                    <span class="text-(--text-secondary) font-medium">Completions</span>
                    <span class="font-mono font-bold text-(--text-primary)">{{
                      props.workload.completions
                    }}</span>
                  </div>
                  <div v-if="props.workload.duration" class="flex justify-between">
                    <span class="text-(--text-secondary) font-medium">Duration</span>
                    <span class="font-mono text-(--text-primary)">{{
                      props.workload.duration
                    }}</span>
                  </div>
                </div>
              </div>

              <!-- CronJob Schedule -->
              <div v-if="props.workload.schedule">
                <h3 class="text-[10px] font-bold text-(--text-muted) uppercase tracking-wider mb-3">
                  CronJob Schedule
                </h3>
                <div
                  class="bg-(--bg-hover)/50 border border-(--border) rounded-xl p-4 text-xs space-y-3"
                >
                  <div class="flex justify-between">
                    <span class="text-(--text-secondary) font-medium">Schedule</span>
                    <span class="font-mono font-bold text-(--text-primary)">{{
                      props.workload.schedule
                    }}</span>
                  </div>
                  <div class="flex justify-between">
                    <span class="text-(--text-secondary) font-medium">Suspend</span>
                    <span class="font-mono text-(--text-primary)">{{
                      props.workload.suspend ? 'True' : 'False'
                    }}</span>
                  </div>
                  <div class="flex justify-between">
                    <span class="text-(--text-secondary) font-medium">Active Jobs</span>
                    <span class="font-mono text-(--text-primary)">{{ props.workload.active }}</span>
                  </div>
                  <div v-if="props.workload.lastSchedule" class="flex justify-between">
                    <span class="text-(--text-secondary) font-medium">Last Schedule</span>
                    <span class="font-mono text-(--text-primary)">{{
                      props.workload.lastSchedule
                    }}</span>
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
                    <span class="text-(--text-muted) block mb-0.5">Namespace</span>
                    <span class="font-semibold text-(--text-secondary)">{{
                      props.workload.namespace
                    }}</span>
                  </div>
                  <div v-if="props.workload.strategy">
                    <span class="text-(--text-muted) block mb-0.5">Strategy</span>
                    <span
                      class="font-semibold text-(--text-secondary) truncate block"
                      :title="props.workload.strategy"
                    >
                      {{ props.workload.strategy }}
                    </span>
                  </div>
                  <div v-if="props.workload.minReadySeconds !== undefined">
                    <span class="text-(--text-muted) block mb-0.5">Min Ready Seconds</span>
                    <span class="font-mono text-(--text-secondary)"
                      >{{ props.workload.minReadySeconds }}s</span
                    >
                  </div>
                  <div v-if="props.workload.revisionHistory !== undefined">
                    <span class="text-(--text-muted) block mb-0.5">Revision History Limit</span>
                    <span class="font-mono text-(--text-secondary)">{{
                      props.workload.revisionHistory
                    }}</span>
                  </div>
                  <div
                    class="col-span-2"
                    v-if="props.workload.images && props.workload.images.length"
                  >
                    <span class="text-(--text-muted) block mb-0.5">Container Images</span>
                    <div class="space-y-1 mt-1">
                      <span
                        v-for="img in props.workload.images"
                        :key="img"
                        class="inline-block bg-(--bg-hover) text-(--text-secondary) font-mono text-[10px] px-2 py-1 rounded border border-(--border) mr-2 mb-1 truncate max-w-full"
                      >
                        {{ img }}
                      </span>
                    </div>
                  </div>
                </div>
              </div>

              <!-- Labels & Annotations -->
              <div v-if="props.workload.labels && Object.keys(props.workload.labels).length">
                <h3 class="text-[10px] font-bold text-(--text-muted) uppercase tracking-wider mb-3">
                  Labels
                </h3>
                <div class="flex flex-wrap gap-2">
                  <div
                    v-for="(val, key) in props.workload.labels"
                    :key="key"
                    class="flex items-center gap-1 bg-violet-500/5 border border-violet-500/10 rounded-md text-[10px] px-2 py-0.5 text-violet-400 font-mono"
                  >
                    <Tag class="w-3 h-3" />
                    <span>{{ key }}={{ val }}</span>
                  </div>
                </div>
              </div>

              <div
                v-if="props.workload.annotations && Object.keys(props.workload.annotations).length"
              >
                <h3 class="text-[10px] font-bold text-(--text-muted) uppercase tracking-wider mb-3">
                  Annotations
                </h3>
                <div class="space-y-1.5">
                  <div
                    v-for="(val, key) in props.workload.annotations"
                    :key="key"
                    class="p-2 rounded bg-(--bg-hover)/50 border border-(--border) text-[10px] font-mono text-(--text-secondary) flex justify-between gap-4"
                  >
                    <span class="text-(--text-muted) truncate shrink-0">{{ key }}</span>
                    <span class="truncate text-right">{{ val }}</span>
                  </div>
                </div>
              </div>
            </TabPanel>

            <!-- PODS PANEL -->
            <TabPanel value="pods" class="space-y-4">
              <div class="text-[10px] font-bold text-(--text-muted) uppercase tracking-wider mb-1">
                Active Pods
              </div>
              <div class="space-y-2">
                <template v-if="props.workload.replicas && props.workload.replicas.current > 0">
                  <div
                    v-for="i in props.workload.replicas.current"
                    :key="i"
                    class="flex items-center justify-between p-3.5 bg-(--bg-hover)/30 border border-(--border) rounded-xl"
                  >
                    <div class="flex items-center gap-3">
                      <span class="w-2 h-2 rounded-full bg-emerald-500"></span>
                      <div>
                        <span class="text-xs font-semibold text-(--text-secondary) font-mono">
                          {{ props.workload.name }}-{{ Math.random().toString(36).substring(2, 7) }}
                        </span>
                        <span class="block text-[10px] text-(--text-muted) mt-0.5"
                          >IP: 10.244.1.{{ 20 + i }}</span
                        >
                      </div>
                    </div>
                    <div class="text-[10px] font-mono text-(--text-muted)">Ready (1/1)</div>
                  </div>
                </template>
                <div v-else class="text-center py-6 text-xs text-(--text-muted)">
                  No active pods for this workload.
                </div>
              </div>
            </TabPanel>

            <!-- EVENTS PANEL -->
            <TabPanel value="events" class="space-y-4">
              <div class="text-[10px] font-bold text-(--text-muted) uppercase tracking-wider mb-1">
                Recent Events
              </div>
              <div class="relative pl-4 border-l border-(--border) space-y-4 ml-2">
                <div class="relative">
                  <span
                    class="absolute -left-[21px] top-1 w-2.5 h-2.5 rounded-full bg-emerald-500 ring-4 ring-(--bg-card)"
                  ></span>
                  <div class="text-xs font-semibold text-(--text-primary)">SyncLoop</div>
                  <div class="text-[10px] text-(--text-muted) mt-0.5">
                    Resource reconciled successfully by controller
                  </div>
                  <div class="text-[9px] font-mono text-(--text-muted) mt-1">2m ago</div>
                </div>
              </div>
            </TabPanel>

            <!-- YAML PANEL -->
            <TabPanel value="yaml" class="h-full flex flex-col">
              <div
                class="flex-1 min-h-0 bg-zinc-950 rounded-lg border border-zinc-800 p-4 overflow-auto font-mono text-[10px] text-zinc-300 leading-relaxed"
              >
                <pre>{{ generateYaml(props.workload) }}</pre>
              </div>
            </TabPanel>
          </TabPanels>
        </Tabs>
      </div>
    </div>
  </Drawer>
</template>
