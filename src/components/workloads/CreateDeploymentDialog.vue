<script setup lang="ts">
import { kubernetesService } from '@/services/kubernetesService'
import { events } from '@/services/nativeService'
import { useKubernetesStore } from '@/stores/kubernetesStore'
import { OrbitEvents } from '@/types/events'
import { isValidK8sName, isValidPort } from '@/utils/validators'
import Button from 'primevue/button'
import InputNumber from 'primevue/inputnumber'
import InputText from 'primevue/inputtext'
import Select from 'primevue/select'
import { computed, inject, onMounted, onUnmounted, ref, type Ref } from 'vue'

interface CreateDeploymentDialogData {
  initialNamespace?: string
}

const dialogRef = inject<
  | Ref<{
      data?: CreateDeploymentDialogData
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
const image = ref('')
const replicas = ref<number>(1)
const port = ref<number | null>(null)
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
  const exists = k8sStore.deployments.some(
    (dep) => dep.namespace === namespace.value && dep.name.toLowerCase() === trimmed.toLowerCase()
  )
  if (exists) {
    return `A Deployment named "${trimmed}" already exists in namespace "${namespace.value}".`
  }
  return null
})

const portErrorMessage = computed(() => {
  if (port.value === null || port.value === undefined) return null
  if (!isValidPort(port.value)) {
    return 'Port must be an integer between 1 and 65535.'
  }
  return null
})

const isFormValid = computed(() => {
  const trimmedName = name.value.trim()
  const trimmedImage = image.value.trim()
  const hasValidName = Boolean(trimmedName) && !nameErrorMessage.value
  const hasValidNamespace = Boolean(namespace.value)
  const hasValidImage = Boolean(trimmedImage)
  const hasValidPort = port.value === null || port.value === undefined || isValidPort(port.value)

  return hasValidName && hasValidNamespace && hasValidImage && hasValidPort
})

const handleCancel = () => {
  dialogRef?.value?.close()
}

const handleCreate = async () => {
  if (!isFormValid.value || isCreating.value) return

  const trimmedName = name.value.trim()
  const trimmedNamespace = namespace.value.trim()
  const trimmedImage = image.value.trim()

  const containerObj: Record<string, unknown> = {
    name: trimmedName,
    image: trimmedImage
  }

  if (port.value !== null && port.value !== undefined && isValidPort(port.value)) {
    containerObj.ports = [
      {
        containerPort: port.value,
        protocol: 'TCP'
      }
    ]
  }

  const manifest: Record<string, unknown> = {
    apiVersion: 'apps/v1',
    kind: 'Deployment',
    metadata: {
      name: trimmedName,
      namespace: trimmedNamespace,
      labels: {
        app: trimmedName
      }
    },
    spec: {
      replicas: replicas.value ?? 1,
      selector: {
        matchLabels: {
          app: trimmedName
        }
      },
      template: {
        metadata: {
          labels: {
            app: trimmedName
          }
        },
        spec: {
          containers: [containerObj]
        }
      }
    }
  }

  isCreating.value = true
  submittedName.value = trimmedName
  try {
    await kubernetesService.createResource({
      namespace: trimmedNamespace,
      kind: 'Deployment',
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
      Create a new Kubernetes Deployment with standard configuration:
    </p>

    <!-- Name -->
    <div class="flex flex-col gap-1.5">
      <label for="create-deployment-name" class="text-xs font-semibold text-muted-color">
        Name <span class="text-(--danger)">*</span>
      </label>
      <InputText
        id="create-deployment-name"
        v-model="name"
        placeholder="e.g. my-app"
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

    <!-- Namespace -->
    <div class="flex flex-col gap-1.5">
      <label for="create-deployment-namespace" class="text-xs font-semibold text-muted-color">
        Namespace <span class="text-(--danger)">*</span>
      </label>
      <Select
        id="create-deployment-namespace"
        v-model="namespace"
        :options="namespaceOptions"
        fluid
        size="small"
        class="text-xs"
      />
    </div>

    <!-- Image -->
    <div class="flex flex-col gap-1.5">
      <label for="create-deployment-image" class="text-xs font-semibold text-muted-color">
        Image <span class="text-(--danger)">*</span>
      </label>
      <InputText
        id="create-deployment-image"
        v-model="image"
        placeholder="e.g. nginx:latest"
        fluid
        size="small"
        class="text-xs"
      />
    </div>

    <!-- Replicas & Port Row -->
    <div class="grid grid-cols-2 gap-3">
      <div class="flex flex-col gap-1.5">
        <label for="create-deployment-replicas" class="text-xs font-semibold text-muted-color">
          Replicas
        </label>
        <InputNumber
          id="create-deployment-replicas"
          v-model="replicas"
          :min="1"
          :max="1000"
          showButtons
          fluid
          size="small"
        />
      </div>

      <div class="flex flex-col gap-1.5">
        <label for="create-deployment-port" class="text-xs font-semibold text-muted-color">
          Port (Optional)
        </label>
        <InputNumber
          id="create-deployment-port"
          v-model="port"
          :min="1"
          :max="65535"
          placeholder="e.g. 80"
          fluid
          size="small"
          :invalid="Boolean(port !== null && portErrorMessage)"
        />
        <small
          v-if="port !== null && portErrorMessage"
          class="text-(--danger) text-[11px] leading-tight"
        >
          {{ portErrorMessage }}
        </small>
      </div>
    </div>

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
