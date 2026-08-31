<script setup lang="ts">
import { kubernetesService } from '@/services/kubernetesService'
import { events } from '@/services/nativeService'
import { useKubernetesStore } from '@/stores/kubernetesStore'
import { OrbitEvents } from '@/types/events'
import {
  KUBERNETES_RESOURCE_KIND,
  KUBERNETES_SERVICE_TYPE,
  type KubernetesServiceType
} from '@/constants/kubernetes'
import { isValidK8sLabel, isValidK8sName, isValidPort } from '@/utils/validators'
import type { Service, ServicePort } from 'kubernetes-types/core/v1'
import Button from 'primevue/button'
import InputNumber from 'primevue/inputnumber'
import InputText from 'primevue/inputtext'
import Select from 'primevue/select'
import { computed, inject, onMounted, onUnmounted, ref, type Ref } from 'vue'

interface CreateServiceDialogData {
  initialNamespace?: string
}

const dialogRef = inject<
  | Ref<{
      data?: CreateServiceDialogData
      close: () => void
    }>
  | undefined
>('dialogRef')

const k8sStore = useKubernetesStore()
const namespaceOptions = computed(() => {
  const list = k8sStore.namespaceList.map((ns) => ns.name)
  if (list.length === 0) return ['default']
  return list
})

const serviceTypeOptions: KubernetesServiceType[] = [
  KUBERNETES_SERVICE_TYPE.ClusterIP,
  KUBERNETES_SERVICE_TYPE.NodePort,
  KUBERNETES_SERVICE_TYPE.LoadBalancer
]

const name = ref('')
const namespace = ref('default')
const serviceType = ref<KubernetesServiceType>(KUBERNETES_SERVICE_TYPE.ClusterIP)
const selectorApp = ref('')
const port = ref<number>(80)
const targetPort = ref<number | null>(null)
const nodePort = ref<number | null>(null)
const isCreating = ref(false)
const submittedName = ref('')

const handleCommandSucceeded = (payload: { message: string }) => {
  if (isCreating.value && submittedName.value && payload.message.includes(submittedName.value)) {
    isCreating.value = false
    submittedName.value = ''
    dialogRef?.value?.close()
  }
}

const handleErrorOccurred = () => {
  if (isCreating.value) {
    isCreating.value = false
    submittedName.value = ''
  }
}

onMounted(() => {
  events.on(OrbitEvents.CommandSucceeded, handleCommandSucceeded)
  events.on(OrbitEvents.ErrorOccurred, handleErrorOccurred)

  if (dialogRef?.value?.data?.initialNamespace) {
    namespace.value = dialogRef.value.data.initialNamespace
  } else if (namespaceOptions.value.length > 0) {
    namespace.value = namespaceOptions.value.includes('default')
      ? 'default'
      : (namespaceOptions.value[0] ?? 'default')
  }
})

onUnmounted(() => {
  events.off(OrbitEvents.CommandSucceeded, handleCommandSucceeded)
  events.off(OrbitEvents.ErrorOccurred, handleErrorOccurred)
})

const nameErrorMessage = computed(() => {
  const trimmed = name.value.trim()
  if (!trimmed) return null
  if (!isValidK8sName(trimmed)) {
    return 'Name must be a valid DNS-1123 subdomain (lowercase letters, numbers, hyphens, dots).'
  }
  const exists = k8sStore.services.some(
    (svc) => svc.namespace === namespace.value && svc.name.toLowerCase() === trimmed.toLowerCase()
  )
  if (exists) {
    return `A Service named "${trimmed}" already exists in namespace "${namespace.value}".`
  }
  return null
})

const selectorAppErrorMessage = computed(() => {
  const trimmed = selectorApp.value.trim()
  if (!trimmed) return null
  if (!isValidK8sLabel(trimmed)) {
    return 'Target App selector must be a valid Kubernetes label value.'
  }
  return null
})

const portErrorMessage = computed(() => {
  if (port.value === null || port.value === undefined) {
    return 'Port is required.'
  }
  if (!isValidPort(port.value)) {
    return 'Port must be an integer between 1 and 65535.'
  }
  return null
})

const targetPortErrorMessage = computed(() => {
  if (targetPort.value === null || targetPort.value === undefined) return null
  if (!isValidPort(targetPort.value)) {
    return 'Target Port must be an integer between 1 and 65535.'
  }
  return null
})

const nodePortErrorMessage = computed(() => {
  if (
    serviceType.value !== KUBERNETES_SERVICE_TYPE.NodePort ||
    nodePort.value === null ||
    nodePort.value === undefined
  ) {
    return null
  }
  if (!isValidPort(nodePort.value)) {
    return 'NodePort must be an integer between 1 and 65535.'
  }
  return null
})

const isFormValid = computed(() => {
  const trimmedName = name.value.trim()
  const hasValidName = Boolean(trimmedName) && !nameErrorMessage.value
  const hasValidNamespace = Boolean(namespace.value)
  const hasValidSelector = !selectorAppErrorMessage.value
  const hasValidPort = port.value !== null && port.value !== undefined && !portErrorMessage.value
  const hasValidTargetPort = !targetPortErrorMessage.value
  const hasValidNodePort = !nodePortErrorMessage.value

  return (
    hasValidName &&
    hasValidNamespace &&
    hasValidSelector &&
    hasValidPort &&
    hasValidTargetPort &&
    hasValidNodePort
  )
})

const handleCancel = () => {
  dialogRef?.value?.close()
}

const handleCreate = async () => {
  if (!isFormValid.value || isCreating.value) return

  const trimmedName = name.value.trim()
  const trimmedNamespace = namespace.value.trim()
  const trimmedSelectorApp = selectorApp.value.trim()

  const servicePortObj: ServicePort = {
    name: 'http',
    port: port.value,
    protocol: 'TCP'
  }

  if (targetPort.value !== null && targetPort.value !== undefined) {
    servicePortObj.targetPort = targetPort.value
  }

  if (
    serviceType.value === KUBERNETES_SERVICE_TYPE.NodePort &&
    nodePort.value !== null &&
    nodePort.value !== undefined
  ) {
    servicePortObj.nodePort = nodePort.value
  }

  const manifest: Service = {
    apiVersion: 'v1',
    kind: KUBERNETES_RESOURCE_KIND.Service,
    metadata: {
      name: trimmedName,
      namespace: trimmedNamespace,
      labels: {
        'app.kubernetes.io/name': trimmedName
      }
    },
    spec: {
      type: serviceType.value,
      ports: [servicePortObj]
    }
  }

  if (trimmedSelectorApp) {
    manifest.spec!.selector = {
      app: trimmedSelectorApp
    }
  }

  isCreating.value = true
  submittedName.value = trimmedName
  try {
    await kubernetesService.createResource({
      namespace: trimmedNamespace,
      kind: KUBERNETES_RESOURCE_KIND.Service,
      name: trimmedName,
      data: manifest
    })
  } catch {
    isCreating.value = false
    submittedName.value = ''
  }
}
</script>

<template>
  <form @submit.prevent="handleCreate" class="flex flex-col gap-3.5">
    <p class="text-xs text-muted-color">
      Create a new Kubernetes Service with standard networking configuration:
    </p>

    <!-- Basic Information (Name & Namespace) -->
    <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
      <div class="flex flex-col gap-1.5">
        <label for="create-service-name" class="text-xs font-semibold text-muted-color">
          Name <span class="text-(--danger)">*</span>
        </label>
        <InputText
          id="create-service-name"
          v-model="name"
          placeholder="e.g. backend-svc"
          fluid
          size="small"
          :invalid="Boolean(name.trim() && nameErrorMessage)"
        />
        <small
          v-if="name.trim() && nameErrorMessage"
          class="text-(--danger) text-[11px] leading-tight"
        >
          {{ nameErrorMessage }}
        </small>
      </div>

      <div class="flex flex-col gap-1.5">
        <label for="create-service-namespace" class="text-xs font-semibold text-muted-color">
          Namespace <span class="text-(--danger)">*</span>
        </label>
        <Select
          id="create-service-namespace"
          v-model="namespace"
          :options="namespaceOptions"
          placeholder="Select Namespace"
          fluid
          size="small"
        />
      </div>
    </div>

    <!-- Service Type & Selector -->
    <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
      <div class="flex flex-col gap-1.5">
        <label for="create-service-type" class="text-xs font-semibold text-muted-color">
          Service Type <span class="text-(--danger)">*</span>
        </label>
        <Select
          id="create-service-type"
          v-model="serviceType"
          :options="serviceTypeOptions"
          placeholder="Select Type"
          fluid
          size="small"
        />
      </div>

      <div class="flex flex-col gap-1.5">
        <label for="create-service-selector" class="text-xs font-semibold text-muted-color">
          Selector (App label)
        </label>
        <InputText
          id="create-service-selector"
          v-model="selectorApp"
          placeholder="e.g. backend"
          fluid
          size="small"
          :invalid="Boolean(selectorApp.trim() && selectorAppErrorMessage)"
        />
        <small
          v-if="selectorApp.trim() && selectorAppErrorMessage"
          class="text-(--danger) text-[11px] leading-tight"
        >
          {{ selectorAppErrorMessage }}
        </small>
      </div>
    </div>

    <!-- Ports Configuration -->
    <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
      <div class="flex flex-col gap-1.5">
        <label for="create-service-port" class="text-xs font-semibold text-muted-color">
          Port <span class="text-(--danger)">*</span>
        </label>
        <InputNumber
          id="create-service-port"
          v-model="port"
          :min="1"
          :max="65535"
          placeholder="e.g. 80"
          fluid
          size="small"
          :invalid="Boolean(portErrorMessage)"
        />
        <small v-if="portErrorMessage" class="text-(--danger) text-[11px] leading-tight">
          {{ portErrorMessage }}
        </small>
      </div>

      <div class="flex flex-col gap-1.5">
        <label for="create-service-target-port" class="text-xs font-semibold text-muted-color">
          Target Port (Optional)
        </label>
        <InputNumber
          id="create-service-target-port"
          v-model="targetPort"
          :min="1"
          :max="65535"
          placeholder="e.g. 8080"
          fluid
          size="small"
          :invalid="Boolean(targetPort !== null && targetPortErrorMessage)"
        />
        <small
          v-if="targetPort !== null && targetPortErrorMessage"
          class="text-(--danger) text-[11px] leading-tight"
        >
          {{ targetPortErrorMessage }}
        </small>
      </div>
    </div>

    <!-- Node Port Row (shown only when NodePort is selected) -->
    <div v-if="serviceType === KUBERNETES_SERVICE_TYPE.NodePort" class="flex flex-col gap-1.5">
      <label for="create-service-node-port" class="text-xs font-semibold text-muted-color">
        Node Port (Optional)
      </label>
      <InputNumber
        id="create-service-node-port"
        v-model="nodePort"
        :min="1"
        :max="65535"
        placeholder="e.g. 30080 (auto-assigned if empty)"
        fluid
        size="small"
        :invalid="Boolean(nodePort !== null && nodePortErrorMessage)"
      />
      <small
        v-if="nodePort !== null && nodePortErrorMessage"
        class="text-(--danger) text-[11px] leading-tight"
      >
        {{ nodePortErrorMessage }}
      </small>
    </div>

    <!-- Action Buttons -->
    <div class="flex justify-end gap-2 pt-2">
      <Button
        type="button"
        label="Cancel"
        severity="secondary"
        variant="text"
        size="small"
        :disabled="isCreating"
        @click="handleCancel"
      />
      <Button
        type="submit"
        label="Create"
        severity="primary"
        size="small"
        :loading="isCreating"
        :disabled="!isFormValid || isCreating"
      />
    </div>
  </form>
</template>
