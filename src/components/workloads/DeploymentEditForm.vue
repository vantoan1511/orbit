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

import ContainerPortsEditor from '@/components/shared/ContainerPortsEditor.vue'
import ContainerResourcesEditor from '@/components/shared/ContainerResourcesEditor.vue'
import KeyValueEditor from '@/components/shared/KeyValueEditor.vue'
import StringListEditor from '@/components/shared/StringListEditor.vue'

const props = defineProps<{
  rawData: Record<string, unknown> | null
}>()

const emit = defineEmits<{
  (e: 'update:rawData', value: Record<string, unknown>): void
}>()

const activeTab = ref('general')

// Form reactive state
const replicas = ref<number>(1)
const paused = ref<boolean>(false)
const strategyType = ref<string>('RollingUpdate')
const maxSurge = ref<string>('25%')
const maxUnavailable = ref<string>('25%')
const minReadySeconds = ref<number>(0)
const revisionHistoryLimit = ref<number>(10)
const progressDeadlineSeconds = ref<number>(600)
const selectorLabels = ref<{ key: string; value: string }[]>([])

const deploymentLabels = ref<{ key: string; value: string }[]>([])
const deploymentAnnotations = ref<{ key: string; value: string }[]>([])
const podLabels = ref<{ key: string; value: string }[]>([])
const podAnnotations = ref<{ key: string; value: string }[]>([])

const serviceAccountName = ref<string>('')
const restartPolicy = ref<string>('Always')
const terminationGracePeriodSeconds = ref<number>(30)
const nodeSelector = ref<{ key: string; value: string }[]>([])

interface ContainerFormState {
  name: string
  image: string
  imagePullPolicy: string
  workingDir: string
  command: string[]
  args: string[]
  env: { key: string; value: string }[]
  preservedValueFromEnv: unknown[]
  ports: { name: string; containerPort: number; protocol: string }[]
  cpuRequest: string
  memoryRequest: string
  cpuLimit: string
  memoryLimit: string
  rawContainer: Record<string, unknown>
}

const containers = ref<ContainerFormState[]>([])
const activeContainerIndex = ref<number>(0)

let isEmitting = false

const kvObjectToArray = (
  obj: Record<string, unknown> | undefined
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

const syncFromRawData = (data: Record<string, unknown> | null) => {
  if (!data || isEmitting) return

  const spec = (data.spec as Record<string, unknown>) || {}
  replicas.value = typeof spec.replicas === 'number' ? spec.replicas : 1
  paused.value = Boolean(spec.paused)

  const strat = (spec.strategy as Record<string, unknown>) || {}
  strategyType.value = typeof strat.type === 'string' ? strat.type : 'RollingUpdate'
  const rolling = (strat.rollingUpdate as Record<string, unknown>) || {}
  maxSurge.value = rolling.maxSurge !== undefined ? String(rolling.maxSurge) : '25%'
  maxUnavailable.value =
    rolling.maxUnavailable !== undefined ? String(rolling.maxUnavailable) : '25%'

  minReadySeconds.value = typeof spec.minReadySeconds === 'number' ? spec.minReadySeconds : 0
  revisionHistoryLimit.value =
    typeof spec.revisionHistoryLimit === 'number' ? spec.revisionHistoryLimit : 10
  progressDeadlineSeconds.value =
    typeof spec.progressDeadlineSeconds === 'number' ? spec.progressDeadlineSeconds : 600

  const selector = (spec.selector as Record<string, unknown>) || {}
  const matchLbls = (selector.matchLabels as Record<string, unknown>) || {}
  selectorLabels.value = kvObjectToArray(matchLbls)

  // Metadata
  const metadata = (data.metadata as Record<string, unknown>) || {}
  deploymentLabels.value = kvObjectToArray(metadata.labels as Record<string, unknown>)
  deploymentAnnotations.value = kvObjectToArray(metadata.annotations as Record<string, unknown>)

  // Pod Template Metadata
  const template = (spec.template as Record<string, unknown>) || {}
  const podMeta = (template.metadata as Record<string, unknown>) || {}
  podLabels.value = kvObjectToArray(podMeta.labels as Record<string, unknown>)
  podAnnotations.value = kvObjectToArray(podMeta.annotations as Record<string, unknown>)

  // Pod Spec
  const podSpec = (template.spec as Record<string, unknown>) || {}
  serviceAccountName.value =
    typeof podSpec.serviceAccountName === 'string' ? podSpec.serviceAccountName : ''
  restartPolicy.value = typeof podSpec.restartPolicy === 'string' ? podSpec.restartPolicy : 'Always'
  terminationGracePeriodSeconds.value =
    typeof podSpec.terminationGracePeriodSeconds === 'number'
      ? podSpec.terminationGracePeriodSeconds
      : 30
  nodeSelector.value = kvObjectToArray(podSpec.nodeSelector as Record<string, unknown>)

  // Containers
  const rawContainers: Record<string, unknown>[] = Array.isArray(podSpec.containers)
    ? (podSpec.containers as Record<string, unknown>[])
    : []
  containers.value = rawContainers.map((c) => {
    const envList = Array.isArray(c.env) ? (c.env as Record<string, unknown>[]) : []
    const simpleEnvs: { key: string; value: string }[] = []
    const preservedEnvs: unknown[] = []

    for (const e of envList) {
      if (e && typeof e === 'object' && 'name' in e) {
        if ('valueFrom' in e || !('value' in e)) {
          preservedEnvs.push(e)
        } else {
          simpleEnvs.push({ key: String(e.name), value: String(e.value ?? '') })
        }
      }
    }

    const portList = Array.isArray(c.ports) ? (c.ports as Record<string, unknown>[]) : []
    const parsedPorts = portList.map((p) => ({
      name: typeof p.name === 'string' ? p.name : '',
      containerPort: typeof p.containerPort === 'number' ? p.containerPort : 80,
      protocol: typeof p.protocol === 'string' ? p.protocol : 'TCP'
    }))

    const resources = (c.resources as Record<string, unknown>) || {}
    const reqs = (resources.requests as Record<string, unknown>) || {}
    const lims = (resources.limits as Record<string, unknown>) || {}

    return {
      name: typeof c.name === 'string' ? c.name : '',
      image: typeof c.image === 'string' ? c.image : '',
      imagePullPolicy: typeof c.imagePullPolicy === 'string' ? c.imagePullPolicy : 'IfNotPresent',
      workingDir: typeof c.workingDir === 'string' ? c.workingDir : '',
      command: Array.isArray(c.command) ? (c.command as string[]) : [],
      args: Array.isArray(c.args) ? (c.args as string[]) : [],
      env: simpleEnvs,
      preservedValueFromEnv: preservedEnvs,
      ports: parsedPorts,
      cpuRequest: reqs.cpu ? String(reqs.cpu) : '',
      memoryRequest: reqs.memory ? String(reqs.memory) : '',
      cpuLimit: lims.cpu ? String(lims.cpu) : '',
      memoryLimit: lims.memory ? String(lims.memory) : '',
      rawContainer: c
    }
  })

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

watch(
  [
    replicas,
    paused,
    strategyType,
    maxSurge,
    maxUnavailable,
    minReadySeconds,
    revisionHistoryLimit,
    progressDeadlineSeconds,
    serviceAccountName,
    restartPolicy,
    terminationGracePeriodSeconds
  ],
  () => {
    if (!isEmitting) {
      handleFieldChange()
    }
  }
)

const emitUpdate = () => {
  if (!props.rawData) return
  isEmitting = true

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const rawObj = JSON.parse(JSON.stringify(toRaw(props.rawData))) as Record<string, any>
  if (!rawObj.metadata) rawObj.metadata = {}
  if (!rawObj.spec) rawObj.spec = {}
  if (!rawObj.spec.template) rawObj.spec.template = {}
  if (!rawObj.spec.template.metadata) rawObj.spec.template.metadata = {}
  if (!rawObj.spec.template.spec) rawObj.spec.template.spec = {}

  // 1. General & Strategy
  rawObj.spec.replicas = replicas.value
  rawObj.spec.paused = paused.value

  if (!rawObj.spec.strategy) rawObj.spec.strategy = {}
  rawObj.spec.strategy.type = strategyType.value

  if (strategyType.value === 'RollingUpdate') {
    if (!rawObj.spec.strategy.rollingUpdate) rawObj.spec.strategy.rollingUpdate = {}
    if (maxSurge.value) rawObj.spec.strategy.rollingUpdate.maxSurge = maxSurge.value
    if (maxUnavailable.value)
      rawObj.spec.strategy.rollingUpdate.maxUnavailable = maxUnavailable.value
  } else {
    delete rawObj.spec.strategy.rollingUpdate
  }

  rawObj.spec.minReadySeconds = minReadySeconds.value
  rawObj.spec.revisionHistoryLimit = revisionHistoryLimit.value
  rawObj.spec.progressDeadlineSeconds = progressDeadlineSeconds.value

  // 2. Metadata
  rawObj.metadata.labels = kvArrayToObject(deploymentLabels.value)
  rawObj.metadata.annotations = kvArrayToObject(deploymentAnnotations.value)
  rawObj.spec.template.metadata.labels = kvArrayToObject(podLabels.value)
  rawObj.spec.template.metadata.annotations = kvArrayToObject(podAnnotations.value)

  // 3. Pod Spec
  if (serviceAccountName.value) {
    rawObj.spec.template.spec.serviceAccountName = serviceAccountName.value
  } else {
    delete rawObj.spec.template.spec.serviceAccountName
  }

  rawObj.spec.template.spec.restartPolicy = restartPolicy.value
  rawObj.spec.template.spec.terminationGracePeriodSeconds = terminationGracePeriodSeconds.value

  const nodeSel = kvArrayToObject(nodeSelector.value)
  if (Object.keys(nodeSel).length > 0) {
    rawObj.spec.template.spec.nodeSelector = nodeSel
  } else {
    delete rawObj.spec.template.spec.nodeSelector
  }

  // 4. Containers
  const updatedContainers = containers.value.map((c) => {
    const containerObj: Record<string, unknown> = { ...c.rawContainer }
    containerObj.name = c.name
    containerObj.image = c.image
    containerObj.imagePullPolicy = c.imagePullPolicy

    if (c.workingDir) {
      containerObj.workingDir = c.workingDir
    } else {
      delete containerObj.workingDir
    }

    if (c.command.length > 0) {
      containerObj.command = c.command.filter((cmd) => cmd.trim() !== '')
    } else {
      delete containerObj.command
    }

    if (c.args.length > 0) {
      containerObj.args = c.args.filter((arg) => arg.trim() !== '')
    } else {
      delete containerObj.args
    }

    // Env vars
    const simpleEnvsMapped = c.env
      .filter((e) => e.key.trim() !== '')
      .map((e) => ({ name: e.key.trim(), value: e.value }))

    const combinedEnv = [...c.preservedValueFromEnv, ...simpleEnvsMapped]
    if (combinedEnv.length > 0) {
      containerObj.env = combinedEnv
    } else {
      delete containerObj.env
    }

    // Ports
    const validPorts = c.ports.map((p) => {
      const portItem: { containerPort: number; protocol: string; name?: string } = {
        containerPort: p.containerPort,
        protocol: p.protocol || 'TCP'
      }
      if (p.name.trim()) portItem.name = p.name.trim()
      return portItem
    })
    if (validPorts.length > 0) {
      containerObj.ports = validPorts
    } else {
      delete containerObj.ports
    }

    // Resources
    const reqs: Record<string, string> = {}
    if (c.cpuRequest.trim()) reqs.cpu = c.cpuRequest.trim()
    if (c.memoryRequest.trim()) reqs.memory = c.memoryRequest.trim()

    const lims: Record<string, string> = {}
    if (c.cpuLimit.trim()) lims.cpu = c.cpuLimit.trim()
    if (c.memoryLimit.trim()) lims.memory = c.memoryLimit.trim()

    if (Object.keys(reqs).length > 0 || Object.keys(lims).length > 0) {
      const resObj = (containerObj.resources as Record<string, unknown>) || {}
      if (Object.keys(reqs).length > 0) resObj.requests = reqs
      else delete resObj.requests

      if (Object.keys(lims).length > 0) resObj.limits = lims
      else delete resObj.limits

      containerObj.resources = resObj
    } else {
      delete containerObj.resources
    }

    return containerObj
  })

  rawObj.spec.template.spec.containers = updatedContainers

  emit('update:rawData', rawObj)

  setTimeout(() => {
    isEmitting = false
  }, 50)
}

const handleFieldChange = () => {
  emitUpdate()
}

const currentContainer = computed(() => containers.value[activeContainerIndex.value] || null)
</script>

<template>
  <div class="flex flex-col h-full w-full">
    <Tabs v-model:value="activeTab" class="w-full flex flex-col h-full">
      <TabList>
        <Tab value="general" class="text-xs font-medium">General & Scaling</Tab>
        <Tab value="metadata" class="text-xs font-medium">Metadata</Tab>
        <Tab value="pod" class="text-xs font-medium">Pod Spec</Tab>
        <Tab value="containers" class="text-xs font-medium">Containers</Tab>
      </TabList>

      <TabPanels class="flex-1 overflow-y-auto pt-6 px-0">
        <!-- GENERAL & SCALING TAB -->
        <TabPanel value="general" class="flex flex-col gap-10 max-w-5xl">
          <!-- Section 1: Deployment Scaling -->
          <div class="grid grid-cols-1 md:grid-cols-12 gap-6">
            <div class="md:col-span-4 flex flex-col gap-1">
              <span class="text-xs font-semibold tracking-wider text-muted-color uppercase">
                Deployment Scaling
              </span>
              <p class="text-xs text-muted-color leading-relaxed">
                Desired pod replica count and execution status.
              </p>
            </div>
            <div class="md:col-span-8 flex items-center gap-8">
              <div class="flex flex-col gap-1.5">
                <label class="text-xs font-medium text-muted-color">Replicas</label>
                <InputNumber
                  v-model="replicas"
                  showButtons
                  buttonLayout="horizontal"
                  size="small"
                  :min="0"
                  :max="1000"
                  @update:model-value="handleFieldChange"
                  @change="handleFieldChange"
                />
              </div>

              <div class="flex flex-col gap-1.5">
                <label class="text-xs font-medium text-muted-color">Execution State</label>
                <div class="flex items-center gap-2 mt-1">
                  <ToggleSwitch v-model="paused" @change="handleFieldChange" />
                  <span class="text-xs text-muted-color">
                    {{ paused ? 'Deployment Paused' : 'Active' }}
                  </span>
                </div>
              </div>
            </div>
          </div>

          <!-- Section 2: Deployment Strategy -->
          <div class="grid grid-cols-1 md:grid-cols-12 gap-6">
            <div class="md:col-span-4 flex flex-col gap-1">
              <span class="text-xs font-semibold tracking-wider text-muted-color uppercase">
                Deployment Strategy
              </span>
              <p class="text-xs text-muted-color leading-relaxed">
                Strategy used to replace old pods with new pods during rollout.
              </p>
            </div>
            <div class="md:col-span-8 flex flex-col gap-4">
              <div class="flex flex-col gap-1.5">
                <label class="text-xs font-medium text-muted-color">Strategy Type</label>
                <Select
                  v-model="strategyType"
                  :options="['RollingUpdate', 'Recreate']"
                  size="small"
                  class="w-full md:w-64"
                  @change="handleFieldChange"
                />
              </div>

              <div v-if="strategyType === 'RollingUpdate'" class="grid grid-cols-2 gap-4">
                <div class="flex flex-col gap-1.5">
                  <label class="text-xs font-medium text-muted-color">Max Surge</label>
                  <InputText
                    v-model="maxSurge"
                    placeholder="e.g. 25% or 1"
                    size="small"
                    fluid
                    @input="handleFieldChange"
                  />
                </div>

                <div class="flex flex-col gap-1.5">
                  <label class="text-xs font-medium text-muted-color">Max Unavailable</label>
                  <InputText
                    v-model="maxUnavailable"
                    placeholder="e.g. 25% or 0"
                    size="small"
                    fluid
                    @input="handleFieldChange"
                  />
                </div>
              </div>
            </div>
          </div>

          <!-- Section 3: Timing & Limits -->
          <div class="grid grid-cols-1 md:grid-cols-12 gap-6">
            <div class="md:col-span-4 flex flex-col gap-1">
              <span class="text-xs font-semibold tracking-wider text-muted-color uppercase">
                Timing & Limits
              </span>
              <p class="text-xs text-muted-color leading-relaxed">
                Deadlines, minimal ready time, and revision history retention.
              </p>
            </div>
            <div class="md:col-span-8 grid grid-cols-3 gap-4">
              <div class="flex flex-col gap-1.5">
                <label class="text-xs font-medium text-muted-color">Min Ready Secs</label>
                <InputNumber
                  v-model="minReadySeconds"
                  :min="0"
                  size="small"
                  fluid
                  @change="handleFieldChange"
                />
              </div>

              <div class="flex flex-col gap-1.5">
                <label class="text-xs font-medium text-muted-color">Rev History Limit</label>
                <InputNumber
                  v-model="revisionHistoryLimit"
                  :min="0"
                  size="small"
                  fluid
                  @change="handleFieldChange"
                />
              </div>

              <div class="flex flex-col gap-1.5">
                <label class="text-xs font-medium text-muted-color">Progress Deadline</label>
                <InputNumber
                  v-model="progressDeadlineSeconds"
                  :min="0"
                  size="small"
                  fluid
                  @change="handleFieldChange"
                />
              </div>
            </div>
          </div>

          <!-- Section 4: Read-only Selector -->
          <div class="grid grid-cols-1 md:grid-cols-12 gap-6">
            <div class="md:col-span-4 flex flex-col gap-1">
              <div class="flex items-center gap-1.5">
                <span class="text-xs font-semibold tracking-wider text-muted-color uppercase">
                  Selector Labels
                </span>
              </div>
              <p class="text-xs text-muted-color leading-relaxed">
                Pod selector labels. Immutable after deployment creation.
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
        <TabPanel value="metadata" class="flex flex-col gap-10 max-w-5xl">
          <!-- Deployment Level -->
          <div class="grid grid-cols-1 md:grid-cols-12 gap-6">
            <div class="md:col-span-4 flex flex-col gap-1">
              <span class="text-xs font-semibold tracking-wider text-muted-color uppercase">
                Deployment Metadata
              </span>
              <p class="text-xs text-muted-color leading-relaxed">
                Labels and annotations attached to the Deployment resource itself.
              </p>
            </div>
            <div class="md:col-span-8 flex flex-col gap-6">
              <div class="p-4 rounded-lg bg-(--bg-hover)/30 flex flex-col gap-3">
                <KeyValueEditor
                  v-model="deploymentLabels"
                  title="Deployment Labels"
                  add-label="Add Label"
                  @update:model-value="handleFieldChange"
                />
              </div>
              <div class="p-4 rounded-lg bg-(--bg-hover)/30 flex flex-col gap-3">
                <KeyValueEditor
                  v-model="deploymentAnnotations"
                  title="Deployment Annotations"
                  add-label="Add Annotation"
                  @update:model-value="handleFieldChange"
                />
              </div>
            </div>
          </div>

          <!-- Pod Template Level -->
          <div class="grid grid-cols-1 md:grid-cols-12 gap-6">
            <div class="md:col-span-4 flex flex-col gap-1">
              <span class="text-xs font-semibold tracking-wider text-muted-color uppercase">
                Pod Template Metadata
              </span>
              <p class="text-xs text-muted-color leading-relaxed">
                Labels and annotations propagated to created Pod instances.
              </p>
            </div>
            <div class="md:col-span-8 flex flex-col gap-6">
              <div class="p-4 rounded-lg bg-(--bg-hover)/30 flex flex-col gap-3">
                <KeyValueEditor
                  v-model="podLabels"
                  title="Pod Template Labels"
                  add-label="Add Label"
                  @update:model-value="handleFieldChange"
                />
              </div>
              <div class="p-4 rounded-lg bg-(--bg-hover)/30 flex flex-col gap-3">
                <KeyValueEditor
                  v-model="podAnnotations"
                  title="Pod Template Annotations"
                  add-label="Add Annotation"
                  @update:model-value="handleFieldChange"
                />
              </div>
            </div>
          </div>
        </TabPanel>

        <!-- POD SPEC TAB -->
        <TabPanel value="pod" class="flex flex-col gap-10 max-w-5xl">
          <!-- Execution Settings -->
          <div class="grid grid-cols-1 md:grid-cols-12 gap-6">
            <div class="md:col-span-4 flex flex-col gap-1">
              <span class="text-xs font-semibold tracking-wider text-muted-color uppercase">
                Execution Settings
              </span>
              <p class="text-xs text-muted-color leading-relaxed">
                Service account authorization, restart policies, and grace periods.
              </p>
            </div>
            <div class="md:col-span-8 flex flex-col gap-4">
              <div class="flex flex-col gap-1.5">
                <label class="text-xs font-medium text-muted-color">Service Account Name</label>
                <InputText
                  v-model="serviceAccountName"
                  placeholder="e.g. default"
                  size="small"
                  fluid
                  @input="handleFieldChange"
                />
              </div>

              <div class="grid grid-cols-2 gap-4">
                <div class="flex flex-col gap-1.5">
                  <label class="text-xs font-medium text-muted-color">Restart Policy</label>
                  <Select
                    v-model="restartPolicy"
                    :options="['Always', 'OnFailure', 'Never']"
                    size="small"
                    fluid
                    @change="handleFieldChange"
                  />
                </div>

                <div class="flex flex-col gap-1.5">
                  <label class="text-xs font-medium text-muted-color"
                    >Termination Grace Period (s)</label
                  >
                  <InputNumber
                    v-model="terminationGracePeriodSeconds"
                    :min="0"
                    size="small"
                    fluid
                    @change="handleFieldChange"
                  />
                </div>
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
                Key-value selector constraints for targeting specific Kubernetes nodes.
              </p>
            </div>
            <div class="md:col-span-8 flex flex-col gap-3">
              <div class="p-4 rounded-lg bg-(--bg-hover)/30">
                <KeyValueEditor
                  v-model="nodeSelector"
                  title="Node Selector"
                  add-label="Add Constraint"
                  @update:model-value="handleFieldChange"
                />
              </div>
            </div>
          </div>
        </TabPanel>

        <!-- CONTAINERS TAB -->
        <TabPanel value="containers" class="flex flex-col gap-10 max-w-5xl">
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
                  Basic container identifier, image reference, and image pull policy.
                </p>
              </div>
              <div class="md:col-span-8 flex flex-col gap-4">
                <div class="grid grid-cols-2 gap-4">
                  <div class="flex flex-col gap-1.5">
                    <label class="text-xs font-medium text-muted-color">Container Name</label>
                    <InputText
                      v-model="currentContainer.name"
                      size="small"
                      fluid
                      @input="handleFieldChange"
                    />
                  </div>

                  <div class="flex flex-col gap-1.5">
                    <label class="text-xs font-medium text-muted-color">Image Pull Policy</label>
                    <Select
                      v-model="currentContainer.imagePullPolicy"
                      :options="['Always', 'IfNotPresent', 'Never']"
                      size="small"
                      fluid
                      @change="handleFieldChange"
                    />
                  </div>
                </div>

                <div class="flex flex-col gap-1.5">
                  <label class="text-xs font-medium text-muted-color">Image</label>
                  <InputText
                    v-model="currentContainer.image"
                    placeholder="e.g. nginx:latest"
                    size="small"
                    fluid
                    @input="handleFieldChange"
                  />
                </div>

                <div class="flex flex-col gap-1.5">
                  <label class="text-xs font-medium text-muted-color">Working Directory</label>
                  <InputText
                    v-model="currentContainer.workingDir"
                    placeholder="e.g. /app"
                    size="small"
                    fluid
                    @input="handleFieldChange"
                  />
                </div>
              </div>
            </div>

            <!-- Resource Allocation -->
            <div class="grid grid-cols-1 md:grid-cols-12 gap-6">
              <div class="md:col-span-4 flex flex-col gap-1">
                <span class="text-xs font-semibold tracking-wider text-muted-color uppercase">
                  Resource Allocation
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
                    title="Command"
                    placeholder="/bin/sh"
                    add-label="Add Cmd"
                    @update:model-value="handleFieldChange"
                  />
                </div>

                <div class="p-4 rounded-lg bg-(--bg-hover)/30 flex flex-col gap-3">
                  <StringListEditor
                    v-model="currentContainer.args"
                    title="Args"
                    placeholder="-c"
                    add-label="Add Arg"
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
                  <p
                    v-if="currentContainer.preservedValueFromEnv.length > 0"
                    class="text-xs text-muted-color mb-1"
                  >
                    ({{ currentContainer.preservedValueFromEnv.length }} valueFrom env var(s)
                    preserved)
                  </p>
                  <KeyValueEditor
                    v-model="currentContainer.env"
                    title="Environment Variables"
                    key-placeholder="NAME"
                    value-placeholder="VALUE"
                    add-label="Add Env"
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
