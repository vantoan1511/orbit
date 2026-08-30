<script setup lang="ts">
import { kubernetesService } from '@/services/kubernetesService'
import { events } from '@/services/nativeService'
import { useKubernetesStore } from '@/stores/kubernetesStore'
import { OrbitEvents } from '@/types/events'
import { isValidHost, isValidK8sName, isValidPath, isValidPort } from '@/utils/validators'
import type { HTTPIngressPath, Ingress, IngressRule } from 'kubernetes-types/networking/v1'
import Button from 'primevue/button'
import InputNumber from 'primevue/inputnumber'
import InputText from 'primevue/inputtext'
import Select from 'primevue/select'
import { computed, inject, onMounted, onUnmounted, ref, type Ref } from 'vue'

interface CreateIngressDialogData {
  initialNamespace?: string
}

const dialogRef = inject<
  | Ref<{
      data?: CreateIngressDialogData
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

const name = ref('')
const namespace = ref('default')
const className = ref('')
const serviceName = ref('')
const servicePort = ref<number>(80)
const host = ref('')
const path = ref('/')
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
  const exists = k8sStore.ingresses.some(
    (ing) => ing.namespace === namespace.value && ing.name.toLowerCase() === trimmed.toLowerCase()
  )
  if (exists) {
    return `An Ingress named "${trimmed}" already exists in namespace "${namespace.value}".`
  }
  return null
})

const serviceNameErrorMessage = computed(() => {
  const trimmed = serviceName.value.trim()
  if (!trimmed) return null
  if (!isValidK8sName(trimmed)) {
    return 'Service name must be a valid DNS-1123 subdomain.'
  }
  return null
})

const portErrorMessage = computed(() => {
  if (servicePort.value === null || servicePort.value === undefined)
    return 'Service port is required.'
  if (!isValidPort(servicePort.value)) {
    return 'Port must be an integer between 1 and 65535.'
  }
  return null
})

const hostErrorMessage = computed(() => {
  const trimmed = host.value.trim()
  if (!trimmed) return null
  if (!isValidHost(trimmed)) {
    return 'Host must be a valid hostname (e.g. example.com or *.example.com).'
  }
  return null
})

const pathErrorMessage = computed(() => {
  const trimmed = path.value.trim()
  if (!trimmed) return null
  if (!isValidPath(trimmed)) {
    return 'Path must begin with a forward slash (/).'
  }
  return null
})

const isFormValid = computed(() => {
  const trimmedName = name.value.trim()
  const trimmedServiceName = serviceName.value.trim()
  const hasValidName = Boolean(trimmedName) && !nameErrorMessage.value
  const hasValidNamespace = Boolean(namespace.value)
  const hasValidServiceName = Boolean(trimmedServiceName) && !serviceNameErrorMessage.value
  const hasValidPort =
    servicePort.value !== null && servicePort.value !== undefined && !portErrorMessage.value
  const hasValidHost = !hostErrorMessage.value
  const hasValidPath = !pathErrorMessage.value

  return (
    hasValidName &&
    hasValidNamespace &&
    hasValidServiceName &&
    hasValidPort &&
    hasValidHost &&
    hasValidPath
  )
})

const handleCancel = () => {
  dialogRef?.value?.close()
}

const handleCreate = async () => {
  if (!isFormValid.value || isCreating.value) return

  const trimmedName = name.value.trim()
  const trimmedNamespace = namespace.value.trim()
  const trimmedServiceName = serviceName.value.trim()
  const trimmedHost = host.value.trim()
  const trimmedPath = path.value.trim() || '/'
  const trimmedClassName = className.value.trim()

  const httpPath: HTTPIngressPath = {
    path: trimmedPath,
    pathType: 'Prefix',
    backend: {
      service: {
        name: trimmedServiceName,
        port: {
          number: servicePort.value
        }
      }
    }
  }

  const rule: IngressRule = {
    http: {
      paths: [httpPath]
    }
  }

  if (trimmedHost) {
    rule.host = trimmedHost
  }

  const manifest: Ingress = {
    apiVersion: 'networking.k8s.io/v1',
    kind: 'Ingress',
    metadata: {
      name: trimmedName,
      namespace: trimmedNamespace,
      labels: {
        'app.kubernetes.io/name': trimmedName
      }
    },
    spec: {
      ...(trimmedClassName ? { ingressClassName: trimmedClassName } : {}),
      rules: [rule]
    }
  }

  isCreating.value = true
  submittedName.value = trimmedName
  try {
    await kubernetesService.createResource({
      namespace: trimmedNamespace,
      kind: 'Ingress',
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
      Create a new Kubernetes Ingress with standard routing configuration:
    </p>

    <!-- Name -->
    <div class="flex flex-col gap-1.5">
      <label for="create-ingress-name" class="text-xs font-semibold text-muted-color">
        Name <span class="text-(--danger)">*</span>
      </label>
      <InputText
        id="create-ingress-name"
        v-model="name"
        placeholder="e.g. my-ingress"
        fluid
        size="small"
        :invalid="Boolean(name.trim() && nameErrorMessage)"
        class="text-xs"
      />
      <small
        v-if="name.trim() && nameErrorMessage"
        class="text-(--danger) text-[11px] leading-tight"
      >
        {{ nameErrorMessage }}
      </small>
    </div>

    <!-- Namespace & IngressClass Row -->
    <div class="grid grid-cols-2 gap-3">
      <div class="flex flex-col gap-1.5">
        <label for="create-ingress-namespace" class="text-xs font-semibold text-muted-color">
          Namespace <span class="text-(--danger)">*</span>
        </label>
        <Select
          id="create-ingress-namespace"
          v-model="namespace"
          :options="namespaceOptions"
          fluid
          size="small"
          class="text-xs"
        />
      </div>

      <div class="flex flex-col gap-1.5">
        <label for="create-ingress-class" class="text-xs font-semibold text-muted-color">
          Ingress Class
        </label>
        <InputText
          id="create-ingress-class"
          v-model="className"
          placeholder="e.g. nginx"
          fluid
          size="small"
          class="text-xs"
        />
      </div>
    </div>

    <!-- Service Name & Service Port Row -->
    <div class="grid grid-cols-2 gap-3">
      <div class="flex flex-col gap-1.5">
        <label for="create-ingress-service" class="text-xs font-semibold text-muted-color">
          Backend Service <span class="text-(--danger)">*</span>
        </label>
        <InputText
          id="create-ingress-service"
          v-model="serviceName"
          placeholder="e.g. web-service"
          fluid
          size="small"
          :invalid="Boolean(serviceName.trim() && serviceNameErrorMessage)"
          class="text-xs"
        />
        <small
          v-if="serviceName.trim() && serviceNameErrorMessage"
          class="text-(--danger) text-[11px] leading-tight"
        >
          {{ serviceNameErrorMessage }}
        </small>
      </div>

      <div class="flex flex-col gap-1.5">
        <label for="create-ingress-port" class="text-xs font-semibold text-muted-color">
          Service Port <span class="text-(--danger)">*</span>
        </label>
        <InputNumber
          id="create-ingress-port"
          v-model="servicePort"
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
    </div>

    <!-- Host & Path Row -->
    <div class="grid grid-cols-2 gap-3">
      <div class="flex flex-col gap-1.5">
        <label for="create-ingress-host" class="text-xs font-semibold text-muted-color">
          Host (Optional)
        </label>
        <InputText
          id="create-ingress-host"
          v-model="host"
          placeholder="e.g. app.example.com"
          fluid
          size="small"
          :invalid="Boolean(host.trim() && hostErrorMessage)"
          class="text-xs"
        />
        <small
          v-if="host.trim() && hostErrorMessage"
          class="text-(--danger) text-[11px] leading-tight"
        >
          {{ hostErrorMessage }}
        </small>
      </div>

      <div class="flex flex-col gap-1.5">
        <label for="create-ingress-path" class="text-xs font-semibold text-muted-color">
          Path
        </label>
        <InputText
          id="create-ingress-path"
          v-model="path"
          placeholder="/"
          fluid
          size="small"
          :invalid="Boolean(path.trim() && pathErrorMessage)"
          class="text-xs"
        />
        <small
          v-if="path.trim() && pathErrorMessage"
          class="text-(--danger) text-[11px] leading-tight"
        >
          {{ pathErrorMessage }}
        </small>
      </div>
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
