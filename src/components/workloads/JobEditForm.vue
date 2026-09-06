<script setup lang="ts">
import Button from 'primevue/button'
import InputNumber from 'primevue/inputnumber'
import InputText from 'primevue/inputtext'
import Select from 'primevue/select'
import Tab from 'primevue/tab'
import TabList from 'primevue/tablist'
import TabPanel from 'primevue/tabpanel'
import TabPanels from 'primevue/tabpanels'
import Tabs from 'primevue/tabs'
import ToggleSwitch from 'primevue/toggleswitch'
import { computed, ref, toRaw, watch } from 'vue'

import ContainerEnvEditor from '@/components/shared/ContainerEnvEditor.vue'
import ContainerPortsEditor from '@/components/shared/ContainerPortsEditor.vue'
import ContainerResourcesEditor from '@/components/shared/ContainerResourcesEditor.vue'
import KeyValueEditor from '@/components/shared/KeyValueEditor.vue'
import StringListEditor from '@/components/shared/StringListEditor.vue'
import {
  KUBERNETES_DNS_POLICIES,
  KUBERNETES_DNS_POLICY,
  KUBERNETES_IMAGE_PULL_POLICIES,
  KUBERNETES_JOB_COMPLETION_MODE,
  KUBERNETES_JOB_COMPLETION_MODES,
  KUBERNETES_JOB_RESTART_POLICIES,
  KUBERNETES_JOB_RESTART_POLICY,
  KUBERNETES_POD_REPLACEMENT_POLICIES,
  type KubernetesJobRestartPolicy
} from '@/constants/kubernetes'
import {
  formatContainersToPodSpec,
  normalizeJobSpec,
  parseContainersFromPodSpec,
  validateJobForm,
  type JobContainerFormState
} from '@/utils/job'
import { isValidK8sLabel, isValidK8sName, isValidPath } from '@/utils/validators'

import type { Job } from 'kubernetes-types/batch/v1'

const props = defineProps<{
  rawData: Job | null
}>()

const emit = defineEmits<{
  (e: 'update:rawData', value: Job): void
  (e: 'update:isValid', value: boolean): void
}>()

const activeTab = ref('general')

// General / Job execution state
const completions = ref<number | null>(1)
const parallelism = ref<number>(1)
const backoffLimit = ref<number>(6)
const activeDeadlineSeconds = ref<number | null>(null)
const ttlSecondsAfterFinished = ref<number | null>(null)
const suspend = ref<boolean>(false)
const completionMode = ref<string>(KUBERNETES_JOB_COMPLETION_MODE.NonIndexed)
const backoffLimitPerIndex = ref<number | null>(null)
const maxFailedIndexes = ref<number | null>(null)
const podReplacementPolicy = ref<string>('')
const selectorLabels = ref<{ key: string; value: string }[]>([])

// Metadata state
const jobLabels = ref<{ key: string; value: string }[]>([])
const jobAnnotations = ref<{ key: string; value: string }[]>([])
const podLabels = ref<{ key: string; value: string }[]>([])
const podAnnotations = ref<{ key: string; value: string }[]>([])

// Pod spec state
const serviceAccountName = ref<string>('')
const restartPolicy = ref<KubernetesJobRestartPolicy>(KUBERNETES_JOB_RESTART_POLICY.Never)
const terminationGracePeriodSeconds = ref<number>(30)
const nodeSelector = ref<{ key: string; value: string }[]>([])
const hostNetwork = ref<boolean>(false)
const hostPID = ref<boolean>(false)
const hostIPC = ref<boolean>(false)
const dnsPolicy = ref<string>(KUBERNETES_DNS_POLICY.ClusterFirst)

const jobNamespace = computed(() => {
  return props.rawData?.metadata?.namespace ?? ''
})

const containers = ref<JobContainerFormState[]>([])
const activeContainerIndex = ref<number>(0)

const currentContainer = computed(() => {
  return containers.value[activeContainerIndex.value] || null
})

const isFormValid = computed(() => {
  return validateJobForm({
    parallelism: parallelism.value,
    completions: completions.value === null ? undefined : completions.value,
    completionMode: completionMode.value,
    backoffLimit: backoffLimit.value,
    activeDeadlineSeconds:
      activeDeadlineSeconds.value === null ? undefined : activeDeadlineSeconds.value,
    ttlSecondsAfterFinished:
      ttlSecondsAfterFinished.value === null ? undefined : ttlSecondsAfterFinished.value,
    backoffLimitPerIndex:
      backoffLimitPerIndex.value === null ? undefined : backoffLimitPerIndex.value,
    maxFailedIndexes: maxFailedIndexes.value === null ? undefined : maxFailedIndexes.value,
    serviceAccountName: serviceAccountName.value,
    containers: containers.value
  })
})

watch(
  isFormValid,
  (val) => {
    emit('update:isValid', val)
  },
  { immediate: true }
)

let isEmitting = false

const kvObjectToArray = (
  obj: Record<string, string> | undefined
): { key: string; value: string }[] => {
  if (!obj || typeof obj !== 'object') return []
  return Object.entries(obj).map(([key, value]) => ({ key, value: String(value ?? '') }))
}

const kvArrayToObject = (arr: { key: string; value: string }[]): Record<string, string> => {
  const res: Record<string, string> = {}
  for (const item of arr) {
    if (item.key.trim()) {
      res[item.key.trim()] = item.value
    }
  }
  return res
}

const syncFromRawData = (data: Job | null) => {
  if (!data || isEmitting) return

  const normalized = normalizeJobSpec(data)
  parallelism.value = normalized.parallelism
  completions.value = normalized.completions
  backoffLimit.value = normalized.backoffLimit
  activeDeadlineSeconds.value = normalized.activeDeadlineSeconds
  ttlSecondsAfterFinished.value = normalized.ttlSecondsAfterFinished
  suspend.value = normalized.suspend
  completionMode.value = normalized.completionMode
  backoffLimitPerIndex.value = normalized.backoffLimitPerIndex
  maxFailedIndexes.value = normalized.maxFailedIndexes
  podReplacementPolicy.value = normalized.podReplacementPolicy
  selectorLabels.value = normalized.selectorLabels

  // Metadata
  jobLabels.value = kvObjectToArray(data.metadata?.labels)
  jobAnnotations.value = kvObjectToArray(data.metadata?.annotations)
  podLabels.value = kvObjectToArray(data.spec?.template?.metadata?.labels)
  podAnnotations.value = kvObjectToArray(data.spec?.template?.metadata?.annotations)

  // Pod template spec
  const templateSpec = data.spec?.template?.spec
  serviceAccountName.value = templateSpec?.serviceAccountName || ''
  if (
    templateSpec?.restartPolicy === KUBERNETES_JOB_RESTART_POLICY.OnFailure ||
    templateSpec?.restartPolicy === KUBERNETES_JOB_RESTART_POLICY.Never
  ) {
    restartPolicy.value = templateSpec.restartPolicy
  } else {
    restartPolicy.value = KUBERNETES_JOB_RESTART_POLICY.Never
  }
  terminationGracePeriodSeconds.value =
    typeof templateSpec?.terminationGracePeriodSeconds === 'number'
      ? templateSpec.terminationGracePeriodSeconds
      : 30
  nodeSelector.value = kvObjectToArray(templateSpec?.nodeSelector)
  hostNetwork.value = Boolean(templateSpec?.hostNetwork)
  hostPID.value = Boolean(templateSpec?.hostPID)
  hostIPC.value = Boolean(templateSpec?.hostIPC)
  dnsPolicy.value = templateSpec?.dnsPolicy || KUBERNETES_DNS_POLICY.ClusterFirst

  // Containers
  containers.value = parseContainersFromPodSpec(templateSpec?.containers)
  if (activeContainerIndex.value >= containers.value.length) {
    activeContainerIndex.value = Math.max(0, containers.value.length - 1)
  }
}

watch(
  () => props.rawData,
  (newData) => {
    syncFromRawData(newData)
  },
  { immediate: true, deep: true }
)

const handleFieldChange = () => {
  if (!props.rawData) return
  isEmitting = true

  const rawObj = JSON.parse(JSON.stringify(toRaw(props.rawData))) as Job
  if (!rawObj.metadata) rawObj.metadata = {}
  if (!rawObj.spec) {
    rawObj.spec = {
      template: { spec: { containers: [] } }
    }
  }
  const spec = rawObj.spec
  if (!spec.template) spec.template = { spec: { containers: [] } }
  if (!spec.template.metadata) spec.template.metadata = {}
  if (!spec.template.spec) spec.template.spec = { containers: [] }

  // 1. General & Execution
  spec.parallelism = typeof parallelism.value === 'number' ? parallelism.value : 1
  if (completions.value !== null && completions.value !== undefined) {
    spec.completions = completions.value
  } else {
    delete spec.completions
  }
  spec.backoffLimit = typeof backoffLimit.value === 'number' ? backoffLimit.value : 6

  if (activeDeadlineSeconds.value !== null && activeDeadlineSeconds.value !== undefined) {
    spec.activeDeadlineSeconds = activeDeadlineSeconds.value
  } else {
    delete spec.activeDeadlineSeconds
  }

  if (ttlSecondsAfterFinished.value !== null && ttlSecondsAfterFinished.value !== undefined) {
    spec.ttlSecondsAfterFinished = ttlSecondsAfterFinished.value
  } else {
    delete spec.ttlSecondsAfterFinished
  }

  if (suspend.value) {
    spec.suspend = true
  } else {
    delete spec.suspend
  }

  if (completionMode.value) {
    spec.completionMode = completionMode.value
  } else {
    delete spec.completionMode
  }

  if (
    completionMode.value === KUBERNETES_JOB_COMPLETION_MODE.Indexed &&
    backoffLimitPerIndex.value !== null
  ) {
    spec.backoffLimitPerIndex = backoffLimitPerIndex.value
  } else {
    delete spec.backoffLimitPerIndex
  }

  if (
    completionMode.value === KUBERNETES_JOB_COMPLETION_MODE.Indexed &&
    maxFailedIndexes.value !== null
  ) {
    spec.maxFailedIndexes = maxFailedIndexes.value
  } else {
    delete spec.maxFailedIndexes
  }

  if (podReplacementPolicy.value) {
    spec.podReplacementPolicy = podReplacementPolicy.value
  } else {
    delete spec.podReplacementPolicy
  }

  // 2. Metadata
  rawObj.metadata.labels = kvArrayToObject(jobLabels.value)
  rawObj.metadata.annotations = kvArrayToObject(jobAnnotations.value)
  spec.template.metadata.labels = kvArrayToObject(podLabels.value)
  spec.template.metadata.annotations = kvArrayToObject(podAnnotations.value)

  // 3. Pod Spec
  if (serviceAccountName.value.trim()) {
    spec.template.spec.serviceAccountName = serviceAccountName.value.trim()
  } else {
    delete spec.template.spec.serviceAccountName
  }

  spec.template.spec.restartPolicy = restartPolicy.value
  spec.template.spec.terminationGracePeriodSeconds = terminationGracePeriodSeconds.value

  const nodeSel = kvArrayToObject(nodeSelector.value)
  if (Object.keys(nodeSel).length > 0) {
    spec.template.spec.nodeSelector = nodeSel
  } else {
    delete spec.template.spec.nodeSelector
  }

  if (hostNetwork.value) {
    spec.template.spec.hostNetwork = true
  } else {
    delete spec.template.spec.hostNetwork
  }

  if (hostPID.value) {
    spec.template.spec.hostPID = true
  } else {
    delete spec.template.spec.hostPID
  }

  if (hostIPC.value) {
    spec.template.spec.hostIPC = true
  } else {
    delete spec.template.spec.hostIPC
  }

  if (dnsPolicy.value) {
    spec.template.spec.dnsPolicy = dnsPolicy.value
  } else {
    delete spec.template.spec.dnsPolicy
  }

  // 4. Containers
  spec.template.spec.containers = formatContainersToPodSpec(containers.value)

  emit('update:rawData', rawObj)

  setTimeout(() => {
    isEmitting = false
  }, 50)
}
</script>

<template>
  <div class="w-full h-full flex flex-col overflow-hidden">
    <Tabs v-model:value="activeTab" class="w-full flex flex-col h-full">
      <TabList>
        <Tab value="general" class="text-xs font-medium">General</Tab>
        <Tab value="metadata" class="text-xs font-medium">Metadata</Tab>
        <Tab value="pod" class="text-xs font-medium">Pod Spec</Tab>
        <Tab value="containers" class="text-xs font-medium">Containers</Tab>
      </TabList>

      <TabPanels class="flex-1 overflow-y-auto pt-6 px-0">
        <!-- GENERAL TAB -->
        <TabPanel value="general" class="flex flex-col gap-10 max-w-7xl">
          <!-- Section 1: Execution & Concurrency -->
          <div class="grid grid-cols-1 md:grid-cols-12 gap-6">
            <div class="md:col-span-4 flex flex-col gap-1">
              <span class="text-xs font-semibold tracking-wider text-muted-color uppercase">
                Job Execution
              </span>
              <p class="text-xs text-muted-color leading-relaxed">
                Controls completion target, parallelism, and retry limits for pod executions.
              </p>
            </div>
            <div class="md:col-span-8 flex flex-col gap-4">
              <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
                <div class="flex flex-col gap-1.5">
                  <label class="text-xs font-medium text-muted-color">Completions</label>
                  <InputNumber
                    v-model="completions"
                    :min="1"
                    size="small"
                    fluid
                    placeholder="Leave empty for work queues"
                    @update:model-value="handleFieldChange"
                  />
                  <span class="text-[11px] text-muted-color">
                    Desired count of successfully completed pods.
                  </span>
                </div>

                <div class="flex flex-col gap-1.5">
                  <label class="text-xs font-medium text-muted-color">Parallelism</label>
                  <InputNumber
                    v-model="parallelism"
                    :min="0"
                    size="small"
                    fluid
                    @update:model-value="handleFieldChange"
                  />
                  <span class="text-[11px] text-muted-color">
                    Maximum number of pods running concurrently.
                  </span>
                </div>
              </div>

              <div class="flex flex-col gap-1.5">
                <label class="text-xs font-medium text-muted-color">Backoff Limit</label>
                <InputNumber
                  v-model="backoffLimit"
                  :min="0"
                  size="small"
                  fluid
                  @update:model-value="handleFieldChange"
                />
                <span class="text-[11px] text-muted-color">
                  Number of retries before marking the job as failed (defaults to 6).
                </span>
              </div>
            </div>
          </div>

          <!-- Section 2: Lifecycle & Termination -->
          <div class="grid grid-cols-1 md:grid-cols-12 gap-6">
            <div class="md:col-span-4 flex flex-col gap-1">
              <span class="text-xs font-semibold tracking-wider text-muted-color uppercase">
                Lifecycle & Termination
              </span>
              <p class="text-xs text-muted-color leading-relaxed">
                Execution time limits, post-execution cleanup, and suspension status.
              </p>
            </div>
            <div class="md:col-span-8 flex flex-col gap-4">
              <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
                <div class="flex flex-col gap-1.5">
                  <label class="text-xs font-medium text-muted-color">
                    Active Deadline Seconds
                  </label>
                  <InputNumber
                    v-model="activeDeadlineSeconds"
                    :min="1"
                    size="small"
                    fluid
                    placeholder="Unlimited"
                    @update:model-value="handleFieldChange"
                  />
                  <span class="text-[11px] text-muted-color">
                    Maximum duration in seconds the Job may run before termination.
                  </span>
                </div>

                <div class="flex flex-col gap-1.5">
                  <label class="text-xs font-medium text-muted-color">
                    TTL Seconds After Finished
                  </label>
                  <InputNumber
                    v-model="ttlSecondsAfterFinished"
                    :min="0"
                    size="small"
                    fluid
                    placeholder="Disabled"
                    @update:model-value="handleFieldChange"
                  />
                  <span class="text-[11px] text-muted-color">
                    Lifetime in seconds before completed/failed job is deleted automatically.
                  </span>
                </div>
              </div>

              <div class="flex items-center justify-between p-4 rounded-lg bg-(--bg-hover)/30">
                <div class="flex flex-col gap-0.5">
                  <span class="text-xs font-medium text-primary">Suspend Job</span>
                  <span class="text-[11px] text-muted-color">
                    Pause execution. Suspending an active Job terminates all running pods.
                  </span>
                </div>
                <ToggleSwitch v-model="suspend" @change="handleFieldChange" />
              </div>
            </div>
          </div>

          <!-- Section 3: Advanced Job Settings -->
          <div class="grid grid-cols-1 md:grid-cols-12 gap-6">
            <div class="md:col-span-4 flex flex-col gap-1">
              <span class="text-xs font-semibold tracking-wider text-muted-color uppercase">
                Advanced Job Settings
              </span>
              <p class="text-xs text-muted-color leading-relaxed">
                Indexed tracking and pod replacement behavior.
              </p>
            </div>
            <div class="md:col-span-8 flex flex-col gap-4">
              <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
                <div class="flex flex-col gap-1.5">
                  <label class="text-xs font-medium text-muted-color">Completion Mode</label>
                  <Select
                    v-model="completionMode"
                    :options="KUBERNETES_JOB_COMPLETION_MODES"
                    size="small"
                    fluid
                    class="text-xs"
                    @change="handleFieldChange"
                  />
                  <span class="text-[11px] text-muted-color">
                    NonIndexed or Indexed completion tracking mode.
                  </span>
                </div>

                <div class="flex flex-col gap-1.5">
                  <label class="text-xs font-medium text-muted-color">
                    Pod Replacement Policy
                  </label>
                  <Select
                    v-model="podReplacementPolicy"
                    :options="KUBERNETES_POD_REPLACEMENT_POLICIES"
                    showClear
                    placeholder="Default"
                    size="small"
                    fluid
                    class="text-xs"
                    @change="handleFieldChange"
                  />
                  <span class="text-[11px] text-muted-color">
                    Determines when replacement pods are created.
                  </span>
                </div>
              </div>

              <div
                v-if="completionMode === KUBERNETES_JOB_COMPLETION_MODE.Indexed"
                class="grid grid-cols-1 sm:grid-cols-2 gap-4 p-4 rounded-lg bg-(--bg-hover)/20"
              >
                <div class="flex flex-col gap-1.5">
                  <label class="text-xs font-medium text-muted-color">
                    Backoff Limit Per Index
                  </label>
                  <InputNumber
                    v-model="backoffLimitPerIndex"
                    :min="0"
                    size="small"
                    fluid
                    placeholder="None"
                    @update:model-value="handleFieldChange"
                  />
                  <span class="text-[11px] text-muted-color">
                    Maximum retries allowed per completion index.
                  </span>
                </div>

                <div class="flex flex-col gap-1.5">
                  <label class="text-xs font-medium text-muted-color">Max Failed Indexes</label>
                  <InputNumber
                    v-model="maxFailedIndexes"
                    :min="0"
                    size="small"
                    fluid
                    placeholder="None"
                    @update:model-value="handleFieldChange"
                  />
                  <span class="text-[11px] text-muted-color">
                    Max failed indexes before terminating entire Job.
                  </span>
                </div>
              </div>
            </div>
          </div>

          <!-- Section 4: Selector Labels (Read-only) -->
          <div class="grid grid-cols-1 md:grid-cols-12 gap-6">
            <div class="md:col-span-4 flex flex-col gap-1">
              <span class="text-xs font-semibold tracking-wider text-muted-color uppercase">
                Selector Labels
              </span>
              <p class="text-xs text-muted-color leading-relaxed">
                Pod selector labels generated by the system. Immutable after Job creation.
              </p>
            </div>
            <div class="md:col-span-8 flex flex-col gap-2">
              <div v-if="selectorLabels.length === 0" class="text-xs text-muted-color">
                None defined
              </div>
              <div class="flex flex-wrap gap-2">
                <span
                  v-for="(s, idx) in selectorLabels"
                  :key="'sel-' + idx"
                  class="px-2.5 py-1 bg-(--bg-hover)/60 rounded text-xs text-muted-color font-mono"
                >
                  {{ s.key }}={{ s.value }}
                </span>
              </div>
            </div>
          </div>
        </TabPanel>

        <!-- METADATA TAB -->
        <TabPanel value="metadata" class="flex flex-col gap-10 max-w-7xl">
          <!-- Job Level Metadata -->
          <div class="grid grid-cols-1 md:grid-cols-12 gap-6">
            <div class="md:col-span-4 flex flex-col gap-1">
              <span class="text-xs font-semibold tracking-wider text-muted-color uppercase">
                Job Metadata
              </span>
              <p class="text-xs text-muted-color leading-relaxed">
                Labels and annotations attached to the Job resource itself.
              </p>
            </div>
            <div class="md:col-span-8 flex flex-col gap-6">
              <div class="p-4 rounded-lg bg-(--bg-hover)/30 flex flex-col gap-3">
                <KeyValueEditor
                  v-model="jobLabels"
                  title="Job Labels"
                  add-label="Add Label"
                  @update:model-value="handleFieldChange"
                />
              </div>
              <div class="p-4 rounded-lg bg-(--bg-hover)/30 flex flex-col gap-3">
                <KeyValueEditor
                  v-model="jobAnnotations"
                  title="Job Annotations"
                  add-label="Add Annotation"
                  @update:model-value="handleFieldChange"
                />
              </div>
            </div>
          </div>

          <!-- Pod Template Metadata -->
          <div class="grid grid-cols-1 md:grid-cols-12 gap-6">
            <div class="md:col-span-4 flex flex-col gap-1">
              <span class="text-xs font-semibold tracking-wider text-muted-color uppercase">
                Pod Template Metadata
              </span>
              <p class="text-xs text-muted-color leading-relaxed">
                Labels and annotations applied to pods spawned by this Job.
              </p>
            </div>
            <div class="md:col-span-8 flex flex-col gap-6">
              <div class="p-4 rounded-lg bg-(--bg-hover)/30 flex flex-col gap-3">
                <KeyValueEditor
                  v-model="podLabels"
                  title="Pod Labels"
                  add-label="Add Pod Label"
                  @update:model-value="handleFieldChange"
                />
              </div>
              <div class="p-4 rounded-lg bg-(--bg-hover)/30 flex flex-col gap-3">
                <KeyValueEditor
                  v-model="podAnnotations"
                  title="Pod Annotations"
                  add-label="Add Pod Annotation"
                  @update:model-value="handleFieldChange"
                />
              </div>
            </div>
          </div>
        </TabPanel>

        <!-- POD SPEC TAB -->
        <TabPanel value="pod" class="flex flex-col gap-10 max-w-7xl">
          <!-- Execution & Lifecycle -->
          <div class="grid grid-cols-1 md:grid-cols-12 gap-6">
            <div class="md:col-span-4 flex flex-col gap-1">
              <span class="text-xs font-semibold tracking-wider text-muted-color uppercase">
                Pod Execution
              </span>
              <p class="text-xs text-muted-color leading-relaxed">
                Core identity, restart policies, and grace periods for Job pods. Pod template
                specifications are immutable after Job creation.
              </p>
            </div>
            <div class="md:col-span-8 flex flex-col gap-4">
              <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
                <div class="flex flex-col gap-1.5">
                  <label class="text-xs font-medium text-muted-color">Service Account</label>
                  <InputText
                    v-model="serviceAccountName"
                    placeholder="e.g. default"
                    :invalid="Boolean(serviceAccountName && !isValidK8sName(serviceAccountName))"
                    size="small"
                    fluid
                    class="text-xs"
                    @input="handleFieldChange"
                  />
                </div>

                <div class="flex flex-col gap-1.5">
                  <label class="text-xs font-medium text-muted-color">Restart Policy</label>
                  <Select
                    v-model="restartPolicy"
                    :options="KUBERNETES_JOB_RESTART_POLICIES"
                    size="small"
                    fluid
                    class="text-xs"
                    @change="handleFieldChange"
                  />
                  <span class="text-[11px] text-muted-color">
                    Job pods require Never or OnFailure (Always is disallowed).
                  </span>
                </div>
              </div>

              <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
                <div class="flex flex-col gap-1.5">
                  <label class="text-xs font-medium text-muted-color">
                    Termination Grace Period (s)
                  </label>
                  <InputNumber
                    v-model="terminationGracePeriodSeconds"
                    :min="0"
                    size="small"
                    fluid
                    class="text-xs"
                    @update:model-value="handleFieldChange"
                  />
                </div>

                <div class="flex flex-col gap-1.5">
                  <label class="text-xs font-medium text-muted-color">DNS Policy</label>
                  <Select
                    v-model="dnsPolicy"
                    :options="KUBERNETES_DNS_POLICIES"
                    size="small"
                    fluid
                    class="text-xs"
                    @change="handleFieldChange"
                  />
                </div>
              </div>
            </div>
          </div>

          <!-- Advanced Networking & Process -->
          <div class="grid grid-cols-1 md:grid-cols-12 gap-6">
            <div class="md:col-span-4 flex flex-col gap-1">
              <span class="text-xs font-semibold tracking-wider text-muted-color uppercase">
                Host Namespaces
              </span>
              <p class="text-xs text-muted-color leading-relaxed">
                Control host namespace sharing for networking, processes, and IPC.
              </p>
            </div>
            <div class="md:col-span-8 flex flex-col gap-3">
              <div
                class="flex items-center justify-between p-3.5 rounded-lg bg-(--bg-hover)/30 text-xs"
              >
                <div>
                  <div class="font-medium text-primary">Host Network</div>
                  <div class="text-muted-color text-[11px]">
                    Use the node's network namespace directly
                  </div>
                </div>
                <ToggleSwitch v-model="hostNetwork" @change="handleFieldChange" />
              </div>

              <div
                class="flex items-center justify-between p-3.5 rounded-lg bg-(--bg-hover)/30 text-xs"
              >
                <div>
                  <div class="font-medium text-primary">Host PID</div>
                  <div class="text-muted-color text-[11px]">
                    Use the host's process ID namespace
                  </div>
                </div>
                <ToggleSwitch v-model="hostPID" @change="handleFieldChange" />
              </div>

              <div
                class="flex items-center justify-between p-3.5 rounded-lg bg-(--bg-hover)/30 text-xs"
              >
                <div>
                  <div class="font-medium text-primary">Host IPC</div>
                  <div class="text-muted-color text-[11px]">
                    Use the host's inter-process communication namespace
                  </div>
                </div>
                <ToggleSwitch v-model="hostIPC" @change="handleFieldChange" />
              </div>
            </div>
          </div>

          <!-- Scheduling Constraints -->
          <div class="grid grid-cols-1 md:grid-cols-12 gap-6">
            <div class="md:col-span-4 flex flex-col gap-1">
              <span class="text-xs font-semibold tracking-wider text-muted-color uppercase">
                Scheduling Constraints
              </span>
              <p class="text-xs text-muted-color leading-relaxed">
                Node selector labels restricting which nodes this Job runs on.
              </p>
            </div>
            <div class="md:col-span-8 flex flex-col gap-3">
              <div class="p-4 rounded-lg bg-(--bg-hover)/30 flex flex-col gap-3">
                <KeyValueEditor
                  v-model="nodeSelector"
                  title="Node Selector Labels"
                  add-label="Add Node Selector"
                  @update:model-value="handleFieldChange"
                />
              </div>
            </div>
          </div>
        </TabPanel>

        <!-- CONTAINERS TAB -->
        <TabPanel value="containers" class="flex flex-col gap-10 max-w-7xl">
          <!-- Container selector if multiple -->
          <div v-if="containers.length > 1" class="flex items-center gap-2">
            <span class="text-xs font-medium text-muted-color">Container:</span>
            <Button
              v-for="(c, idx) in containers"
              :key="'cbtn-' + idx"
              :label="c.name || `Container ${idx + 1}`"
              size="small"
              :variant="activeContainerIndex === idx ? undefined : 'text'"
              :severity="activeContainerIndex === idx ? 'primary' : 'secondary'"
              class="text-xs cursor-pointer"
              @click="activeContainerIndex = idx"
            />
          </div>

          <div v-if="currentContainer" class="flex flex-col gap-10">
            <!-- Basic Info -->
            <div class="grid grid-cols-1 md:grid-cols-12 gap-6">
              <div class="md:col-span-4 flex flex-col gap-1">
                <span class="text-xs font-semibold tracking-wider text-muted-color uppercase">
                  Container Image
                </span>
                <p class="text-xs text-muted-color leading-relaxed">
                  Basic container identifier, image reference, and image pull policy. Pod template
                  specifications are immutable after Job creation.
                </p>
              </div>
              <div class="md:col-span-8 flex flex-col gap-4">
                <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
                  <div class="flex flex-col gap-1.5">
                    <label class="text-xs font-medium text-muted-color">Container Name</label>
                    <InputText
                      v-model="currentContainer.name"
                      :invalid="
                        Boolean(
                          !currentContainer.name?.trim() ||
                          !isValidK8sLabel(currentContainer.name.trim())
                        )
                      "
                      size="small"
                      fluid
                      class="text-xs"
                      @input="handleFieldChange"
                    />
                  </div>

                  <div class="flex flex-col gap-1.5">
                    <label class="text-xs font-medium text-muted-color">Image Pull Policy</label>
                    <Select
                      v-model="currentContainer.imagePullPolicy"
                      :options="KUBERNETES_IMAGE_PULL_POLICIES"
                      size="small"
                      fluid
                      class="text-xs"
                      @change="handleFieldChange"
                    />
                  </div>
                </div>

                <div class="flex flex-col gap-1.5">
                  <label class="text-xs font-medium text-muted-color">Image</label>
                  <InputText
                    v-model="currentContainer.image"
                    placeholder="e.g. busybox:latest"
                    size="small"
                    fluid
                    class="text-xs"
                    @input="handleFieldChange"
                  />
                </div>

                <div class="flex flex-col gap-1.5">
                  <label class="text-xs font-medium text-muted-color">Working Directory</label>
                  <InputText
                    v-model="currentContainer.workingDir"
                    placeholder="e.g. /workspace"
                    :invalid="
                      Boolean(
                        currentContainer.workingDir && !isValidPath(currentContainer.workingDir)
                      )
                    "
                    size="small"
                    fluid
                    class="text-xs"
                    @input="handleFieldChange"
                  />
                </div>
              </div>
            </div>

            <!-- Compute Resources -->
            <div class="grid grid-cols-1 md:grid-cols-12 gap-6">
              <div class="md:col-span-4 flex flex-col gap-1">
                <span class="text-xs font-semibold tracking-wider text-muted-color uppercase">
                  Compute Resources
                </span>
                <p class="text-xs text-muted-color leading-relaxed">
                  Compute requests (guaranteed) and limits (maximum cap) for CPU and Memory.
                </p>
              </div>
              <div class="md:col-span-8 flex flex-col gap-3">
                <ContainerResourcesEditor
                  v-model:cpu-request="currentContainer.cpuRequest"
                  v-model:memory-request="currentContainer.memoryRequest"
                  v-model:cpu-limit="currentContainer.cpuLimit"
                  v-model:memory-limit="currentContainer.memoryLimit"
                  @update:cpu-request="handleFieldChange"
                  @update:memory-request="handleFieldChange"
                  @update:cpu-limit="handleFieldChange"
                  @update:memory-limit="handleFieldChange"
                />
              </div>
            </div>

            <!-- Execution Commands -->
            <div class="grid grid-cols-1 md:grid-cols-12 gap-6">
              <div class="md:col-span-4 flex flex-col gap-1">
                <span class="text-xs font-semibold tracking-wider text-muted-color uppercase">
                  Execution Commands
                </span>
                <p class="text-xs text-muted-color leading-relaxed">
                  Entrypoint commands and arguments executed by the container runtime.
                </p>
              </div>
              <div class="md:col-span-8 flex flex-col gap-4">
                <div class="p-4 rounded-lg bg-(--bg-hover)/30 flex flex-col gap-3">
                  <StringListEditor
                    v-model="currentContainer.command"
                    title="Commands"
                    placeholder="e.g. /bin/sh"
                    add-label="Add Command"
                    @update:model-value="handleFieldChange"
                  />
                </div>
                <div class="p-4 rounded-lg bg-(--bg-hover)/30 flex flex-col gap-3">
                  <StringListEditor
                    v-model="currentContainer.args"
                    title="Arguments"
                    placeholder="e.g. -c"
                    add-label="Add Argument"
                    @update:model-value="handleFieldChange"
                  />
                </div>
              </div>
            </div>

            <!-- Environment Variables -->
            <div class="grid grid-cols-1 md:grid-cols-12 gap-6">
              <div class="md:col-span-4 flex flex-col gap-1">
                <span class="text-xs font-semibold tracking-wider text-muted-color uppercase">
                  Environment
                </span>
                <p class="text-xs text-muted-color leading-relaxed">
                  Environment variables passed directly into the container process.
                </p>
              </div>
              <div class="md:col-span-8 flex flex-col gap-3">
                <div class="p-4 rounded-lg bg-(--bg-hover)/30 flex flex-col gap-3">
                  <ContainerEnvEditor
                    v-model="currentContainer.environments"
                    :namespace="jobNamespace"
                    @update:model-value="handleFieldChange"
                  />
                </div>
              </div>
            </div>

            <!-- Container Ports -->
            <div class="grid grid-cols-1 md:grid-cols-12 gap-6">
              <div class="md:col-span-4 flex flex-col gap-1">
                <span class="text-xs font-semibold tracking-wider text-muted-color uppercase">
                  Networking & Ports
                </span>
                <p class="text-xs text-muted-color leading-relaxed">
                  Network ports exposed by container processes for incoming cluster traffic.
                </p>
              </div>
              <div class="md:col-span-8 flex flex-col gap-3">
                <div class="p-4 rounded-lg bg-(--bg-hover)/30">
                  <ContainerPortsEditor
                    v-model="currentContainer.ports"
                    @update:model-value="handleFieldChange"
                  />
                </div>
              </div>
            </div>
          </div>
        </TabPanel>
      </TabPanels>
    </Tabs>
  </div>
</template>
