<script setup lang="ts">
import { useCreateResourceDialog } from '@/composables/useCreateResourceDialog'
import { kubernetesService } from '@/services/kubernetesService'
import { KUBERNETES_RESOURCE_KIND } from '@/constants/kubernetes'
import { isValidK8sName, isValidPort, sanitizeK8sLabel } from '@/utils/validators'
import type { DaemonSet } from 'kubernetes-types/apps/v1'
import type { Container } from 'kubernetes-types/core/v1'
import Button from 'primevue/button'
import InputNumber from 'primevue/inputnumber'
import InputText from 'primevue/inputtext'
import Select from 'primevue/select'
import { computed, ref } from 'vue'

const {
  k8sStore,
  namespace,
  namespaceOptions,
  isCreating,
  handleCancel,
  submitCreation,
  handleCreationFailure,
  isNameTaken
} = useCreateResourceDialog()

const name = ref('')
const image = ref('')
const port = ref<number | null>(null)

const nameErrorMessage = computed(() => {
  const trimmed = name.value.trim()
  if (!trimmed) return null
  if (!isValidK8sName(trimmed)) {
    return 'Name must be a valid DNS-1123 subdomain (lowercase letters, numbers, hyphens, dots).'
  }
  if (isNameTaken(trimmed, k8sStore.daemonSets)) {
    return `A DaemonSet named "${trimmed}" already exists in namespace "${namespace.value}".`
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

const handleCreate = async () => {
  if (!isFormValid.value || isCreating.value) return

  const trimmedName = name.value.trim()
  const trimmedNamespace = namespace.value.trim()
  const trimmedImage = image.value.trim()
  const containerName = sanitizeK8sLabel(trimmedName)

  const containerObj: Container = {
    name: containerName,
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

  const manifest: DaemonSet = {
    apiVersion: 'apps/v1',
    kind: KUBERNETES_RESOURCE_KIND.DaemonSet,
    metadata: {
      name: trimmedName,
      namespace: trimmedNamespace,
      labels: {
        app: trimmedName
      }
    },
    spec: {
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

  submitCreation(trimmedName)
  try {
    await kubernetesService.createResource({
      namespace: trimmedNamespace,
      kind: KUBERNETES_RESOURCE_KIND.DaemonSet,
      name: trimmedName,
      data: manifest
    })
  } catch {
    handleCreationFailure()
  }
}
</script>

<template>
  <form @submit.prevent="handleCreate" class="flex flex-col gap-3.5">
    <p class="text-xs text-muted-color">
      Create a new Kubernetes DaemonSet that runs a copy of a pod on each node:
    </p>

    <!-- Name -->
    <div class="flex flex-col gap-1.5">
      <label for="create-daemonset-name" class="text-xs font-semibold text-muted-color">
        Name <span class="text-(--danger)">*</span>
      </label>
      <InputText
        id="create-daemonset-name"
        v-model="name"
        placeholder="e.g. node-exporter"
        fluid
        size="small"
        :invalid="Boolean(name.trim() && nameErrorMessage)"
        aria-describedby="create-daemonset-name-error"
        class="text-xs"
      />
      <small
        v-if="name.trim() && nameErrorMessage"
        id="create-daemonset-name-error"
        class="text-(--danger) text-[11px] leading-tight"
      >
        {{ nameErrorMessage }}
      </small>
    </div>

    <!-- Namespace -->
    <div class="flex flex-col gap-1.5">
      <label for="create-daemonset-namespace" class="text-xs font-semibold text-muted-color">
        Namespace <span class="text-(--danger)">*</span>
      </label>
      <Select
        id="create-daemonset-namespace"
        v-model="namespace"
        :options="namespaceOptions"
        fluid
        size="small"
        class="text-xs"
      />
    </div>

    <!-- Image -->
    <div class="flex flex-col gap-1.5">
      <label for="create-daemonset-image" class="text-xs font-semibold text-muted-color">
        Image <span class="text-(--danger)">*</span>
      </label>
      <InputText
        id="create-daemonset-image"
        v-model="image"
        placeholder="e.g. prom/node-exporter:latest"
        fluid
        size="small"
        class="text-xs"
      />
    </div>

    <!-- Port -->
    <div class="flex flex-col gap-1.5">
      <label for="create-daemonset-port" class="text-xs font-semibold text-muted-color">
        Port (Optional)
      </label>
      <InputNumber
        id="create-daemonset-port"
        v-model="port"
        :min="1"
        :max="65535"
        placeholder="e.g. 9100"
        fluid
        size="small"
        :invalid="Boolean(port !== null && portErrorMessage)"
        aria-describedby="create-daemonset-port-error"
      />
      <small
        v-if="port !== null && portErrorMessage"
        id="create-daemonset-port-error"
        class="text-(--danger) text-[11px] leading-tight"
      >
        {{ portErrorMessage }}
      </small>
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
