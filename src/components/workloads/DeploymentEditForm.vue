<script setup lang="ts">
import { ref, watch, toRaw, computed } from 'vue'
import Tabs from 'primevue/tabs'
import TabList from 'primevue/tablist'
import Tab from 'primevue/tab'
import TabPanels from 'primevue/tabpanels'
import TabPanel from 'primevue/tabpanel'
import Select from 'primevue/select'
import InputText from 'primevue/inputtext'
import InputNumber from 'primevue/inputnumber'
import ToggleSwitch from 'primevue/toggleswitch'
import Button from 'primevue/button'

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

// Helper methods for arrays
const addKV = (list: { key: string; value: string }[]) => {
  list.push({ key: '', value: '' })
  handleFieldChange()
}

const removeKV = (list: { key: string; value: string }[], index: number) => {
  list.splice(index, 1)
  handleFieldChange()
}

const addCommandItem = (container: ContainerFormState) => {
  container.command.push('')
  handleFieldChange()
}

const removeCommandItem = (container: ContainerFormState, index: number) => {
  container.command.splice(index, 1)
  handleFieldChange()
}

const addArgItem = (container: ContainerFormState) => {
  container.args.push('')
  handleFieldChange()
}

const removeArgItem = (container: ContainerFormState, index: number) => {
  container.args.splice(index, 1)
  handleFieldChange()
}

const addPortItem = (container: ContainerFormState) => {
  container.ports.push({ name: '', containerPort: 80, protocol: 'TCP' })
  handleFieldChange()
}

const removePortItem = (container: ContainerFormState, index: number) => {
  container.ports.splice(index, 1)
  handleFieldChange()
}
</script>

<template>
  <div class="flex flex-col h-full w-full">
    <Tabs v-model:value="activeTab" class="w-full flex flex-col h-full">
      <TabList class="border-b border-(--border)">
        <Tab value="general" class="text-xs font-medium px-3 py-2">General & Scaling</Tab>
        <Tab value="metadata" class="text-xs font-medium px-3 py-2">Metadata</Tab>
        <Tab value="pod" class="text-xs font-medium px-3 py-2">Pod Spec</Tab>
        <Tab value="containers" class="text-xs font-medium px-3 py-2">Containers</Tab>
      </TabList>

      <TabPanels class="flex-1 overflow-y-auto p-4">
        <!-- GENERAL & SCALING TAB -->
        <TabPanel value="general" class="flex flex-col gap-5">
          <div class="flex flex-col gap-3">
            <h3 class="text-xs font-semibold text-(--text-primary) uppercase tracking-wider">
              Deployment Scaling
            </h3>

            <div class="grid grid-cols-2 gap-4">
              <div class="flex flex-col gap-1.5">
                <label class="text-xs font-medium text-(--text-secondary)">Replicas</label>
                <InputNumber
                  v-model="replicas"
                  :min="0"
                  :max="1000"
                  showButtons
                  buttonLayout="horizontal"
                  class="w-full"
                  inputClass="w-full px-2 py-1.5 bg-(--bg-primary) border border-(--border) text-xs text-(--text-primary) text-center"
                  @update:model-value="handleFieldChange"
                  @change="handleFieldChange"
                />
              </div>

              <div class="flex flex-col gap-1.5 justify-center">
                <label class="text-xs font-medium text-(--text-secondary)">Paused</label>
                <div class="flex items-center gap-2 mt-1">
                  <ToggleSwitch v-model="paused" @change="handleFieldChange" />
                  <span class="text-xs text-(--text-muted)">{{
                    paused ? 'Deployment Paused' : 'Active'
                  }}</span>
                </div>
              </div>
            </div>
          </div>

          <hr class="border-(--border)" />

          <div class="flex flex-col gap-3">
            <h3 class="text-xs font-semibold text-(--text-primary) uppercase tracking-wider">
              Deployment Strategy
            </h3>

            <div class="flex flex-col gap-1.5">
              <label class="text-xs font-medium text-(--text-secondary)">Strategy Type</label>
              <Select
                v-model="strategyType"
                :options="['RollingUpdate', 'Recreate']"
                class="w-full text-xs"
                @change="handleFieldChange"
              />
            </div>

            <div v-if="strategyType === 'RollingUpdate'" class="grid grid-cols-2 gap-4 mt-1">
              <div class="flex flex-col gap-1.5">
                <label class="text-xs font-medium text-(--text-secondary)">Max Surge</label>
                <InputText
                  v-model="maxSurge"
                  placeholder="e.g. 25% or 1"
                  class="px-2.5 py-1.5 bg-(--bg-primary) border border-(--border) rounded-md text-xs text-(--text-primary)"
                  @input="handleFieldChange"
                />
              </div>

              <div class="flex flex-col gap-1.5">
                <label class="text-xs font-medium text-(--text-secondary)">Max Unavailable</label>
                <InputText
                  v-model="maxUnavailable"
                  placeholder="e.g. 25% or 0"
                  class="px-2.5 py-1.5 bg-(--bg-primary) border border-(--border) rounded-md text-xs text-(--text-primary)"
                  @input="handleFieldChange"
                />
              </div>
            </div>
          </div>

          <hr class="border-(--border)" />

          <div class="flex flex-col gap-3">
            <h3 class="text-xs font-semibold text-(--text-primary) uppercase tracking-wider">
              Timing & Limits
            </h3>

            <div class="grid grid-cols-3 gap-3">
              <div class="flex flex-col gap-1.5">
                <label class="text-xs font-medium text-(--text-secondary)">Min Ready Secs</label>
                <InputNumber
                  v-model="minReadySeconds"
                  :min="0"
                  class="w-full"
                  inputClass="w-full px-2 py-1.5 bg-(--bg-primary) border border-(--border) text-xs text-(--text-primary)"
                  @change="handleFieldChange"
                />
              </div>

              <div class="flex flex-col gap-1.5">
                <label class="text-xs font-medium text-(--text-secondary)">Rev History Limit</label>
                <InputNumber
                  v-model="revisionHistoryLimit"
                  :min="0"
                  class="w-full"
                  inputClass="w-full px-2 py-1.5 bg-(--bg-primary) border border-(--border) text-xs text-(--text-primary)"
                  @change="handleFieldChange"
                />
              </div>

              <div class="flex flex-col gap-1.5">
                <label class="text-xs font-medium text-(--text-secondary)">Progress Deadline</label>
                <InputNumber
                  v-model="progressDeadlineSeconds"
                  :min="0"
                  class="w-full"
                  inputClass="w-full px-2 py-1.5 bg-(--bg-primary) border border-(--border) text-xs text-(--text-primary)"
                  @change="handleFieldChange"
                />
              </div>
            </div>
          </div>

          <hr class="border-(--border)" />

          <!-- Read-only Selector -->
          <div class="flex flex-col gap-2">
            <div class="flex items-center justify-between">
              <h3 class="text-xs font-semibold text-(--text-primary) uppercase tracking-wider">
                Selector Labels
              </h3>
              <span class="text-[11px] text-(--text-muted) italic">(Immutable in apps/v1)</span>
            </div>
            <div v-if="selectorLabels.length === 0" class="text-xs text-(--text-muted)">
              None defined
            </div>
            <div class="flex flex-wrap gap-2">
              <span
                v-for="(s, idx) in selectorLabels"
                :key="'sel-' + idx"
                class="px-2 py-1 bg-(--bg-primary) border border-(--border) rounded-md text-xs text-(--text-secondary) font-mono"
              >
                {{ s.key }}={{ s.value }}
              </span>
            </div>
          </div>
        </TabPanel>

        <!-- METADATA TAB -->
        <TabPanel value="metadata" class="flex flex-col gap-5">
          <!-- Deployment Labels -->
          <div class="flex flex-col gap-2">
            <div class="flex items-center justify-between">
              <label class="text-xs font-semibold text-(--text-primary) uppercase tracking-wider"
                >Deployment Labels</label
              >
              <Button
                size="small"
                variant="text"
                icon="pi pi-plus"
                label="Add"
                class="text-xs cursor-pointer"
                @click="addKV(deploymentLabels)"
              />
            </div>
            <div
              v-for="(lbl, idx) in deploymentLabels"
              :key="'dlbl-' + idx"
              class="flex items-center gap-2"
            >
              <InputText
                v-model="lbl.key"
                placeholder="Key"
                class="w-1/2 px-2 py-1 bg-(--bg-primary) border border-(--border) text-xs"
                @input="handleFieldChange"
              />
              <span class="text-(--text-secondary)">=</span>
              <InputText
                v-model="lbl.value"
                placeholder="Value"
                class="w-1/2 px-2 py-1 bg-(--bg-primary) border border-(--border) text-xs"
                @input="handleFieldChange"
              />
              <button
                @click="removeKV(deploymentLabels, idx)"
                class="p-1 text-red-400 hover:text-red-300 cursor-pointer"
              >
                <i class="pi pi-times text-xs"></i>
              </button>
            </div>
          </div>

          <hr class="border-(--border)" />

          <!-- Deployment Annotations -->
          <div class="flex flex-col gap-2">
            <div class="flex items-center justify-between">
              <label class="text-xs font-semibold text-(--text-primary) uppercase tracking-wider"
                >Deployment Annotations</label
              >
              <Button
                size="small"
                variant="text"
                icon="pi pi-plus"
                label="Add"
                class="text-xs cursor-pointer"
                @click="addKV(deploymentAnnotations)"
              />
            </div>
            <div
              v-for="(ann, idx) in deploymentAnnotations"
              :key="'dann-' + idx"
              class="flex items-center gap-2"
            >
              <InputText
                v-model="ann.key"
                placeholder="Key"
                class="w-1/2 px-2 py-1 bg-(--bg-primary) border border-(--border) text-xs"
                @input="handleFieldChange"
              />
              <span class="text-(--text-secondary)">=</span>
              <InputText
                v-model="ann.value"
                placeholder="Value"
                class="w-1/2 px-2 py-1 bg-(--bg-primary) border border-(--border) text-xs"
                @input="handleFieldChange"
              />
              <button
                @click="removeKV(deploymentAnnotations, idx)"
                class="p-1 text-red-400 hover:text-red-300 cursor-pointer"
              >
                <i class="pi pi-times text-xs"></i>
              </button>
            </div>
          </div>

          <hr class="border-(--border)" />

          <!-- Pod Labels -->
          <div class="flex flex-col gap-2">
            <div class="flex items-center justify-between">
              <div>
                <label class="text-xs font-semibold text-(--text-primary) uppercase tracking-wider"
                  >Pod Template Labels</label
                >
                <p class="text-[11px] text-(--text-muted)">Must include all selector labels</p>
              </div>
              <Button
                size="small"
                variant="text"
                icon="pi pi-plus"
                label="Add"
                class="text-xs cursor-pointer"
                @click="addKV(podLabels)"
              />
            </div>
            <div
              v-for="(lbl, idx) in podLabels"
              :key="'plbl-' + idx"
              class="flex items-center gap-2"
            >
              <InputText
                v-model="lbl.key"
                placeholder="Key"
                class="w-1/2 px-2 py-1 bg-(--bg-primary) border border-(--border) text-xs"
                @input="handleFieldChange"
              />
              <span class="text-(--text-secondary)">=</span>
              <InputText
                v-model="lbl.value"
                placeholder="Value"
                class="w-1/2 px-2 py-1 bg-(--bg-primary) border border-(--border) text-xs"
                @input="handleFieldChange"
              />
              <button
                @click="removeKV(podLabels, idx)"
                class="p-1 text-red-400 hover:text-red-300 cursor-pointer"
              >
                <i class="pi pi-times text-xs"></i>
              </button>
            </div>
          </div>

          <hr class="border-(--border)" />

          <!-- Pod Annotations -->
          <div class="flex flex-col gap-2">
            <div class="flex items-center justify-between">
              <label class="text-xs font-semibold text-(--text-primary) uppercase tracking-wider"
                >Pod Template Annotations</label
              >
              <Button
                size="small"
                variant="text"
                icon="pi pi-plus"
                label="Add"
                class="text-xs cursor-pointer"
                @click="addKV(podAnnotations)"
              />
            </div>
            <div
              v-for="(ann, idx) in podAnnotations"
              :key="'pann-' + idx"
              class="flex items-center gap-2"
            >
              <InputText
                v-model="ann.key"
                placeholder="Key"
                class="w-1/2 px-2 py-1 bg-(--bg-primary) border border-(--border) text-xs"
                @input="handleFieldChange"
              />
              <span class="text-(--text-secondary)">=</span>
              <InputText
                v-model="ann.value"
                placeholder="Value"
                class="w-1/2 px-2 py-1 bg-(--bg-primary) border border-(--border) text-xs"
                @input="handleFieldChange"
              />
              <button
                @click="removeKV(podAnnotations, idx)"
                class="p-1 text-red-400 hover:text-red-300 cursor-pointer"
              >
                <i class="pi pi-times text-xs"></i>
              </button>
            </div>
          </div>
        </TabPanel>

        <!-- POD SPEC TAB -->
        <TabPanel value="pod" class="flex flex-col gap-5">
          <div class="flex flex-col gap-3">
            <h3 class="text-xs font-semibold text-(--text-primary) uppercase tracking-wider">
              Pod Execution Settings
            </h3>

            <div class="flex flex-col gap-1.5">
              <label class="text-xs font-medium text-(--text-secondary)"
                >Service Account Name</label
              >
              <InputText
                v-model="serviceAccountName"
                placeholder="e.g. default"
                class="px-2.5 py-1.5 bg-(--bg-primary) border border-(--border) rounded-md text-xs text-(--text-primary)"
                @input="handleFieldChange"
              />
            </div>

            <div class="grid grid-cols-2 gap-4">
              <div class="flex flex-col gap-1.5">
                <label class="text-xs font-medium text-(--text-secondary)">Restart Policy</label>
                <Select
                  v-model="restartPolicy"
                  :options="['Always', 'OnFailure', 'Never']"
                  class="w-full text-xs"
                  @change="handleFieldChange"
                />
              </div>

              <div class="flex flex-col gap-1.5">
                <label class="text-xs font-medium text-(--text-secondary)"
                  >Termination Grace Period (s)</label
                >
                <InputNumber
                  v-model="terminationGracePeriodSeconds"
                  :min="0"
                  class="w-full"
                  inputClass="w-full px-2 py-1.5 bg-(--bg-primary) border border-(--border) text-xs text-(--text-primary)"
                  @change="handleFieldChange"
                />
              </div>
            </div>
          </div>

          <hr class="border-(--border)" />

          <!-- Node Selector -->
          <div class="flex flex-col gap-2">
            <div class="flex items-center justify-between">
              <div>
                <label class="text-xs font-semibold text-(--text-primary) uppercase tracking-wider"
                  >Node Selector</label
                >
                <p class="text-[11px] text-(--text-muted)">Constraints for target node labels</p>
              </div>
              <Button
                size="small"
                variant="text"
                icon="pi pi-plus"
                label="Add"
                class="text-xs cursor-pointer"
                @click="addKV(nodeSelector)"
              />
            </div>
            <div
              v-for="(node, idx) in nodeSelector"
              :key="'node-' + idx"
              class="flex items-center gap-2"
            >
              <InputText
                v-model="node.key"
                placeholder="Key"
                class="w-1/2 px-2 py-1 bg-(--bg-primary) border border-(--border) text-xs"
                @input="handleFieldChange"
              />
              <span class="text-(--text-secondary)">=</span>
              <InputText
                v-model="node.value"
                placeholder="Value"
                class="w-1/2 px-2 py-1 bg-(--bg-primary) border border-(--border) text-xs"
                @input="handleFieldChange"
              />
              <button
                @click="removeKV(nodeSelector, idx)"
                class="p-1 text-red-400 hover:text-red-300 cursor-pointer"
              >
                <i class="pi pi-times text-xs"></i>
              </button>
            </div>
          </div>
        </TabPanel>

        <!-- CONTAINERS TAB -->
        <TabPanel value="containers" class="flex flex-col gap-5">
          <!-- Container selector if multiple -->
          <div
            v-if="containers.length > 1"
            class="flex items-center gap-2 border-b border-(--border) pb-2"
          >
            <span class="text-xs font-medium text-(--text-secondary)">Container:</span>
            <button
              v-for="(c, idx) in containers"
              :key="'cbtn-' + idx"
              @click="activeContainerIndex = idx"
              class="px-2.5 py-1 text-xs rounded-md transition-colors cursor-pointer"
              :class="
                activeContainerIndex === idx
                  ? 'bg-blue-600 text-white font-medium'
                  : 'bg-(--bg-primary) text-(--text-secondary) hover:bg-(--bg-hover)'
              "
            >
              {{ c.name || `Container ${idx + 1}` }}
            </button>
          </div>

          <div v-if="currentContainer" class="flex flex-col gap-5">
            <!-- Basic Info -->
            <div class="flex flex-col gap-3">
              <h3 class="text-xs font-semibold text-(--text-primary) uppercase tracking-wider">
                Basic Info ({{ currentContainer.name || 'Container' }})
              </h3>

              <div class="grid grid-cols-2 gap-4">
                <div class="flex flex-col gap-1.5">
                  <label class="text-xs font-medium text-(--text-secondary)">Container Name</label>
                  <InputText
                    v-model="currentContainer.name"
                    class="px-2.5 py-1.5 bg-(--bg-primary) border border-(--border) rounded-md text-xs text-(--text-primary)"
                    @input="handleFieldChange"
                  />
                </div>

                <div class="flex flex-col gap-1.5">
                  <label class="text-xs font-medium text-(--text-secondary)"
                    >Image Pull Policy</label
                  >
                  <Select
                    v-model="currentContainer.imagePullPolicy"
                    :options="['Always', 'IfNotPresent', 'Never']"
                    class="w-full text-xs"
                    @change="handleFieldChange"
                  />
                </div>
              </div>

              <div class="flex flex-col gap-1.5">
                <label class="text-xs font-medium text-(--text-secondary)">Image</label>
                <InputText
                  v-model="currentContainer.image"
                  placeholder="e.g. nginx:latest"
                  class="px-2.5 py-1.5 bg-(--bg-primary) border border-(--border) rounded-md text-xs text-(--text-primary)"
                  @input="handleFieldChange"
                />
              </div>

              <div class="flex flex-col gap-1.5">
                <label class="text-xs font-medium text-(--text-secondary)">Working Directory</label>
                <InputText
                  v-model="currentContainer.workingDir"
                  placeholder="e.g. /app"
                  class="px-2.5 py-1.5 bg-(--bg-primary) border border-(--border) rounded-md text-xs text-(--text-primary)"
                  @input="handleFieldChange"
                />
              </div>
            </div>

            <hr class="border-(--border)" />

            <!-- Resource Limits & Requests -->
            <div class="flex flex-col gap-3">
              <h3 class="text-xs font-semibold text-(--text-primary) uppercase tracking-wider">
                Resources
              </h3>

              <div class="grid grid-cols-2 gap-4">
                <!-- Requests -->
                <div
                  class="flex flex-col gap-2 p-3 bg-(--bg-primary) border border-(--border) rounded-md"
                >
                  <span class="text-xs font-semibold text-(--text-secondary)">Requests</span>
                  <div class="flex flex-col gap-1.5">
                    <label class="text-[11px] text-(--text-muted)">CPU (e.g. 100m)</label>
                    <InputText
                      v-model="currentContainer.cpuRequest"
                      placeholder="100m"
                      class="px-2 py-1 bg-(--bg-card) border border-(--border) text-xs"
                      @input="handleFieldChange"
                    />
                  </div>
                  <div class="flex flex-col gap-1.5">
                    <label class="text-[11px] text-(--text-muted)">Memory (e.g. 128Mi)</label>
                    <InputText
                      v-model="currentContainer.memoryRequest"
                      placeholder="128Mi"
                      class="px-2 py-1 bg-(--bg-card) border border-(--border) text-xs"
                      @input="handleFieldChange"
                    />
                  </div>
                </div>

                <!-- Limits -->
                <div
                  class="flex flex-col gap-2 p-3 bg-(--bg-primary) border border-(--border) rounded-md"
                >
                  <span class="text-xs font-semibold text-(--text-secondary)">Limits</span>
                  <div class="flex flex-col gap-1.5">
                    <label class="text-[11px] text-(--text-muted)">CPU (e.g. 500m)</label>
                    <InputText
                      v-model="currentContainer.cpuLimit"
                      placeholder="500m"
                      class="px-2 py-1 bg-(--bg-card) border border-(--border) text-xs"
                      @input="handleFieldChange"
                    />
                  </div>
                  <div class="flex flex-col gap-1.5">
                    <label class="text-[11px] text-(--text-muted)">Memory (e.g. 512Mi)</label>
                    <InputText
                      v-model="currentContainer.memoryLimit"
                      placeholder="512Mi"
                      class="px-2 py-1 bg-(--bg-card) border border-(--border) text-xs"
                      @input="handleFieldChange"
                    />
                  </div>
                </div>
              </div>
            </div>

            <hr class="border-(--border)" />

            <!-- Command & Args -->
            <div class="flex flex-col gap-4">
              <!-- Command -->
              <div class="flex flex-col gap-2">
                <div class="flex items-center justify-between">
                  <label
                    class="text-xs font-semibold text-(--text-primary) uppercase tracking-wider"
                    >Command</label
                  >
                  <Button
                    size="small"
                    variant="text"
                    icon="pi pi-plus"
                    label="Add Cmd"
                    class="text-xs cursor-pointer"
                    @click="addCommandItem(currentContainer)"
                  />
                </div>
                <div
                  v-for="(_, cIdx) in currentContainer.command"
                  :key="'cmd-' + cIdx"
                  class="flex items-center gap-2"
                >
                  <InputText
                    v-model="currentContainer.command[cIdx]"
                    placeholder="/bin/sh"
                    class="w-full px-2 py-1 bg-(--bg-primary) border border-(--border) text-xs"
                    @input="handleFieldChange"
                  />
                  <button
                    @click="removeCommandItem(currentContainer, cIdx)"
                    class="p-1 text-red-400 hover:text-red-300 cursor-pointer"
                  >
                    <i class="pi pi-times text-xs"></i>
                  </button>
                </div>
              </div>

              <!-- Args -->
              <div class="flex flex-col gap-2">
                <div class="flex items-center justify-between">
                  <label
                    class="text-xs font-semibold text-(--text-primary) uppercase tracking-wider"
                    >Args</label
                  >
                  <Button
                    size="small"
                    variant="text"
                    icon="pi pi-plus"
                    label="Add Arg"
                    class="text-xs cursor-pointer"
                    @click="addArgItem(currentContainer)"
                  />
                </div>
                <div
                  v-for="(_, aIdx) in currentContainer.args"
                  :key="'arg-' + aIdx"
                  class="flex items-center gap-2"
                >
                  <InputText
                    v-model="currentContainer.args[aIdx]"
                    placeholder="-c"
                    class="w-full px-2 py-1 bg-(--bg-primary) border border-(--border) text-xs"
                    @input="handleFieldChange"
                  />
                  <button
                    @click="removeArgItem(currentContainer, aIdx)"
                    class="p-1 text-red-400 hover:text-red-300 cursor-pointer"
                  >
                    <i class="pi pi-times text-xs"></i>
                  </button>
                </div>
              </div>
            </div>

            <hr class="border-(--border)" />

            <!-- Environment Variables -->
            <div class="flex flex-col gap-2">
              <div class="flex items-center justify-between">
                <div>
                  <label
                    class="text-xs font-semibold text-(--text-primary) uppercase tracking-wider"
                    >Environment Variables</label
                  >
                  <p
                    v-if="currentContainer.preservedValueFromEnv.length > 0"
                    class="text-[11px] text-(--text-muted)"
                  >
                    ({{ currentContainer.preservedValueFromEnv.length }} valueFrom env var(s)
                    preserved)
                  </p>
                </div>
                <Button
                  size="small"
                  variant="text"
                  icon="pi pi-plus"
                  label="Add Env"
                  class="text-xs cursor-pointer"
                  @click="addKV(currentContainer.env)"
                />
              </div>
              <div
                v-for="(e, eIdx) in currentContainer.env"
                :key="'cenv-' + eIdx"
                class="flex items-center gap-2"
              >
                <InputText
                  v-model="e.key"
                  placeholder="NAME"
                  class="w-1/2 px-2 py-1 bg-(--bg-primary) border border-(--border) text-xs font-mono"
                  @input="handleFieldChange"
                />
                <span class="text-(--text-secondary)">=</span>
                <InputText
                  v-model="e.value"
                  placeholder="VALUE"
                  class="w-1/2 px-2 py-1 bg-(--bg-primary) border border-(--border) text-xs font-mono"
                  @input="handleFieldChange"
                />
                <button
                  @click="removeKV(currentContainer.env, eIdx)"
                  class="p-1 text-red-400 hover:text-red-300 cursor-pointer"
                >
                  <i class="pi pi-times text-xs"></i>
                </button>
              </div>
            </div>

            <hr class="border-(--border)" />

            <!-- Container Ports -->
            <div class="flex flex-col gap-2">
              <div class="flex items-center justify-between">
                <label class="text-xs font-semibold text-(--text-primary) uppercase tracking-wider"
                  >Container Ports</label
                >
                <Button
                  size="small"
                  variant="text"
                  icon="pi pi-plus"
                  label="Add Port"
                  class="text-xs cursor-pointer"
                  @click="addPortItem(currentContainer)"
                />
              </div>
              <div
                v-for="(p, pIdx) in currentContainer.ports"
                :key="'cport-' + pIdx"
                class="flex items-center gap-2"
              >
                <InputText
                  v-model="p.name"
                  placeholder="Name"
                  class="w-1/3 px-2 py-1 bg-(--bg-primary) border border-(--border) text-xs"
                  @input="handleFieldChange"
                />
                <InputNumber
                  v-model="p.containerPort"
                  :min="1"
                  :max="65535"
                  class="w-1/3"
                  inputClass="w-full px-2 py-1 bg-(--bg-primary) border border-(--border) text-xs"
                  @change="handleFieldChange"
                />
                <Select
                  v-model="p.protocol"
                  :options="['TCP', 'UDP', 'SCTP']"
                  class="w-1/3 text-xs"
                  @change="handleFieldChange"
                />
                <button
                  @click="removePortItem(currentContainer, pIdx)"
                  class="p-1 text-red-400 hover:text-red-300 cursor-pointer"
                >
                  <i class="pi pi-times text-xs"></i>
                </button>
              </div>
            </div>
          </div>
        </TabPanel>
      </TabPanels>
    </Tabs>
  </div>
</template>
