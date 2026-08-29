<script setup lang="ts">
import ActivePortForwardsList from '@/components/shared/ActivePortForwardsList.vue'
import KeyValueBadgeList from '@/components/shared/KeyValueBadgeList.vue'
import ReactiveAge from '@/components/shared/ReactiveAge.vue'
import ReplicasProgressBar from '@/components/shared/ReplicasProgressBar.vue'
import type { DaemonSetReplicas, WorkloadInfo } from '@/types/kubernetes'
import { computed } from 'vue'
import * as yaml from 'yaml'

const props = defineProps<{
  workload: WorkloadInfo | null
  workloadKind: string
  workloadNamespace: string
  workloadAge: string
  workloadStatus: string
  replicas: DaemonSetReplicas | null
  desiredReplicas?: number
  currentReplicas?: number
  readyReplicas?: number
  availableReplicas?: number
  completions?: string
  duration?: string
  schedule?: string
  suspend?: boolean
  active?: number
  lastSchedule?: string
  strategy?: string
  minReadySeconds?: number
  revisionHistory?: number
  workloadImages: string[]
  workloadLabels: Record<string, string>
  workloadAnnotations: Record<string, string>
  rawResourceData?: Record<string, unknown> | null
}>()

// 1. Revision info
const revision = computed<string | undefined>(() => {
  const meta = props.rawResourceData?.metadata as Record<string, unknown> | undefined
  const annotations = (meta?.annotations || props.workloadAnnotations) as
    Record<string, string> | undefined
  return annotations?.['deployment.kubernetes.io/revision']
})

// 2. Pod Selectors (matchLabels)
const selectorMatchLabels = computed<Record<string, string>>(() => {
  const spec = props.rawResourceData?.spec as Record<string, unknown> | undefined
  const selector = spec?.selector as Record<string, unknown> | undefined
  return (selector?.matchLabels as Record<string, string>) || {}
})

// 3. Conditions
interface ConditionItem {
  type: string
  status: string
  reason?: string
  message?: string
  lastTransitionTime?: string
  lastUpdateTime?: string
}

const conditions = computed<ConditionItem[]>(() => {
  const status = props.rawResourceData?.status as Record<string, unknown> | undefined
  return Array.isArray(status?.conditions) ? (status.conditions as ConditionItem[]) : []
})

const getConditionSeverity = (status: string, type: string) => {
  // Failure / Pressure / Degraded conditions where "True" indicates a problem
  if (
    type === 'ReplicaFailure' ||
    type.endsWith('Pressure') ||
    type.endsWith('Failure') ||
    type.endsWith('Degraded')
  ) {
    if (status === 'True') return 'danger'
    if (status === 'False') return 'secondary'
    return 'warn'
  }

  // Standard health/progress conditions (Available, Progressing, Ready, etc.)
  if (status === 'True') return 'success'
  if (status === 'False') return 'danger'
  return 'warn'
}

// Helper to extract pod spec from standard workloads or CronJobs
const podSpec = computed<Record<string, unknown> | undefined>(() => {
  if (!props.rawResourceData) return undefined
  const spec = props.rawResourceData.spec as Record<string, unknown> | undefined
  if (!spec) return undefined
  const jobTemplate = spec.jobTemplate as Record<string, unknown> | undefined
  const jobSpec = jobTemplate?.spec as Record<string, unknown> | undefined
  const cronTemplate = jobSpec?.template as Record<string, unknown> | undefined
  if (cronTemplate?.spec) {
    return cronTemplate.spec as Record<string, unknown>
  }
  const template = spec.template as Record<string, unknown> | undefined
  return template?.spec as Record<string, unknown> | undefined
})

// 4. Scheduling Constraints
const nodeSelector = computed<Record<string, string>>(() => {
  return (podSpec.value?.nodeSelector as Record<string, string>) || {}
})

interface TolerationItem {
  key?: string
  operator?: string
  value?: string
  effect?: string
  tolerationSeconds?: number
}

const tolerations = computed<TolerationItem[]>(() => {
  return Array.isArray(podSpec.value?.tolerations)
    ? (podSpec.value.tolerations as TolerationItem[])
    : []
})

const formatToleration = (t: TolerationItem): string => {
  const parts: string[] = []
  if (t.key) {
    if (t.operator === 'Exists') {
      parts.push(`${t.key} Exists`)
    } else {
      parts.push(
        `${t.key}${t.operator === 'Equal' ? '=' : ` ${t.operator || '='} `}${t.value ?? ''}`
      )
    }
  } else if (t.operator === 'Exists') {
    parts.push('All Keys Exists')
  }
  if (t.effect) parts.push(`:${t.effect}`)
  if (t.tolerationSeconds !== undefined) parts.push(`(${t.tolerationSeconds}s)`)
  return parts.join(' ')
}

const affinityYaml = computed<string | null>(() => {
  if (
    podSpec.value?.affinity &&
    typeof podSpec.value.affinity === 'object' &&
    Object.keys(podSpec.value.affinity).length > 0
  ) {
    return yaml.stringify(podSpec.value.affinity)
  }
  return null
})

// 5. Pod Template Containers
interface ContainerEnvItem {
  name: string
  value?: string
  valueFrom?: {
    configMapKeyRef?: { name: string; key: string }
    secretKeyRef?: { name: string; key: string }
    fieldRef?: { fieldPath: string }
    resourceFieldRef?: { resource: string }
  }
}

interface ContainerSpecItem {
  name: string
  image: string
  ports?: { name?: string; containerPort: number; protocol?: string }[]
  resources?: {
    requests?: { cpu?: string; memory?: string }
    limits?: { cpu?: string; memory?: string }
  }
  env?: ContainerEnvItem[]
  envFrom?: { prefix?: string; configMapRef?: { name: string }; secretRef?: { name: string } }[]
}

const templateContainers = computed<ContainerSpecItem[]>(() => {
  return Array.isArray(podSpec.value?.containers)
    ? (podSpec.value.containers as ContainerSpecItem[])
    : []
})

const formatEnvValue = (e: ContainerEnvItem): string => {
  if (e.value !== undefined) return e.value
  if (e.valueFrom) {
    const vf = e.valueFrom
    if (vf.configMapKeyRef) {
      return `ConfigMap: ${vf.configMapKeyRef.name}.${vf.configMapKeyRef.key}`
    }
    if (vf.secretKeyRef) {
      return `Secret: ${vf.secretKeyRef.name}.${vf.secretKeyRef.key}`
    }
    if (vf.fieldRef) {
      return `Field: ${vf.fieldRef.fieldPath}`
    }
    if (vf.resourceFieldRef) {
      return `Resource: ${vf.resourceFieldRef.resource}`
    }
  }
  return '-'
}
</script>

<template>
  <div class="space-y-6">
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
      <h3 class="text-[10px] font-bold text-muted-color uppercase tracking-wider mb-3">
        Job Status
      </h3>
      <div class="bg-(--bg-hover)/50 border border-(--border) rounded-xl p-4 text-xs space-y-3">
        <div class="flex justify-between">
          <span class="text-muted-color font-medium">Completions</span>
          <span class="font-mono font-bold text-primary">{{ completions }}</span>
        </div>
        <div v-if="duration" class="flex justify-between">
          <span class="text-muted-color font-medium">Duration</span>
          <span class="font-mono text-primary">{{ duration }}</span>
        </div>
      </div>
    </div>

    <!-- CronJob Schedule -->
    <div v-if="schedule">
      <h3 class="text-[10px] font-bold text-muted-color uppercase tracking-wider mb-3">
        CronJob Schedule
      </h3>
      <div class="bg-(--bg-hover)/50 border border-(--border) rounded-xl p-4 text-xs space-y-3">
        <div class="flex justify-between">
          <span class="text-muted-color font-medium">Schedule</span>
          <span class="font-mono font-bold text-primary">{{ schedule }}</span>
        </div>
        <div class="flex justify-between">
          <span class="text-muted-color font-medium">Suspend</span>
          <span class="font-mono text-primary">{{ suspend ? 'True' : 'False' }}</span>
        </div>
        <div class="flex justify-between">
          <span class="text-muted-color font-medium">Active Jobs</span>
          <span class="font-mono text-primary">{{ active ?? 0 }}</span>
        </div>
        <div v-if="lastSchedule" class="flex justify-between">
          <span class="text-muted-color font-medium">Last Schedule</span>
          <span class="font-mono text-primary">{{ lastSchedule }}</span>
        </div>
      </div>
    </div>

    <!-- Configuration Metadata Grid -->
    <div>
      <h3 class="text-[10px] font-bold text-muted-color uppercase tracking-wider mb-3">
        Configuration
      </h3>
      <div
        class="grid grid-cols-2 gap-4 bg-(--bg-hover)/30 border border-(--border) rounded-xl p-4 text-xs"
      >
        <div>
          <span class="text-muted-color block mb-0.5">Kind</span>
          <span class="font-semibold text-muted-color">{{ workloadKind }}</span>
        </div>
        <div>
          <span class="text-muted-color block mb-0.5">Namespace</span>
          <span class="font-semibold text-muted-color">{{ workloadNamespace }}</span>
        </div>
        <div>
          <span class="text-muted-color block mb-0.5">Age</span>
          <span class="font-semibold text-muted-color"><ReactiveAge :age="workloadAge" /></span>
        </div>
        <div>
          <span class="text-muted-color block mb-0.5">Status</span>
          <span class="font-semibold text-muted-color">{{ workloadStatus }}</span>
        </div>
        <div v-if="revision">
          <span class="text-muted-color block mb-0.5">Revision</span>
          <span class="font-mono font-semibold text-primary">#{{ revision }}</span>
        </div>
        <div v-if="strategy">
          <span class="text-muted-color block mb-0.5">Strategy</span>
          <span class="font-semibold text-muted-color truncate block" :title="strategy">
            {{ strategy }}
          </span>
        </div>
        <div v-if="minReadySeconds !== undefined">
          <span class="text-muted-color block mb-0.5">Min Ready Seconds</span>
          <span class="font-mono text-muted-color">{{ minReadySeconds }}s</span>
        </div>
        <div v-if="revisionHistory !== undefined">
          <span class="text-muted-color block mb-0.5">Revision History Limit</span>
          <span class="font-mono text-muted-color">{{ revisionHistory }}</span>
        </div>
        <div class="col-span-2" v-if="workloadImages && workloadImages.length">
          <span class="text-muted-color block mb-0.5">Container Images</span>
          <div class="flex flex-wrap gap-1.5 mt-1">
            <Tag
              v-for="(img, idx) in workloadImages"
              :key="idx"
              severity="secondary"
              class="font-mono truncate max-w-full"
              :title="img"
              :value="img"
            />
          </div>
        </div>
      </div>
    </div>

    <!-- Active Port Forwards -->
    <ActivePortForwardsList
      :kind="workloadKind"
      :namespace="workloadNamespace"
      :name="workload?.name || ''"
    />

    <!-- Pod Selectors -->
    <KeyValueBadgeList
      v-if="Object.keys(selectorMatchLabels).length > 0"
      :items="selectorMatchLabels"
      title="Pod Selectors (matchLabels)"
      variant="tag"
    />

    <!-- Conditions -->
    <div v-if="conditions.length > 0">
      <h3 class="text-[10px] font-bold text-muted-color uppercase tracking-wider mb-3">
        Conditions
      </h3>
      <div class="space-y-2">
        <div
          v-for="cond in conditions"
          :key="cond.type"
          class="bg-(--bg-hover)/30 border border-(--border) rounded-xl p-3 text-xs flex flex-col gap-1.5"
        >
          <div class="flex items-center justify-between">
            <div class="flex items-center gap-2">
              <Tag
                :value="cond.status"
                :severity="getConditionSeverity(cond.status, cond.type)"
                class="font-mono text-[10px] px-1.5 py-0.5"
              />
              <span class="font-bold font-mono text-primary">{{ cond.type }}</span>
              <span v-if="cond.reason" class="text-muted-color text-[11px] font-mono">
                ({{ cond.reason }})
              </span>
            </div>
            <span v-if="cond.lastTransitionTime" class="text-[10px] text-muted-color font-mono">
              {{ cond.lastTransitionTime }}
            </span>
          </div>
          <p v-if="cond.message" class="text-muted-color text-xs leading-relaxed font-mono pl-0.5">
            {{ cond.message }}
          </p>
        </div>
      </div>
    </div>

    <!-- Scheduling Constraints -->
    <div
      v-if="Object.keys(nodeSelector).length > 0 || tolerations.length > 0 || affinityYaml"
      class="space-y-4"
    >
      <h3 class="text-[10px] font-bold text-muted-color uppercase tracking-wider mb-3">
        Scheduling Constraints
      </h3>

      <!-- Node Selector -->
      <KeyValueBadgeList
        v-if="Object.keys(nodeSelector).length > 0"
        :items="nodeSelector"
        title="Node Selector"
        variant="tag"
      />

      <!-- Tolerations -->
      <div v-if="tolerations.length > 0">
        <span class="text-xs font-semibold text-muted-color block mb-2">Tolerations</span>
        <div class="flex flex-wrap gap-1.5">
          <Tag
            v-for="(tol, idx) in tolerations"
            :key="idx"
            severity="secondary"
            class="font-mono text-[11px] truncate max-w-full"
            :value="formatToleration(tol)"
            :title="formatToleration(tol)"
          />
        </div>
      </div>

      <!-- Affinity -->
      <div v-if="affinityYaml">
        <span class="text-xs font-semibold text-muted-color block mb-2">Affinity</span>
        <pre
          class="bg-(--bg-hover)/40 border border-(--border) rounded-lg p-3 text-[11px] font-mono text-muted-color overflow-x-auto leading-relaxed"
          >{{ affinityYaml }}</pre>
      </div>
    </div>

    <!-- Pod Template Containers -->
    <div v-if="templateContainers.length > 0">
      <h3 class="text-[10px] font-bold text-muted-color uppercase tracking-wider mb-3">
        Pod Template Containers ({{ templateContainers.length }})
      </h3>
      <div class="space-y-3">
        <div
          v-for="c in templateContainers"
          :key="c.name"
          class="bg-(--bg-hover)/30 border border-(--border) rounded-xl p-4 text-xs space-y-3"
        >
          <!-- Container Header -->
          <div class="flex items-center justify-between gap-2 border-b border-(--border)/60 pb-2">
            <div class="flex items-center gap-2 min-w-0">
              <span class="font-bold font-mono text-primary">{{ c.name }}</span>
              <span
                class="text-muted-color font-mono truncate max-w-72 text-[11px]"
                :title="c.image"
              >
                {{ c.image }}
              </span>
            </div>
          </div>

          <!-- Container Specs Grid -->
          <div class="grid grid-cols-1 sm:grid-cols-2 gap-3 text-xs">
            <!-- Ports -->
            <div v-if="c.ports && c.ports.length > 0">
              <span class="text-muted-color block mb-1 text-[11px]">Ports</span>
              <div class="flex flex-wrap gap-1">
                <Tag
                  v-for="(p, pIdx) in c.ports"
                  :key="pIdx"
                  severity="secondary"
                  class="font-mono text-[10px]"
                  :value="`${p.containerPort}/${p.protocol || 'TCP'}${p.name ? ` (${p.name})` : ''}`"
                />
              </div>
            </div>

            <!-- Resources -->
            <div v-if="c.resources && (c.resources.requests || c.resources.limits)">
              <span class="text-muted-color block mb-1 text-[11px]">Resources</span>
              <div class="font-mono text-[11px] space-y-0.5">
                <div v-if="c.resources.requests" class="text-muted-color">
                  Req:
                  <span class="text-primary"
                    >CPU {{ c.resources.requests.cpu || '-' }} / Mem
                    {{ c.resources.requests.memory || '-' }}</span
                  >
                </div>
                <div v-if="c.resources.limits" class="text-muted-color">
                  Lim:
                  <span class="text-primary"
                    >CPU {{ c.resources.limits.cpu || '-' }} / Mem
                    {{ c.resources.limits.memory || '-' }}</span
                  >
                </div>
              </div>
            </div>
          </div>

          <!-- Environment Variables -->
          <div v-if="(c.env && c.env.length > 0) || (c.envFrom && c.envFrom.length > 0)">
            <span class="text-muted-color block mb-1.5 text-[11px]">Environment</span>
            <div class="space-y-1 max-h-40 overflow-y-auto pr-1">
              <div
                v-for="(ef, efIdx) in c.envFrom || []"
                :key="efIdx"
                class="flex items-center justify-between p-1.5 bg-(--bg-hover)/50 rounded text-[11px] font-mono"
              >
                <span class="text-muted-color">envFrom</span>
                <span class="text-primary">
                  {{
                    ef.configMapRef
                      ? `ConfigMap: ${ef.configMapRef.name}`
                      : `Secret: ${ef.secretRef?.name}`
                  }}
                </span>
              </div>
              <div
                v-for="(e, eIdx) in c.env || []"
                :key="e.name || eIdx"
                class="flex items-center justify-between p-1.5 bg-(--bg-hover)/50 rounded text-[11px] font-mono"
              >
                <span class="text-muted-color shrink-0 mr-2">{{ e.name }}</span>
                <span class="text-primary truncate text-right" :title="formatEnvValue(e)">
                  {{ formatEnvValue(e) }}
                </span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Labels & Annotations -->
    <KeyValueBadgeList :items="workloadLabels" title="Labels" variant="tag" />

    <KeyValueBadgeList :items="workloadAnnotations" title="Annotations" variant="list" />
  </div>
</template>
