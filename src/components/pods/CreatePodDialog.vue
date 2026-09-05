<script setup lang="ts">
import { kubernetesService } from '@/services/kubernetesService'
import { events } from '@/services/nativeService'
import { useKubernetesStore } from '@/stores/kubernetesStore'
import { OrbitEvents } from '@/types/events'
import {
  KUBERNETES_RESTART_POLICY,
  KUBERNETES_RESTART_POLICIES,
  KUBERNETES_RESOURCE_KIND,
  type KubernetesRestartPolicy
} from '@/constants/kubernetes'
import { isValidK8sLabel, isValidK8sName, isValidPort } from '@/utils/validators'
import type { Container, EnvVar, Pod } from 'kubernetes-types/core/v1'
import Button from 'primevue/button'
import InputNumber from 'primevue/inputnumber'
import InputText from 'primevue/inputtext'
import Select from 'primevue/select'
import KeyValueEditor, { type KeyValuePair } from '@/components/shared/KeyValueEditor.vue'
import { ChevronDown, ChevronUp } from '@lucide/vue'
import { computed, inject, onMounted, onUnmounted, ref, type Ref } from 'vue'

interface CreatePodDialogData {
  initialNamespace?: string
}

const dialogRef = inject<
  | Ref<{
      data?: CreatePodDialogData
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
const port = ref<number | null>(null)
const restartPolicy = ref<KubernetesRestartPolicy>(KUBERNETES_RESTART_POLICY.Always)

// Advanced Options
const showAdvanced = ref(false)
const containerName = ref('')
const command = ref('')
const labels = ref<KeyValuePair[]>([])
const envVars = ref<KeyValuePair[]>([])

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
  const exists = k8sStore.pods.some(
    (pod) => pod.namespace === namespace.value && pod.name.toLowerCase() === trimmed.toLowerCase()
  )
  if (exists) {
    return `A Pod named "${trimmed}" already exists in namespace "${namespace.value}".`
  }
  return null
})

const containerNameErrorMessage = computed(() => {
  const trimmed = containerName.value.trim()
  if (!trimmed) return null
  if (!isValidK8sLabel(trimmed)) {
    return 'Container name must be a valid DNS-1123 label (lowercase letters, numbers, hyphens).'
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
  const hasValidContainerName = !containerNameErrorMessage.value
  const hasValidPort = port.value === null || port.value === undefined || isValidPort(port.value)

  return hasValidName && hasValidNamespace && hasValidImage && hasValidContainerName && hasValidPort
})

const handleCancel = () => {
  dialogRef?.value?.close()
}

const handleCreate = async () => {
  if (!isFormValid.value || isCreating.value) return

  const trimmedName = name.value.trim()
  const trimmedNamespace = namespace.value.trim()
  const trimmedImage = image.value.trim()
  const fallbackContainerName =
    trimmedName
      .toLowerCase()
      .replace(/[^a-z0-9-]/g, '-')
      .replace(/^-+|-+$/g, '')
      .slice(0, 63) || 'main'
  const trimmedContainerName = containerName.value.trim() || fallbackContainerName

  const containerObj: Container = {
    name: trimmedContainerName,
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

  if (command.value.trim()) {
    const matches = command.value.trim().match(/(?:[^\s,"']+|"[^"]*"|'[^']*')+/g)
    if (matches && matches.length > 0) {
      containerObj.command = matches.map((arg) => arg.replace(/^["']|["']$/g, ''))
    }
  }

  const envList: EnvVar[] = []
  for (const item of envVars.value) {
    const k = item.key.trim()
    if (k) {
      envList.push({ name: k, value: item.value })
    }
  }
  if (envList.length > 0) {
    containerObj.env = envList
  }

  const labelsMap: Record<string, string> = {
    app: trimmedName
  }
  for (const item of labels.value) {
    const k = item.key.trim()
    if (k) {
      labelsMap[k] = item.value.trim()
    }
  }

  const manifest: Pod = {
    apiVersion: 'v1',
    kind: KUBERNETES_RESOURCE_KIND.Pod,
    metadata: {
      name: trimmedName,
      namespace: trimmedNamespace,
      labels: labelsMap
    },
    spec: {
      restartPolicy: restartPolicy.value,
      containers: [containerObj]
    }
  }

  isCreating.value = true
  submittedName.value = trimmedName
  try {
    await kubernetesService.createResource({
      namespace: trimmedNamespace,
      kind: KUBERNETES_RESOURCE_KIND.Pod,
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
  <form
    @submit.prevent="handleCreate"
    class="flex flex-col gap-3.5 max-h-[80vh] overflow-y-auto pr-1"
  >
    <p class="text-xs text-muted-color">
      Create a new standalone Kubernetes Pod with container configuration:
    </p>

    <!-- Name -->
    <div class="flex flex-col gap-1.5">
      <label for="create-pod-name" class="text-xs font-semibold text-muted-color">
        Name <span class="text-(--danger)">*</span>
      </label>
      <InputText
        id="create-pod-name"
        v-model="name"
        placeholder="e.g. my-pod"
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
      <label for="create-pod-namespace" class="text-xs font-semibold text-muted-color">
        Namespace <span class="text-(--danger)">*</span>
      </label>
      <Select
        id="create-pod-namespace"
        v-model="namespace"
        :options="namespaceOptions"
        fluid
        size="small"
        class="text-xs"
      />
    </div>

    <!-- Image -->
    <div class="flex flex-col gap-1.5">
      <label for="create-pod-image" class="text-xs font-semibold text-muted-color">
        Image <span class="text-(--danger)">*</span>
      </label>
      <InputText
        id="create-pod-image"
        v-model="image"
        placeholder="e.g. nginx:alpine"
        fluid
        size="small"
        class="text-xs"
      />
    </div>

    <!-- Port & Restart Policy Row -->
    <div class="grid grid-cols-2 gap-3">
      <div class="flex flex-col gap-1.5">
        <label for="create-pod-port" class="text-xs font-semibold text-muted-color">
          Port (Optional)
        </label>
        <InputNumber
          id="create-pod-port"
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

      <div class="flex flex-col gap-1.5">
        <label for="create-pod-restart-policy" class="text-xs font-semibold text-muted-color">
          Restart Policy
        </label>
        <Select
          id="create-pod-restart-policy"
          v-model="restartPolicy"
          :options="KUBERNETES_RESTART_POLICIES"
          fluid
          size="small"
          class="text-xs"
        />
      </div>
    </div>

    <!-- Advanced Options Toggle -->
    <div class="pt-1">
      <Button
        type="button"
        variant="text"
        severity="secondary"
        size="small"
        class="text-xs p-0! font-semibold text-muted-color flex items-center gap-1.5 hover:text-primary"
        @click="showAdvanced = !showAdvanced"
      >
        <ChevronUp v-if="showAdvanced" class="w-3.5 h-3.5" />
        <ChevronDown v-else class="w-3.5 h-3.5" />
        <span>{{
          showAdvanced
            ? 'Hide Advanced Options'
            : 'Advanced Options (Container Name, Command, Labels, Env)'
        }}</span>
      </Button>
    </div>

    <!-- Collapsible Advanced Section -->
    <div v-if="showAdvanced" class="flex flex-col gap-3.5 p-3 rounded-lg bg-(--bg-hover)/30">
      <!-- Container Name -->
      <div class="flex flex-col gap-1.5">
        <label for="create-pod-container-name" class="text-xs font-semibold text-muted-color">
          Container Name (Optional)
        </label>
        <InputText
          id="create-pod-container-name"
          v-model="containerName"
          placeholder="Defaults to pod name"
          fluid
          size="small"
          :invalid="Boolean(containerName.trim() && containerNameErrorMessage)"
          class="text-xs"
        />
        <small
          v-if="containerName.trim() && containerNameErrorMessage"
          class="text-(--danger) text-[11px] leading-tight"
        >
          {{ containerNameErrorMessage }}
        </small>
      </div>

      <!-- Command -->
      <div class="flex flex-col gap-1.5">
        <label for="create-pod-command" class="text-xs font-semibold text-muted-color">
          Command (Optional)
        </label>
        <InputText
          id="create-pod-command"
          v-model="command"
          placeholder="e.g. sleep 3600 or sh, -c, echo hello"
          fluid
          size="small"
          class="text-xs"
        />
      </div>

      <!-- Labels -->
      <KeyValueEditor
        v-model="labels"
        title="Labels (Optional)"
        keyPlaceholder="e.g. tier"
        valuePlaceholder="e.g. backend"
        addLabel="Add Label"
      />

      <!-- Environment Variables -->
      <KeyValueEditor
        v-model="envVars"
        title="Environment Variables (Optional)"
        keyPlaceholder="e.g. ENV_NAME"
        valuePlaceholder="e.g. value"
        addLabel="Add Variable"
      />
    </div>

    <!-- Actions Footer -->
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
