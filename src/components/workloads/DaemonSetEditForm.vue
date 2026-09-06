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
import Tag from 'primevue/tag'
import ToggleSwitch from 'primevue/toggleswitch'
import { computed, ref, toRaw, watch } from 'vue'

import ContainerEnvEditor from '@/components/shared/ContainerEnvEditor.vue'
import ContainerPortsEditor from '@/components/shared/ContainerPortsEditor.vue'
import ContainerResourcesEditor from '@/components/shared/ContainerResourcesEditor.vue'
import KeyValueEditor from '@/components/shared/KeyValueEditor.vue'
import StringListEditor from '@/components/shared/StringListEditor.vue'
import {
  KUBERNETES_DAEMONSET_UPDATE_STRATEGY,
  KUBERNETES_DAEMONSET_UPDATE_STRATEGIES,
  KUBERNETES_DNS_POLICIES,
  KUBERNETES_IMAGE_PULL_POLICIES,
  KUBERNETES_RESTART_POLICY
} from '@/constants/kubernetes'
import {
  formatContainersToPodSpec,
  isValidMaxSurge,
  isValidMaxUnavailable,
  normalizeDaemonSetSpec,
  parseContainersFromPodSpec,
  validateDaemonSetForm,
  type DaemonSetContainerFormState
} from '@/utils/daemonSet'
import { isValidK8sLabel, isValidK8sName, isValidPath } from '@/utils/validators'

import type { DaemonSet } from 'kubernetes-types/apps/v1'

const props = defineProps<{
  rawData: DaemonSet | null
}>()

const emit = defineEmits<{
  (e: 'update:rawData', value: DaemonSet): void
  (e: 'update:isValid', value: boolean): void
}>()

const activeTab = ref('general')

// General & Strategy state
const strategyType = ref<string>(KUBERNETES_DAEMONSET_UPDATE_STRATEGY.RollingUpdate)
const maxUnavailable = ref<string>('1')
const maxSurge = ref<string>('0')
const minReadySeconds = ref<number>(0)
const revisionHistoryLimit = ref<number>(10)
const selectorLabels = ref<{ key: string; value: string }[]>([])

// Metadata state
const daemonSetLabels = ref<{ key: string; value: string }[]>([])
const daemonSetAnnotations = ref<{ key: string; value: string }[]>([])
const podLabels = ref<{ key: string; value: string }[]>([])
const podAnnotations = ref<{ key: string; value: string }[]>([])

// Pod spec state
const serviceAccountName = ref<string>('')
const restartPolicy = ref<string>(KUBERNETES_RESTART_POLICY.Always)
const terminationGracePeriodSeconds = ref<number>(30)
const nodeSelector = ref<{ key: string; value: string }[]>([])
const hostNetwork = ref<boolean>(false)
const hostPID = ref<boolean>(false)
const hostIPC = ref<boolean>(false)
const dnsPolicy = ref<string>('ClusterFirst')

const DNS_POLICIES = KUBERNETES_DNS_POLICIES

const daemonSetNamespace = computed(() => {
  return props.rawData?.metadata?.namespace ?? ''
})

const containers = ref<DaemonSetContainerFormState[]>([])
const activeContainerIndex = ref<number>(0)

const currentContainer = computed(() => {
  return containers.value[activeContainerIndex.value] || null
})

const isFormValid = computed(() => {
  return validateDaemonSetForm({
    serviceAccountName: serviceAccountName.value,
    strategyType: strategyType.value,
    maxUnavailable: maxUnavailable.value,
    maxSurge: maxSurge.value,
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

const syncFromRawData = (data: DaemonSet | null) => {
  if (!data || isEmitting) return

  const normalized = normalizeDaemonSetSpec(data)
  strategyType.value = normalized.strategyType
  maxUnavailable.value = normalized.maxUnavailable
  maxSurge.value = normalized.maxSurge
  minReadySeconds.value = normalized.minReadySeconds
  revisionHistoryLimit.value = normalized.revisionHistoryLimit
  selectorLabels.value = normalized.selectorLabels

  // Metadata
  daemonSetLabels.value = kvObjectToArray(data.metadata?.labels)
  daemonSetAnnotations.value = kvObjectToArray(data.metadata?.annotations)

  // Pod Template Metadata
  const template = data.spec?.template
  podLabels.value = kvObjectToArray(template?.metadata?.labels)
  podAnnotations.value = kvObjectToArray(template?.metadata?.annotations)

  // Pod Spec
  const podSpec = template?.spec
  serviceAccountName.value = podSpec?.serviceAccountName || ''
  restartPolicy.value = podSpec?.restartPolicy || KUBERNETES_RESTART_POLICY.Always
  terminationGracePeriodSeconds.value =
    typeof podSpec?.terminationGracePeriodSeconds === 'number'
      ? podSpec.terminationGracePeriodSeconds
      : 30
  nodeSelector.value = kvObjectToArray(podSpec?.nodeSelector)
  hostNetwork.value = Boolean(podSpec?.hostNetwork)
  hostPID.value = Boolean(podSpec?.hostPID)
  hostIPC.value = Boolean(podSpec?.hostIPC)
  dnsPolicy.value = podSpec?.dnsPolicy || 'ClusterFirst'

  // Containers
  containers.value = parseContainersFromPodSpec(podSpec?.containers)

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
  emitUpdate()
}

watch(
  [
    strategyType,
    maxUnavailable,
    maxSurge,
    minReadySeconds,
    revisionHistoryLimit,
    serviceAccountName,
    terminationGracePeriodSeconds,
    hostNetwork,
    hostPID,
    hostIPC,
    dnsPolicy
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

  const rawObj = JSON.parse(JSON.stringify(toRaw(props.rawData))) as DaemonSet
  if (!rawObj.metadata) rawObj.metadata = {}
  if (!rawObj.spec) {
    rawObj.spec = {
      selector: { matchLabels: {} },
      template: { spec: { containers: [] } }
    }
  }
  const spec = rawObj.spec
  if (!spec.template) spec.template = { spec: { containers: [] } }
  if (!spec.template.metadata) spec.template.metadata = {}
  if (!spec.template.spec) spec.template.spec = { containers: [] }

  // 1. General & Strategy
  if (!spec.updateStrategy) spec.updateStrategy = {}
  spec.updateStrategy.type = strategyType.value

  if (strategyType.value === KUBERNETES_DAEMONSET_UPDATE_STRATEGY.RollingUpdate) {
    if (!spec.updateStrategy.rollingUpdate) spec.updateStrategy.rollingUpdate = {}
    const trimmedUnavailable = maxUnavailable.value.trim()
    if (trimmedUnavailable && isValidMaxUnavailable(trimmedUnavailable)) {
      const numVal = Number(trimmedUnavailable)
      spec.updateStrategy.rollingUpdate.maxUnavailable =
        !Number.isNaN(numVal) && !trimmedUnavailable.includes('%') ? numVal : trimmedUnavailable
    } else {
      delete spec.updateStrategy.rollingUpdate.maxUnavailable
    }

    const trimmedSurge = maxSurge.value.trim()
    if (trimmedSurge && isValidMaxSurge(trimmedSurge)) {
      const numVal = Number(trimmedSurge)
      spec.updateStrategy.rollingUpdate.maxSurge =
        !Number.isNaN(numVal) && !trimmedSurge.includes('%') ? numVal : trimmedSurge
    } else {
      delete spec.updateStrategy.rollingUpdate.maxSurge
    }
  } else {
    delete spec.updateStrategy.rollingUpdate
  }

  spec.minReadySeconds = minReadySeconds.value
  spec.revisionHistoryLimit = revisionHistoryLimit.value

  // 2. Metadata
  rawObj.metadata.labels = kvArrayToObject(daemonSetLabels.value)
  rawObj.metadata.annotations = kvArrayToObject(daemonSetAnnotations.value)
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
          <!-- Section 1: Update Strategy -->
          <div class="grid grid-cols-1 md:grid-cols-12 gap-6">
            <div class="md:col-span-4 flex flex-col gap-1">
              <span class="text-xs font-semibold tracking-wider text-muted-color uppercase">
                Update Strategy
              </span>
              <p class="text-xs text-muted-color leading-relaxed">
                Strategy used to replace old DaemonSet pods with new ones. RollingUpdate replaces
                pods gradually across nodes.
              </p>
            </div>
            <div class="md:col-span-8 flex flex-col gap-4">
              <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
                <div class="flex flex-col gap-1.5">
                  <label class="text-xs font-medium text-muted-color">Strategy Type</label>
                  <Select
                    v-model="strategyType"
                    :options="KUBERNETES_DAEMONSET_UPDATE_STRATEGIES"
                    size="small"
                    fluid
                    class="text-xs"
                    @change="handleFieldChange"
                  />
                </div>
              </div>

              <!-- Rolling Update Configuration -->
              <div
                v-if="strategyType === KUBERNETES_DAEMONSET_UPDATE_STRATEGY.RollingUpdate"
                class="grid grid-cols-1 sm:grid-cols-2 gap-4 p-4 rounded-lg bg-(--bg-hover)/30"
              >
                <div class="flex flex-col gap-1.5">
                  <label class="text-xs font-medium text-muted-color">Max Unavailable</label>
                  <InputText
                    v-model="maxUnavailable"
                    placeholder="e.g. 1 or 25%"
                    :invalid="Boolean(maxUnavailable && !isValidMaxUnavailable(maxUnavailable))"
                    size="small"
                    fluid
                    class="text-xs"
                    @input="handleFieldChange"
                  />
                  <span class="text-[11px] text-muted-color">
                    Maximum unavailable pods during rolling update (number or percentage).
                  </span>
                </div>

                <div class="flex flex-col gap-1.5">
                  <label class="text-xs font-medium text-muted-color">Max Surge</label>
                  <InputText
                    v-model="maxSurge"
                    placeholder="e.g. 0 or 25%"
                    :invalid="Boolean(maxSurge && !isValidMaxSurge(maxSurge))"
                    size="small"
                    fluid
                    class="text-xs"
                    @input="handleFieldChange"
                  />
                  <span class="text-[11px] text-muted-color">
                    Maximum extra pods created above total desired nodes (Kubernetes 1.22+).
                  </span>
                </div>
              </div>
            </div>
          </div>

          <!-- Section 2: Timing & Limits -->
          <div class="grid grid-cols-1 md:grid-cols-12 gap-6">
            <div class="md:col-span-4 flex flex-col gap-1">
              <span class="text-xs font-semibold tracking-wider text-muted-color uppercase">
                Timing & Limits
              </span>
              <p class="text-xs text-muted-color leading-relaxed">
                Minimum ready duration and revision history retention limits.
              </p>
            </div>
            <div class="md:col-span-8 grid grid-cols-1 sm:grid-cols-2 gap-4">
              <div class="flex flex-col gap-1.5">
                <label class="text-xs font-medium text-muted-color">Min Ready Seconds</label>
                <InputNumber
                  v-model="minReadySeconds"
                  :min="0"
                  size="small"
                  fluid
                  @change="handleFieldChange"
                />
              </div>

              <div class="flex flex-col gap-1.5">
                <label class="text-xs font-medium text-muted-color">Revision History Limit</label>
                <InputNumber
                  v-model="revisionHistoryLimit"
                  :min="0"
                  size="small"
                  fluid
                  @change="handleFieldChange"
                />
              </div>
            </div>
          </div>

          <!-- Section 3: Selector Labels (Read-only) -->
          <div class="grid grid-cols-1 md:grid-cols-12 gap-6">
            <div class="md:col-span-4 flex flex-col gap-1">
              <span class="text-xs font-semibold tracking-wider text-muted-color uppercase">
                Selector Labels
              </span>
              <p class="text-xs text-muted-color leading-relaxed">
                Pod selector labels. Immutable after DaemonSet creation.
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
          <!-- DaemonSet Level -->
          <div class="grid grid-cols-1 md:grid-cols-12 gap-6">
            <div class="md:col-span-4 flex flex-col gap-1">
              <span class="text-xs font-semibold tracking-wider text-muted-color uppercase">
                DaemonSet Metadata
              </span>
              <p class="text-xs text-muted-color leading-relaxed">
                Labels and annotations attached to the DaemonSet resource itself.
              </p>
            </div>
            <div class="md:col-span-8 flex flex-col gap-6">
              <div class="p-4 rounded-lg bg-(--bg-hover)/30 flex flex-col gap-3">
                <KeyValueEditor
                  v-model="daemonSetLabels"
                  title="DaemonSet Labels"
                  add-label="Add Label"
                  @update:model-value="handleFieldChange"
                />
              </div>
              <div class="p-4 rounded-lg bg-(--bg-hover)/30 flex flex-col gap-3">
                <KeyValueEditor
                  v-model="daemonSetAnnotations"
                  title="DaemonSet Annotations"
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
                Labels and annotations propagated to created DaemonSet Pod instances.
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
        <TabPanel value="pod" class="flex flex-col gap-10 max-w-7xl">
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
                  :invalid="Boolean(serviceAccountName && !isValidK8sName(serviceAccountName))"
                  size="small"
                  fluid
                  class="text-xs"
                  @input="handleFieldChange"
                />
              </div>

              <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
                <div class="flex flex-col gap-1.5">
                  <label class="text-xs font-medium text-muted-color">Restart Policy</label>
                  <div class="flex items-center gap-2">
                    <Tag
                      :value="KUBERNETES_RESTART_POLICY.Always"
                      severity="secondary"
                      class="text-xs font-mono"
                    />
                    <span class="text-[11px] text-muted-color">
                      (DaemonSet templates require 'Always')
                    </span>
                  </div>
                </div>

                <div class="flex flex-col gap-1.5">
                  <label class="text-xs font-medium text-muted-color">
                    Termination Grace Period (s)
                  </label>
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

          <!-- Host Namespace & Networking -->
          <div class="grid grid-cols-1 md:grid-cols-12 gap-6">
            <div class="md:col-span-4 flex flex-col gap-1">
              <span class="text-xs font-semibold tracking-wider text-muted-color uppercase">
                Host Namespace & Networking
              </span>
              <p class="text-xs text-muted-color leading-relaxed">
                Host-level namespaces and DNS policies frequently utilized by node-level daemons.
              </p>
            </div>
            <div class="md:col-span-8 flex flex-col gap-4">
              <div class="grid grid-cols-1 sm:grid-cols-3 gap-4">
                <div class="flex items-center justify-between p-3 rounded-lg bg-(--bg-hover)/30">
                  <div class="flex flex-col">
                    <span class="text-xs font-medium text-primary">Host Network</span>
                    <span class="text-[11px] text-muted-color">Use node network namespace</span>
                  </div>
                  <ToggleSwitch v-model="hostNetwork" @change="handleFieldChange" />
                </div>

                <div class="flex items-center justify-between p-3 rounded-lg bg-(--bg-hover)/30">
                  <div class="flex flex-col">
                    <span class="text-xs font-medium text-primary">Host PID</span>
                    <span class="text-[11px] text-muted-color">Use node process namespace</span>
                  </div>
                  <ToggleSwitch v-model="hostPID" @change="handleFieldChange" />
                </div>

                <div class="flex items-center justify-between p-3 rounded-lg bg-(--bg-hover)/30">
                  <div class="flex flex-col">
                    <span class="text-xs font-medium text-primary">Host IPC</span>
                    <span class="text-[11px] text-muted-color">Use node IPC namespace</span>
                  </div>
                  <ToggleSwitch v-model="hostIPC" @change="handleFieldChange" />
                </div>
              </div>

              <div class="flex flex-col gap-1.5">
                <label class="text-xs font-medium text-muted-color">DNS Policy</label>
                <Select
                  v-model="dnsPolicy"
                  :options="DNS_POLICIES"
                  size="small"
                  fluid
                  class="text-xs"
                  @change="handleFieldChange"
                />
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
                Node selector labels restricting which nodes this DaemonSet runs on.
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
                  Basic container identifier, image reference, and image pull policy.
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
                    placeholder="e.g. fluent/fluentd:v1.16"
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
                    placeholder="e.g. /home/fluent"
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
                  <ContainerEnvEditor
                    v-model="currentContainer.environments"
                    :namespace="daemonSetNamespace"
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
