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
import { computed, ref, toRaw, watch } from 'vue'

import ContainerEnvEditor from '@/components/shared/ContainerEnvEditor.vue'
import ContainerPortsEditor from '@/components/shared/ContainerPortsEditor.vue'
import ContainerResourcesEditor from '@/components/shared/ContainerResourcesEditor.vue'
import KeyValueEditor from '@/components/shared/KeyValueEditor.vue'
import StringListEditor from '@/components/shared/StringListEditor.vue'
import {
  KUBERNETES_RESTART_POLICY,
  KUBERNETES_STATEFULSET_POD_MANAGEMENT_POLICY,
  KUBERNETES_STATEFULSET_UPDATE_STRATEGY,
  KUBERNETES_STATEFULSET_UPDATE_STRATEGIES
} from '@/constants/kubernetes'
import {
  formatContainersToPodSpec,
  isValidMaxUnavailable,
  normalizeStatefulSetSpec,
  parseContainersFromPodSpec,
  parseVolumeClaimTemplates,
  validateStatefulSetForm,
  type StatefulSetContainerFormState
} from '@/utils/statefulSet'
import { isValidK8sLabel, isValidK8sName, isValidPath } from '@/utils/validators'

import type { StatefulSet } from 'kubernetes-types/apps/v1'
import type { PersistentVolumeClaim } from 'kubernetes-types/core/v1'

const props = defineProps<{
  rawData: StatefulSet | null
}>()

const emit = defineEmits<{
  (e: 'update:rawData', value: StatefulSet): void
  (e: 'update:isValid', value: boolean): void
}>()

const activeTab = ref('general')

// General & Scaling state
const replicas = ref<number>(1)
const serviceName = ref<string>('')
const podManagementPolicy = ref<string>(KUBERNETES_STATEFULSET_POD_MANAGEMENT_POLICY.OrderedReady)
const strategyType = ref<string>(KUBERNETES_STATEFULSET_UPDATE_STRATEGY.RollingUpdate)
const partition = ref<number>(0)
const maxUnavailable = ref<string>('1')
const minReadySeconds = ref<number>(0)
const revisionHistoryLimit = ref<number>(10)
const selectorLabels = ref<{ key: string; value: string }[]>([])

// Metadata state
const statefulSetLabels = ref<{ key: string; value: string }[]>([])
const statefulSetAnnotations = ref<{ key: string; value: string }[]>([])
const podLabels = ref<{ key: string; value: string }[]>([])
const podAnnotations = ref<{ key: string; value: string }[]>([])

// Pod spec state
const serviceAccountName = ref<string>('')
const restartPolicy = ref<string>(KUBERNETES_RESTART_POLICY.Always)
const terminationGracePeriodSeconds = ref<number>(30)
const nodeSelector = ref<{ key: string; value: string }[]>([])

// Storage state
const volumeClaimTemplates = ref<PersistentVolumeClaim[]>([])
const pvcRetentionPolicy = ref<{ whenDeleted?: string; whenScaled?: string } | null>(null)

const statefulSetNamespace = computed(() => {
  return props.rawData?.metadata?.namespace ?? ''
})

const formattedVolumeClaims = computed(() => {
  return parseVolumeClaimTemplates(volumeClaimTemplates.value)
})

const containers = ref<StatefulSetContainerFormState[]>([])
const activeContainerIndex = ref<number>(0)

const isFormValid = computed(() => {
  return validateStatefulSetForm({
    serviceName: serviceName.value,
    serviceAccountName: serviceAccountName.value,
    strategyType: strategyType.value,
    maxUnavailable: maxUnavailable.value,
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

const syncFromRawData = (data: StatefulSet | null) => {
  if (!data || isEmitting) return

  const normalized = normalizeStatefulSetSpec(data)
  replicas.value = normalized.replicas
  serviceName.value = normalized.serviceName
  podManagementPolicy.value = normalized.podManagementPolicy
  strategyType.value = normalized.strategyType
  partition.value = normalized.partition
  maxUnavailable.value = normalized.maxUnavailable
  minReadySeconds.value = normalized.minReadySeconds
  revisionHistoryLimit.value = normalized.revisionHistoryLimit
  selectorLabels.value = normalized.selectorLabels

  // Metadata
  statefulSetLabels.value = kvObjectToArray(data.metadata?.labels)
  statefulSetAnnotations.value = kvObjectToArray(data.metadata?.annotations)

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

  // Storage
  volumeClaimTemplates.value = data.spec?.volumeClaimTemplates || []
  pvcRetentionPolicy.value = data.spec?.persistentVolumeClaimRetentionPolicy || null

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
    replicas,
    serviceName,
    strategyType,
    partition,
    maxUnavailable,
    minReadySeconds,
    revisionHistoryLimit,
    serviceAccountName,
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

  const rawObj = JSON.parse(JSON.stringify(toRaw(props.rawData))) as StatefulSet
  if (!rawObj.metadata) rawObj.metadata = {}
  if (!rawObj.spec) {
    rawObj.spec = {
      serviceName: '',
      selector: { matchLabels: {} },
      template: { spec: { containers: [] } }
    }
  }
  const spec = rawObj.spec
  if (!spec.template) spec.template = { spec: { containers: [] } }
  if (!spec.template.metadata) spec.template.metadata = {}
  if (!spec.template.spec) spec.template.spec = { containers: [] }

  // 1. General & Scaling
  spec.replicas = replicas.value
  spec.serviceName = serviceName.value.trim()

  if (!spec.updateStrategy) spec.updateStrategy = {}
  spec.updateStrategy.type = strategyType.value

  if (strategyType.value === KUBERNETES_STATEFULSET_UPDATE_STRATEGY.RollingUpdate) {
    if (!spec.updateStrategy.rollingUpdate) spec.updateStrategy.rollingUpdate = {}
    spec.updateStrategy.rollingUpdate.partition = partition.value

    const trimmedMaxUnavailable = maxUnavailable.value.trim()
    if (trimmedMaxUnavailable && isValidMaxUnavailable(trimmedMaxUnavailable)) {
      const numVal = Number(trimmedMaxUnavailable)
      spec.updateStrategy.rollingUpdate.maxUnavailable =
        !Number.isNaN(numVal) && !trimmedMaxUnavailable.includes('%')
          ? numVal
          : trimmedMaxUnavailable
    } else {
      delete spec.updateStrategy.rollingUpdate.maxUnavailable
    }
  } else {
    delete spec.updateStrategy.rollingUpdate
  }

  spec.minReadySeconds = minReadySeconds.value
  spec.revisionHistoryLimit = revisionHistoryLimit.value

  // 2. Metadata
  rawObj.metadata.labels = kvArrayToObject(statefulSetLabels.value)
  rawObj.metadata.annotations = kvArrayToObject(statefulSetAnnotations.value)
  spec.template.metadata.labels = kvArrayToObject(podLabels.value)
  spec.template.metadata.annotations = kvArrayToObject(podAnnotations.value)

  // 3. Pod Spec
  if (serviceAccountName.value.trim()) {
    spec.template.spec.serviceAccountName = serviceAccountName.value.trim()
  } else {
    delete spec.template.spec.serviceAccountName
  }

  spec.template.spec.restartPolicy = KUBERNETES_RESTART_POLICY.Always
  spec.template.spec.terminationGracePeriodSeconds = terminationGracePeriodSeconds.value

  const nodeSel = kvArrayToObject(nodeSelector.value)
  if (Object.keys(nodeSel).length > 0) {
    spec.template.spec.nodeSelector = nodeSel
  } else {
    delete spec.template.spec.nodeSelector
  }

  // 4. Containers
  spec.template.spec.containers = formatContainersToPodSpec(containers.value)

  emit('update:rawData', rawObj)

  setTimeout(() => {
    isEmitting = false
  }, 50)
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
        <Tab value="storage" class="text-xs font-medium">Storage & PVCs</Tab>
      </TabList>

      <TabPanels class="flex-1 overflow-y-auto pt-6 px-0">
        <!-- GENERAL & SCALING TAB -->
        <TabPanel value="general" class="flex flex-col gap-10 max-w-7xl">
          <!-- Section 1: StatefulSet Scaling -->
          <div class="grid grid-cols-1 md:grid-cols-12 gap-6">
            <div class="md:col-span-4 flex flex-col gap-1">
              <span class="text-xs font-semibold tracking-wider text-muted-color uppercase">
                StatefulSet Scaling
              </span>
              <p class="text-xs text-muted-color leading-relaxed">
                Desired pod replicas, headless governing service, and pod management policy.
              </p>
            </div>
            <div class="md:col-span-8 flex flex-col gap-4">
              <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
                <div class="flex flex-col gap-1.5">
                  <label class="text-xs font-medium text-muted-color">Replicas</label>
                  <InputNumber
                    v-model="replicas"
                    showButtons
                    buttonLayout="horizontal"
                    size="small"
                    :min="0"
                    :max="1000"
                  />
                </div>

                <div class="flex flex-col gap-1.5">
                  <label class="text-xs font-medium text-muted-color">
                    Governing Service Name
                  </label>
                  <InputText
                    v-model="serviceName"
                    placeholder="e.g. headless-svc"
                    :invalid="!serviceName || !isValidK8sName(serviceName)"
                    size="small"
                    fluid
                    class="text-xs font-mono"
                    @input="handleFieldChange"
                  />
                  <span class="text-[11px] text-muted-color">
                    Headless service responsible for the network domain of the StatefulSet.
                  </span>
                </div>
              </div>

              <div class="flex flex-col gap-1.5">
                <label class="text-xs font-medium text-muted-color">Pod Management Policy</label>
                <div class="flex items-center gap-2">
                  <Tag
                    :value="podManagementPolicy"
                    severity="secondary"
                    class="text-xs font-mono"
                  />
                  <span class="text-[11px] text-muted-color">
                    (Immutable after StatefulSet creation)
                  </span>
                </div>
              </div>
            </div>
          </div>

          <!-- Section 2: Update Strategy -->
          <div class="grid grid-cols-1 md:grid-cols-12 gap-6">
            <div class="md:col-span-4 flex flex-col gap-1">
              <span class="text-xs font-semibold tracking-wider text-muted-color uppercase">
                Update Strategy
              </span>
              <p class="text-xs text-muted-color leading-relaxed">
                Strategy employed to update pods when revisions are made to the template.
              </p>
            </div>
            <div class="md:col-span-8 flex flex-col gap-4">
              <div class="flex flex-col gap-1.5">
                <label class="text-xs font-medium text-muted-color">Strategy Type</label>
                <Select
                  v-model="strategyType"
                  :options="KUBERNETES_STATEFULSET_UPDATE_STRATEGIES"
                  size="small"
                  class="w-full md:w-64"
                  @change="handleFieldChange"
                />
              </div>

              <div
                v-if="strategyType === KUBERNETES_STATEFULSET_UPDATE_STRATEGY.RollingUpdate"
                class="grid grid-cols-1 sm:grid-cols-2 gap-4"
              >
                <div class="flex flex-col gap-1.5">
                  <label class="text-xs font-medium text-muted-color">Partition Ordinal</label>
                  <InputNumber
                    v-model="partition"
                    :min="0"
                    size="small"
                    fluid
                    @change="handleFieldChange"
                  />
                  <span class="text-[11px] text-muted-color">
                    Canary barrier: pods from Replicas-1 down to Partition will be updated.
                  </span>
                </div>

                <div class="flex flex-col gap-1.5">
                  <label class="text-xs font-medium text-muted-color">Max Unavailable</label>
                  <InputText
                    v-model="maxUnavailable"
                    placeholder="e.g. 1 or 25%"
                    :invalid="Boolean(maxUnavailable && !isValidMaxUnavailable(maxUnavailable))"
                    size="small"
                    fluid
                    @input="handleFieldChange"
                  />
                  <span class="text-[11px] text-muted-color">
                    Maximum unavailable pods during rolling update (number or percentage).
                  </span>
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
                Minimal ready time and revision history retention limits.
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

          <!-- Section 4: Selector Labels (Read-only) -->
          <div class="grid grid-cols-1 md:grid-cols-12 gap-6">
            <div class="md:col-span-4 flex flex-col gap-1">
              <span class="text-xs font-semibold tracking-wider text-muted-color uppercase">
                Selector Labels
              </span>
              <p class="text-xs text-muted-color leading-relaxed">
                Pod selector labels. Immutable after StatefulSet creation.
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
          <!-- StatefulSet Level -->
          <div class="grid grid-cols-1 md:grid-cols-12 gap-6">
            <div class="md:col-span-4 flex flex-col gap-1">
              <span class="text-xs font-semibold tracking-wider text-muted-color uppercase">
                StatefulSet Metadata
              </span>
              <p class="text-xs text-muted-color leading-relaxed">
                Labels and annotations attached to the StatefulSet resource itself.
              </p>
            </div>
            <div class="md:col-span-8 flex flex-col gap-6">
              <div class="p-4 rounded-lg bg-(--bg-hover)/30 flex flex-col gap-3">
                <KeyValueEditor
                  v-model="statefulSetLabels"
                  title="StatefulSet Labels"
                  add-label="Add Label"
                  @update:model-value="handleFieldChange"
                />
              </div>
              <div class="p-4 rounded-lg bg-(--bg-hover)/30 flex flex-col gap-3">
                <KeyValueEditor
                  v-model="statefulSetAnnotations"
                  title="StatefulSet Annotations"
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
                Labels and annotations propagated to created StatefulSet Pod instances.
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
                      (StatefulSet templates require 'Always')
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
                        Boolean(currentContainer.name && !isValidK8sLabel(currentContainer.name))
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
                      :options="['Always', 'IfNotPresent', 'Never']"
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
                    placeholder="e.g. redis:7.2"
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
                    placeholder="e.g. /app"
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
                    :namespace="statefulSetNamespace"
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

        <!-- STORAGE & PVCs TAB -->
        <TabPanel value="storage" class="flex flex-col gap-10 max-w-7xl">
          <div class="grid grid-cols-1 md:grid-cols-12 gap-6">
            <div class="md:col-span-4 flex flex-col gap-1">
              <span class="text-xs font-semibold tracking-wider text-muted-color uppercase">
                Volume Claim Templates
              </span>
              <p class="text-xs text-muted-color leading-relaxed">
                Persistent volume claims stamped out for each StatefulSet pod replica. These claims
                are immutable after creation.
              </p>
            </div>
            <div class="md:col-span-8 flex flex-col gap-4">
              <div
                v-if="formattedVolumeClaims.length === 0"
                class="p-6 rounded-lg bg-(--bg-hover)/20 flex items-center justify-center text-xs text-muted-color"
              >
                No Volume Claim Templates defined for this StatefulSet.
              </div>
              <div v-else class="flex flex-col gap-3">
                <div
                  v-for="(pvc, idx) in formattedVolumeClaims"
                  :key="'pvc-' + idx"
                  class="p-4 rounded-lg bg-(--bg-hover)/30 flex flex-col gap-3 border border-(--border)/40"
                >
                  <div class="flex items-center justify-between">
                    <div class="flex items-center gap-2">
                      <span class="text-xs font-bold text-primary font-mono">{{ pvc.name }}</span>
                      <Tag
                        :value="pvc.storageClassName"
                        severity="secondary"
                        class="text-[11px] font-mono"
                      />
                    </div>
                    <span class="text-xs font-semibold text-primary font-mono">
                      {{ pvc.storageCapacity }}
                    </span>
                  </div>
                  <div class="grid grid-cols-2 gap-2 text-xs text-muted-color">
                    <div>
                      <span class="text-muted-color/80">Access Modes: </span>
                      <span class="font-mono text-primary">{{ pvc.accessModes }}</span>
                    </div>
                    <div>
                      <span class="text-muted-color/80">Volume Mode: </span>
                      <span class="font-mono text-primary">{{ pvc.volumeMode }}</span>
                    </div>
                  </div>
                </div>
              </div>

              <!-- PVC Retention Policy -->
              <div
                v-if="pvcRetentionPolicy"
                class="p-4 rounded-lg bg-(--bg-hover)/20 flex flex-col gap-2 text-xs mt-2"
              >
                <span class="font-semibold text-muted-color uppercase tracking-wider text-[11px]">
                  PVC Retention Policy
                </span>
                <div class="grid grid-cols-2 gap-2 text-muted-color">
                  <div>
                    When Deleted:
                    <span class="font-mono text-primary">
                      {{ pvcRetentionPolicy.whenDeleted || 'Retain' }}
                    </span>
                  </div>
                  <div>
                    When Scaled:
                    <span class="font-mono text-primary">
                      {{ pvcRetentionPolicy.whenScaled || 'Retain' }}
                    </span>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </TabPanel>
      </TabPanels>
    </Tabs>
  </div>
</template>
