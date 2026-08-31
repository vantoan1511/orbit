<script setup lang="ts">
import { kubernetesService } from '@/services/kubernetesService'
import { events } from '@/services/nativeService'
import { useKubernetesStore } from '@/stores/kubernetesStore'
import { OrbitEvents } from '@/types/events'
import { KUBERNETES_RESOURCE_KIND } from '@/constants/kubernetes'
import { isValidK8sName } from '@/utils/validators'
import type { ConfigMap } from 'kubernetes-types/core/v1'
import Button from 'primevue/button'
import InputText from 'primevue/inputtext'
import Select from 'primevue/select'
import KeyValueEditor, { type KeyValuePair } from '@/components/shared/KeyValueEditor.vue'
import { computed, inject, onMounted, onUnmounted, ref, type Ref } from 'vue'

interface CreateConfigMapDialogData {
  initialNamespace?: string
}

const dialogRef = inject<
  | Ref<{
      data?: CreateConfigMapDialogData
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
const dataItems = ref<KeyValuePair[]>([])
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
  const exists = k8sStore.configMaps.some(
    (cm) => cm.namespace === namespace.value && cm.name.toLowerCase() === trimmed.toLowerCase()
  )
  if (exists) {
    return `A ConfigMap named "${trimmed}" already exists in namespace "${namespace.value}".`
  }
  return null
})

const isFormValid = computed(() => {
  const trimmedName = name.value.trim()
  const hasValidName = Boolean(trimmedName) && !nameErrorMessage.value
  const hasValidNamespace = Boolean(namespace.value)

  return hasValidName && hasValidNamespace
})

const handleCancel = () => {
  dialogRef?.value?.close()
}

const handleCreate = async () => {
  if (!isFormValid.value || isCreating.value) return

  const trimmedName = name.value.trim()
  const trimmedNamespace = namespace.value.trim()

  const dataMap: Record<string, string> = {}
  for (const item of dataItems.value) {
    const k = item.key.trim()
    if (k) {
      dataMap[k] = item.value
    }
  }

  const manifest: ConfigMap = {
    apiVersion: 'v1',
    kind: KUBERNETES_RESOURCE_KIND.ConfigMap,
    metadata: {
      name: trimmedName,
      namespace: trimmedNamespace
    }
  }

  if (Object.keys(dataMap).length > 0) {
    manifest.data = dataMap
  }

  isCreating.value = true
  submittedName.value = trimmedName
  try {
    await kubernetesService.createResource({
      namespace: trimmedNamespace,
      kind: KUBERNETES_RESOURCE_KIND.ConfigMap,
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
      Create a new Kubernetes ConfigMap to store non-confidential key-value configuration data:
    </p>

    <!-- Name & Namespace -->
    <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
      <div class="flex flex-col gap-1.5">
        <label for="create-configmap-name" class="text-xs font-semibold text-muted-color">
          Name <span class="text-(--danger)">*</span>
        </label>
        <InputText
          id="create-configmap-name"
          v-model="name"
          placeholder="e.g. app-config"
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
        <label for="create-configmap-namespace" class="text-xs font-semibold text-muted-color">
          Namespace <span class="text-(--danger)">*</span>
        </label>
        <Select
          id="create-configmap-namespace"
          v-model="namespace"
          :options="namespaceOptions"
          placeholder="Select Namespace"
          fluid
          size="small"
        />
      </div>
    </div>

    <!-- Data (Key-Value pairs) -->
    <div class="flex flex-col gap-1.5">
      <KeyValueEditor
        v-model="dataItems"
        title="Data (Key-Value pairs)"
        key-placeholder="Key"
        value-placeholder="Value"
        add-label="Add entry"
        :disabled="isCreating"
      />
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
