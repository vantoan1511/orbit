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
  KUBERNETES_JOB_RESTART_POLICIES,
  KUBERNETES_JOB_RESTART_POLICY,
  type KubernetesJobRestartPolicy
} from '@/constants/kubernetes'
import {
  KUBERNETES_CONCURRENCY_POLICIES,
  formatContainersToPodSpec,
  normalizeCronJobSpec,
  parseContainersFromPodSpec,
  validateCronJobForm,
  type CronJobContainerFormState
} from '@/utils/cronJob'
import { isValidCron, isValidK8sLabel, isValidPath } from '@/utils/validators'

import type { CronJob } from 'kubernetes-types/batch/v1'

const props = defineProps<{
  rawData: CronJob | null
}>()

const emit = defineEmits<{
  (e: 'update:rawData', value: CronJob): void
  (e: 'update:isValid', value: boolean): void
}>()

const activeTab = ref('general')

// General / Schedule & Concurrency state
const schedule = ref<string>('*/5 * * * *')
const concurrencyPolicy = ref<string>('Allow')
const suspend = ref<boolean>(false)
const startingDeadlineSeconds = ref<number | null>(null)

// History limits state
const successfulJobsHistoryLimit = ref<number>(3)
const failedJobsHistoryLimit = ref<number>(1)

// Job template execution state
const completions = ref<number | null>(null)
const parallelism = ref<number>(1)
const backoffLimit = ref<number>(6)
const activeDeadlineSeconds = ref<number | null>(null)
const ttlSecondsAfterFinished = ref<number | null>(null)

// Metadata state
const cronJobLabels = ref<{ key: string; value: string }[]>([])
const cronJobAnnotations = ref<{ key: string; value: string }[]>([])
const podLabels = ref<{ key: string; value: string }[]>([])
const podAnnotations = ref<{ key: string; value: string }[]>([])

// Pod spec state
const serviceAccountName = ref<string>('')
const restartPolicy = ref<KubernetesJobRestartPolicy>(KUBERNETES_JOB_RESTART_POLICY.OnFailure)
const terminationGracePeriodSeconds = ref<number>(30)
const nodeSelector = ref<{ key: string; value: string }[]>([])
const hostNetwork = ref<boolean>(false)
const hostPID = ref<boolean>(false)
const hostIPC = ref<boolean>(false)
const dnsPolicy = ref<string>(KUBERNETES_DNS_POLICY.ClusterFirst)

const cronJobNamespace = computed(() => {
  return props.rawData?.metadata?.namespace ?? ''
})

const containers = ref<CronJobContainerFormState[]>([])
const activeContainerIndex = ref<number>(0)

const currentContainer = computed(() => {
  return containers.value[activeContainerIndex.value] || null
})

const isScheduleValid = computed(() => isValidCron(schedule.value))

const isFormValid = computed(() => {
  return validateCronJobForm({
    schedule: schedule.value,
    concurrencyPolicy: concurrencyPolicy.value,
    startingDeadlineSeconds: startingDeadlineSeconds.value,
    successfulJobsHistoryLimit: successfulJobsHistoryLimit.value,
    failedJobsHistoryLimit: failedJobsHistoryLimit.value,
    parallelism: parallelism.value,
    completions: completions.value,
    backoffLimit: backoffLimit.value,
    activeDeadlineSeconds: activeDeadlineSeconds.value,
    ttlSecondsAfterFinished: ttlSecondsAfterFinished.value,
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

const syncFromRawData = (data: CronJob | null) => {
  if (!data || isEmitting) return

  const normalized = normalizeCronJobSpec(data)
  schedule.value = normalized.schedule
  concurrencyPolicy.value = normalized.concurrencyPolicy
  suspend.value = normalized.suspend
  startingDeadlineSeconds.value = normalized.startingDeadlineSeconds
  successfulJobsHistoryLimit.value = normalized.successfulJobsHistoryLimit
  failedJobsHistoryLimit.value = normalized.failedJobsHistoryLimit
  parallelism.value = normalized.parallelism
  completions.value = normalized.completions
  backoffLimit.value = normalized.backoffLimit
  activeDeadlineSeconds.value = normalized.activeDeadlineSeconds
  ttlSecondsAfterFinished.value = normalized.ttlSecondsAfterFinished

  // Metadata
  cronJobLabels.value = kvObjectToArray(data.metadata?.labels)
  cronJobAnnotations.value = kvObjectToArray(data.metadata?.annotations)

  const jobTemplate = data.spec?.jobTemplate
  podLabels.value = kvObjectToArray(jobTemplate?.spec?.template?.metadata?.labels)
  podAnnotations.value = kvObjectToArray(jobTemplate?.spec?.template?.metadata?.annotations)

  // Pod template spec
  const templateSpec = jobTemplate?.spec?.template?.spec
  serviceAccountName.value = templateSpec?.serviceAccountName || ''
  if (
    templateSpec?.restartPolicy === KUBERNETES_JOB_RESTART_POLICY.OnFailure ||
    templateSpec?.restartPolicy === KUBERNETES_JOB_RESTART_POLICY.Never
  ) {
    restartPolicy.value = templateSpec.restartPolicy
  } else {
    restartPolicy.value = KUBERNETES_JOB_RESTART_POLICY.OnFailure
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

  const rawObj = JSON.parse(JSON.stringify(toRaw(props.rawData))) as CronJob
  if (!rawObj.metadata) rawObj.metadata = {}
  if (!rawObj.spec) {
    rawObj.spec = {
      schedule: '*/5 * * * *',
      jobTemplate: {
        spec: {
          template: { spec: { containers: [] } }
        }
      }
    }
  }

  const spec = rawObj.spec
  if (!spec.jobTemplate) {
    spec.jobTemplate = {
      spec: {
        template: { spec: { containers: [] } }
      }
    }
  }
  if (!spec.jobTemplate.spec) {
    spec.jobTemplate.spec = {
      template: { spec: { containers: [] } }
    }
  }
  const jobSpec = spec.jobTemplate.spec
  if (!jobSpec.template) {
    jobSpec.template = { spec: { containers: [] } }
  }
  if (!jobSpec.template.metadata) jobSpec.template.metadata = {}
  if (!jobSpec.template.spec) jobSpec.template.spec = { containers: [] }

  // 1. CronJob General & Schedule
  spec.schedule = schedule.value.trim()
  spec.concurrencyPolicy = concurrencyPolicy.value
  spec.suspend = suspend.value
  if (startingDeadlineSeconds.value !== null && startingDeadlineSeconds.value !== undefined) {
    spec.startingDeadlineSeconds = startingDeadlineSeconds.value
  } else {
    delete spec.startingDeadlineSeconds
  }

  // History limits
  spec.successfulJobsHistoryLimit =
    typeof successfulJobsHistoryLimit.value === 'number' ? successfulJobsHistoryLimit.value : 3
  spec.failedJobsHistoryLimit =
    typeof failedJobsHistoryLimit.value === 'number' ? failedJobsHistoryLimit.value : 1

  // Job execution parameters
  jobSpec.parallelism = typeof parallelism.value === 'number' ? parallelism.value : 1
  if (completions.value !== null && completions.value !== undefined) {
    jobSpec.completions = completions.value
  } else {
    delete jobSpec.completions
  }
  jobSpec.backoffLimit = typeof backoffLimit.value === 'number' ? backoffLimit.value : 6

  if (activeDeadlineSeconds.value !== null && activeDeadlineSeconds.value !== undefined) {
    jobSpec.activeDeadlineSeconds = activeDeadlineSeconds.value
  } else {
    delete jobSpec.activeDeadlineSeconds
  }

  if (ttlSecondsAfterFinished.value !== null && ttlSecondsAfterFinished.value !== undefined) {
    jobSpec.ttlSecondsAfterFinished = ttlSecondsAfterFinished.value
  } else {
    delete jobSpec.ttlSecondsAfterFinished
  }

  // 2. Metadata
  const cleanMetaLabels = kvArrayToObject(cronJobLabels.value)
  if (Object.keys(cleanMetaLabels).length > 0) {
    rawObj.metadata.labels = cleanMetaLabels
  } else {
    delete rawObj.metadata.labels
  }

  const cleanMetaAnnotations = kvArrayToObject(cronJobAnnotations.value)
  if (Object.keys(cleanMetaAnnotations).length > 0) {
    rawObj.metadata.annotations = cleanMetaAnnotations
  } else {
    delete rawObj.metadata.annotations
  }

  const cleanPodLabels = kvArrayToObject(podLabels.value)
  if (Object.keys(cleanPodLabels).length > 0) {
    jobSpec.template.metadata.labels = cleanPodLabels
  } else {
    delete jobSpec.template.metadata.labels
  }

  const cleanPodAnnotations = kvArrayToObject(podAnnotations.value)
  if (Object.keys(cleanPodAnnotations).length > 0) {
    jobSpec.template.metadata.annotations = cleanPodAnnotations
  } else {
    delete jobSpec.template.metadata.annotations
  }

  // 3. Pod Spec
  const podSpec = jobSpec.template.spec
  if (serviceAccountName.value.trim()) {
    podSpec.serviceAccountName = serviceAccountName.value.trim()
  } else {
    delete podSpec.serviceAccountName
  }

  podSpec.restartPolicy = restartPolicy.value
  podSpec.terminationGracePeriodSeconds = terminationGracePeriodSeconds.value
  podSpec.dnsPolicy = dnsPolicy.value

  const cleanNodeSelector = kvArrayToObject(nodeSelector.value)
  if (Object.keys(cleanNodeSelector).length > 0) {
    podSpec.nodeSelector = cleanNodeSelector
  } else {
    delete podSpec.nodeSelector
  }

  if (hostNetwork.value) {
    podSpec.hostNetwork = true
  } else {
    delete podSpec.hostNetwork
  }

  if (hostPID.value) {
    podSpec.hostPID = true
  } else {
    delete podSpec.hostPID
  }

  if (hostIPC.value) {
    podSpec.hostIPC = true
  } else {
    delete podSpec.hostIPC
  }

  // 4. Containers
  podSpec.containers = formatContainersToPodSpec(containers.value)

  emit('update:rawData', rawObj)
  isEmitting = false
}

const addContainer = () => {
  const newName = `container-${containers.value.length + 1}`
  containers.value.push({
    name: newName,
    image: '',
    imagePullPolicy: 'IfNotPresent',
    workingDir: '',
    command: [],
    args: [],
    environments: [],
    ports: [],
    cpuRequest: '',
    memoryRequest: '',
    cpuLimit: '',
    memoryLimit: '',
    rawContainer: { name: newName }
  })
  activeContainerIndex.value = containers.value.length - 1
  handleFieldChange()
}

const removeContainer = (index: number) => {
  if (containers.value.length <= 1) return
  containers.value.splice(index, 1)
  if (activeContainerIndex.value >= containers.value.length) {
    activeContainerIndex.value = Math.max(0, containers.value.length - 1)
  }
  handleFieldChange()
}
</script>

<template>
  <div class="flex flex-col gap-6" data-component="cronjob-edit-form">
    <Tabs v-model:value="activeTab">
      <TabList>
        <Tab value="general">General</Tab>
        <Tab value="metadata">Metadata</Tab>
        <Tab value="pod-spec">Pod Spec</Tab>
        <Tab value="containers">Containers</Tab>
      </TabList>

      <TabPanels class="pt-6">
        <!-- GENERAL TAB -->
        <TabPanel value="general">
          <div class="flex flex-col gap-8 max-w-4xl">
            <!-- Section 1: Schedule & Concurrency -->
            <div class="grid grid-cols-1 md:grid-cols-12 gap-6">
              <div class="md:col-span-4 flex flex-col gap-1">
                <span class="text-xs font-semibold tracking-wider text-muted-color uppercase">
                  Schedule & Concurrency
                </span>
                <p class="text-xs text-muted-color leading-relaxed">
                  Execution schedule in standard Cron syntax and concurrent run policy.
                </p>
              </div>
              <div class="md:col-span-8 flex flex-col gap-4">
                <div class="flex flex-col gap-1.5">
                  <label class="text-xs font-medium text-muted-color">Schedule (Cron)</label>
                  <InputText
                    v-model="schedule"
                    placeholder="*/5 * * * *"
                    fluid
                    :invalid="!isScheduleValid"
                    class="font-mono text-xs"
                    @update:model-value="handleFieldChange"
                  />
                  <span v-if="!isScheduleValid" class="text-[11px] text-(--danger)">
                    Invalid cron expression or macro.
                  </span>
                  <span v-else class="text-[11px] text-muted-color">
                    Standard 5-part Cron syntax (e.g. <code>*/5 * * * *</code>) or macros (e.g.
                    <code>@daily</code>).
                  </span>
                </div>

                <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
                  <div class="flex flex-col gap-1.5">
                    <label class="text-xs font-medium text-muted-color">Concurrency Policy</label>
                    <Select
                      v-model="concurrencyPolicy"
                      :options="KUBERNETES_CONCURRENCY_POLICIES"
                      fluid
                      class="text-xs"
                      @change="handleFieldChange"
                    />
                    <span class="text-[11px] text-muted-color">
                      Specifies how to treat concurrent executions of a Job.
                    </span>
                  </div>

                  <div class="flex flex-col gap-1.5">
                    <label class="text-xs font-medium text-muted-color">
                      Starting Deadline Seconds
                    </label>
                    <InputNumber
                      v-model="startingDeadlineSeconds"
                      :min="1"
                      fluid
                      placeholder="Optional"
                      @update:model-value="handleFieldChange"
                    />
                    <span class="text-[11px] text-muted-color">
                      Deadline in seconds for starting the job if schedule was missed.
                    </span>
                  </div>
                </div>

                <div
                  class="flex items-center justify-between p-4 rounded-[var(--ui-radius-card)] bg-(--bg-hover)/30"
                >
                  <div class="flex flex-col gap-0.5">
                    <span class="text-xs font-medium text-primary">Suspend CronJob</span>
                    <span class="text-[11px] text-muted-color">
                      Pause scheduled runs. Existing running Jobs will continue unless cancelled.
                    </span>
                  </div>
                  <ToggleSwitch v-model="suspend" @change="handleFieldChange" />
                </div>
              </div>
            </div>

            <!-- Section 2: History Limits -->
            <div class="grid grid-cols-1 md:grid-cols-12 gap-6">
              <div class="md:col-span-4 flex flex-col gap-1">
                <span class="text-xs font-semibold tracking-wider text-muted-color uppercase">
                  History Limits
                </span>
                <p class="text-xs text-muted-color leading-relaxed">
                  Controls how many completed and failed child Jobs are kept for auditing.
                </p>
              </div>
              <div class="md:col-span-8 flex flex-col gap-4">
                <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
                  <div class="flex flex-col gap-1.5">
                    <label class="text-xs font-medium text-muted-color">
                      Successful Jobs History Limit
                    </label>
                    <InputNumber
                      v-model="successfulJobsHistoryLimit"
                      :min="0"
                      fluid
                      @update:model-value="handleFieldChange"
                    />
                    <span class="text-[11px] text-muted-color">
                      Number of successful finished jobs to retain (defaults to 3).
                    </span>
                  </div>

                  <div class="flex flex-col gap-1.5">
                    <label class="text-xs font-medium text-muted-color">
                      Failed Jobs History Limit
                    </label>
                    <InputNumber
                      v-model="failedJobsHistoryLimit"
                      :min="0"
                      fluid
                      @update:model-value="handleFieldChange"
                    />
                    <span class="text-[11px] text-muted-color">
                      Number of failed finished jobs to retain (defaults to 1).
                    </span>
                  </div>
                </div>
              </div>
            </div>

            <!-- Section 3: Job Execution Parameters -->
            <div class="grid grid-cols-1 md:grid-cols-12 gap-6">
              <div class="md:col-span-4 flex flex-col gap-1">
                <span class="text-xs font-semibold tracking-wider text-muted-color uppercase">
                  Job Execution Defaults
                </span>
                <p class="text-xs text-muted-color leading-relaxed">
                  Execution boundaries applied to each spawned Job.
                </p>
              </div>
              <div class="md:col-span-8 flex flex-col gap-4">
                <div class="grid grid-cols-1 sm:grid-cols-3 gap-4">
                  <div class="flex flex-col gap-1.5">
                    <label class="text-xs font-medium text-muted-color">Completions</label>
                    <InputNumber
                      v-model="completions"
                      :min="1"
                      fluid
                      placeholder="1"
                      @update:model-value="handleFieldChange"
                    />
                    <span class="text-[11px] text-muted-color">
                      Desired successful pods to complete the job.
                    </span>
                  </div>

                  <div class="flex flex-col gap-1.5">
                    <label class="text-xs font-medium text-muted-color">Parallelism</label>
                    <InputNumber
                      v-model="parallelism"
                      :min="0"
                      fluid
                      @update:model-value="handleFieldChange"
                    />
                    <span class="text-[11px] text-muted-color">
                      Maximum pods running simultaneously.
                    </span>
                  </div>

                  <div class="flex flex-col gap-1.5">
                    <label class="text-xs font-medium text-muted-color">Backoff Limit</label>
                    <InputNumber
                      v-model="backoffLimit"
                      :min="0"
                      fluid
                      @update:model-value="handleFieldChange"
                    />
                    <span class="text-[11px] text-muted-color">
                      Retries before marking a job as failed (defaults to 6).
                    </span>
                  </div>
                </div>

                <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
                  <div class="flex flex-col gap-1.5">
                    <label class="text-xs font-medium text-muted-color">
                      Active Deadline Seconds
                    </label>
                    <InputNumber
                      v-model="activeDeadlineSeconds"
                      :min="1"
                      fluid
                      placeholder="Unlimited"
                      @update:model-value="handleFieldChange"
                    />
                    <span class="text-[11px] text-muted-color">
                      Maximum duration each spawned Job may run.
                    </span>
                  </div>

                  <div class="flex flex-col gap-1.5">
                    <label class="text-xs font-medium text-muted-color">
                      TTL Seconds After Finished
                    </label>
                    <InputNumber
                      v-model="ttlSecondsAfterFinished"
                      :min="0"
                      fluid
                      placeholder="Disabled"
                      @update:model-value="handleFieldChange"
                    />
                    <span class="text-[11px] text-muted-color">
                      Seconds before finished job is cleaned up automatically.
                    </span>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </TabPanel>

        <!-- METADATA TAB -->
        <TabPanel value="metadata">
          <div class="flex flex-col gap-8 max-w-4xl">
            <!-- CronJob Labels & Annotations -->
            <div class="grid grid-cols-1 md:grid-cols-12 gap-6">
              <div class="md:col-span-4 flex flex-col gap-1">
                <span class="text-xs font-semibold tracking-wider text-muted-color uppercase">
                  CronJob Metadata
                </span>
                <p class="text-xs text-muted-color leading-relaxed">
                  Key-value pairs applied directly to the CronJob object.
                </p>
              </div>
              <div class="md:col-span-8 flex flex-col gap-6">
                <div class="flex flex-col gap-2">
                  <label class="text-xs font-medium text-muted-color">CronJob Labels</label>
                  <KeyValueEditor
                    v-model="cronJobLabels"
                    placeholder-key="app"
                    placeholder-value="cron-worker"
                    @update:model-value="handleFieldChange"
                  />
                </div>

                <div class="flex flex-col gap-2">
                  <label class="text-xs font-medium text-muted-color">CronJob Annotations</label>
                  <KeyValueEditor
                    v-model="cronJobAnnotations"
                    placeholder-key="description"
                    placeholder-value="Scheduled database sync"
                    @update:model-value="handleFieldChange"
                  />
                </div>
              </div>
            </div>

            <!-- Pod Template Labels & Annotations -->
            <div class="grid grid-cols-1 md:grid-cols-12 gap-6">
              <div class="md:col-span-4 flex flex-col gap-1">
                <span class="text-xs font-semibold tracking-wider text-muted-color uppercase">
                  Pod Template Metadata
                </span>
                <p class="text-xs text-muted-color leading-relaxed">
                  Key-value pairs inherited by the pods created by this CronJob.
                </p>
              </div>
              <div class="md:col-span-8 flex flex-col gap-6">
                <div class="flex flex-col gap-2">
                  <label class="text-xs font-medium text-muted-color">Pod Labels</label>
                  <KeyValueEditor
                    v-model="podLabels"
                    placeholder-key="tier"
                    placeholder-value="batch"
                    @update:model-value="handleFieldChange"
                  />
                </div>

                <div class="flex flex-col gap-2">
                  <label class="text-xs font-medium text-muted-color">Pod Annotations</label>
                  <KeyValueEditor
                    v-model="podAnnotations"
                    placeholder-key="sidecar.istio.io/inject"
                    placeholder-value="false"
                    @update:model-value="handleFieldChange"
                  />
                </div>
              </div>
            </div>
          </div>
        </TabPanel>

        <!-- POD SPEC TAB -->
        <TabPanel value="pod-spec">
          <div class="flex flex-col gap-8 max-w-4xl">
            <!-- Security & Identity -->
            <div class="grid grid-cols-1 md:grid-cols-12 gap-6">
              <div class="md:col-span-4 flex flex-col gap-1">
                <span class="text-xs font-semibold tracking-wider text-muted-color uppercase">
                  Security & Identity
                </span>
                <p class="text-xs text-muted-color leading-relaxed">
                  Configure the Service Account and execution lifecycle.
                </p>
              </div>
              <div class="md:col-span-8 flex flex-col gap-4">
                <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
                  <div class="flex flex-col gap-1.5">
                    <label class="text-xs font-medium text-muted-color">
                      Service Account Name
                    </label>
                    <InputText
                      v-model="serviceAccountName"
                      placeholder="default"
                      fluid
                      class="text-xs"
                      @update:model-value="handleFieldChange"
                    />
                  </div>

                  <div class="flex flex-col gap-1.5">
                    <label class="text-xs font-medium text-muted-color">Restart Policy</label>
                    <Select
                      v-model="restartPolicy"
                      :options="KUBERNETES_JOB_RESTART_POLICIES"
                      fluid
                      class="text-xs"
                      @change="handleFieldChange"
                    />
                    <span class="text-[11px] text-muted-color">
                      CronJobs support OnFailure or Never.
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
                      fluid
                      @update:model-value="handleFieldChange"
                    />
                  </div>

                  <div class="flex flex-col gap-1.5">
                    <label class="text-xs font-medium text-muted-color">DNS Policy</label>
                    <Select
                      v-model="dnsPolicy"
                      :options="KUBERNETES_DNS_POLICIES"
                      fluid
                      class="text-xs"
                      @change="handleFieldChange"
                    />
                  </div>
                </div>
              </div>
            </div>

            <!-- Host Namespaces & Scheduling -->
            <div class="grid grid-cols-1 md:grid-cols-12 gap-6">
              <div class="md:col-span-4 flex flex-col gap-1">
                <span class="text-xs font-semibold tracking-wider text-muted-color uppercase">
                  Host Namespaces & Placement
                </span>
                <p class="text-xs text-muted-color leading-relaxed">
                  Node scheduling selectors and host networking flags.
                </p>
              </div>
              <div class="md:col-span-8 flex flex-col gap-4">
                <div class="flex flex-col gap-2">
                  <label class="text-xs font-medium text-muted-color">Node Selector</label>
                  <KeyValueEditor
                    v-model="nodeSelector"
                    placeholder-key="disktype"
                    placeholder-value="ssd"
                    @update:model-value="handleFieldChange"
                  />
                </div>

                <div class="grid grid-cols-1 sm:grid-cols-3 gap-4 pt-2">
                  <div
                    class="flex items-center justify-between p-3 rounded-[var(--ui-radius-card)] bg-(--bg-hover)/30"
                  >
                    <span class="text-xs font-medium text-primary">Host Network</span>
                    <ToggleSwitch v-model="hostNetwork" @change="handleFieldChange" />
                  </div>
                  <div
                    class="flex items-center justify-between p-3 rounded-[var(--ui-radius-card)] bg-(--bg-hover)/30"
                  >
                    <span class="text-xs font-medium text-primary">Host PID</span>
                    <ToggleSwitch v-model="hostPID" @change="handleFieldChange" />
                  </div>
                  <div
                    class="flex items-center justify-between p-3 rounded-[var(--ui-radius-card)] bg-(--bg-hover)/30"
                  >
                    <span class="text-xs font-medium text-primary">Host IPC</span>
                    <ToggleSwitch v-model="hostIPC" @change="handleFieldChange" />
                  </div>
                </div>
              </div>
            </div>
          </div>
        </TabPanel>

        <!-- CONTAINERS TAB -->
        <TabPanel value="containers">
          <div class="flex flex-col gap-6 max-w-4xl">
            <!-- Container Pills / Selector -->
            <div class="flex items-center gap-2 flex-wrap">
              <span v-if="containers.length > 1" class="text-xs font-medium text-muted-color">
                Container:
              </span>
              <Button
                v-for="(c, idx) in containers"
                :key="'cbtn-' + idx"
                :label="c.name || `Container ${idx + 1}`"
                :variant="activeContainerIndex === idx ? undefined : 'text'"
                :severity="activeContainerIndex === idx ? 'primary' : 'secondary'"
                class="text-xs cursor-pointer flex items-center gap-1.5"
                @click="activeContainerIndex = idx"
              >
                <span>{{ c.name || `Container ${idx + 1}` }}</span>
                <span
                  v-if="containers.length > 1"
                  class="hover:opacity-75 cursor-pointer ml-1 text-xs"
                  @click.stop="removeContainer(idx)"
                >
                  &times;
                </span>
              </Button>

              <Button
                variant="outlined"
                severity="secondary"
                class="text-xs flex items-center gap-1.5"
                @click="addContainer"
              >
                <i class="pi pi-plus text-xs" />
                <span>Add Container</span>
              </Button>
            </div>

            <!-- Active Container Form -->
            <div v-if="currentContainer" class="flex flex-col gap-8">
              <!-- Basic Info -->
              <div class="grid grid-cols-1 md:grid-cols-12 gap-6">
                <div class="md:col-span-4 flex flex-col gap-1">
                  <span class="text-xs font-semibold tracking-wider text-muted-color uppercase">
                    Container Details
                  </span>
                  <p class="text-xs text-muted-color leading-relaxed">
                    Name, image repository, pull policy, and working directory.
                  </p>
                </div>
                <div class="md:col-span-8 flex flex-col gap-4">
                  <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
                    <div class="flex flex-col gap-1.5">
                      <label class="text-xs font-medium text-muted-color">Container Name</label>
                      <InputText
                        v-model="currentContainer.name"
                        fluid
                        :invalid="!isValidK8sLabel(currentContainer.name)"
                        class="text-xs"
                        @update:model-value="handleFieldChange"
                      />
                      <span
                        v-if="!isValidK8sLabel(currentContainer.name)"
                        class="text-[11px] text-(--danger)"
                      >
                        Must be a valid DNS-1123 label.
                      </span>
                    </div>

                    <div class="flex flex-col gap-1.5">
                      <label class="text-xs font-medium text-muted-color">Image Pull Policy</label>
                      <Select
                        v-model="currentContainer.imagePullPolicy"
                        :options="KUBERNETES_IMAGE_PULL_POLICIES"
                        fluid
                        class="text-xs"
                        @change="handleFieldChange"
                      />
                    </div>
                  </div>

                  <div class="flex flex-col gap-1.5">
                    <label class="text-xs font-medium text-muted-color">Container Image</label>
                    <InputText
                      v-model="currentContainer.image"
                      placeholder="e.g. busybox:latest"
                      fluid
                      class="text-xs font-mono"
                      @update:model-value="handleFieldChange"
                    />
                  </div>

                  <div class="flex flex-col gap-1.5">
                    <label class="text-xs font-medium text-muted-color">Working Directory</label>
                    <InputText
                      v-model="currentContainer.workingDir"
                      placeholder="/app"
                      fluid
                      :invalid="!isValidPath(currentContainer.workingDir)"
                      class="text-xs font-mono"
                      @update:model-value="handleFieldChange"
                    />
                  </div>
                </div>
              </div>

              <!-- Resources -->
              <div class="grid grid-cols-1 md:grid-cols-12 gap-6">
                <div class="md:col-span-4 flex flex-col gap-1">
                  <span class="text-xs font-semibold tracking-wider text-muted-color uppercase">
                    Compute Resources
                  </span>
                  <p class="text-xs text-muted-color leading-relaxed">
                    CPU and memory request / limit constraints.
                  </p>
                </div>
                <div class="md:col-span-8">
                  <ContainerResourcesEditor
                    v-model:cpu-request="currentContainer.cpuRequest"
                    v-model:memory-request="currentContainer.memoryRequest"
                    v-model:cpu-limit="currentContainer.cpuLimit"
                    v-model:memory-limit="currentContainer.memoryLimit"
                    @change="handleFieldChange"
                  />
                </div>
              </div>

              <!-- Commands & Arguments -->
              <div class="grid grid-cols-1 md:grid-cols-12 gap-6">
                <div class="md:col-span-4 flex flex-col gap-1">
                  <span class="text-xs font-semibold tracking-wider text-muted-color uppercase">
                    Commands & Arguments
                  </span>
                  <p class="text-xs text-muted-color leading-relaxed">
                    Override container entrypoint and arguments.
                  </p>
                </div>
                <div class="md:col-span-8 flex flex-col gap-4">
                  <div class="flex flex-col gap-2">
                    <label class="text-xs font-medium text-muted-color">Command (Entrypoint)</label>
                    <StringListEditor
                      v-model="currentContainer.command"
                      placeholder="e.g. /bin/sh"
                      @change="handleFieldChange"
                    />
                  </div>
                  <div class="flex flex-col gap-2">
                    <label class="text-xs font-medium text-muted-color">Arguments</label>
                    <StringListEditor
                      v-model="currentContainer.args"
                      placeholder="e.g. -c"
                      @change="handleFieldChange"
                    />
                  </div>
                </div>
              </div>

              <!-- Environments -->
              <div class="grid grid-cols-1 md:grid-cols-12 gap-6">
                <div class="md:col-span-4 flex flex-col gap-1">
                  <span class="text-xs font-semibold tracking-wider text-muted-color uppercase">
                    Environment Variables
                  </span>
                  <p class="text-xs text-muted-color leading-relaxed">
                    Literal, Secret, ConfigMap, and Field references.
                  </p>
                </div>
                <div class="md:col-span-8">
                  <ContainerEnvEditor
                    v-model="currentContainer.environments"
                    :namespace="cronJobNamespace"
                    @change="handleFieldChange"
                  />
                </div>
              </div>

              <!-- Ports -->
              <div class="grid grid-cols-1 md:grid-cols-12 gap-6">
                <div class="md:col-span-4 flex flex-col gap-1">
                  <span class="text-xs font-semibold tracking-wider text-muted-color uppercase">
                    Container Ports
                  </span>
                  <p class="text-xs text-muted-color leading-relaxed">
                    Exposed network ports and protocols.
                  </p>
                </div>
                <div class="md:col-span-8">
                  <ContainerPortsEditor
                    v-model="currentContainer.ports"
                    @change="handleFieldChange"
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
