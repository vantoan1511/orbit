<script setup lang="ts">
import { kubernetesService } from '@/services/kubernetesService'
import { events } from '@/services/nativeService'
import { useKubernetesStore } from '@/stores/kubernetesStore'
import { OrbitEvents } from '@/types/events'
import { KUBERNETES_RESOURCE_KIND } from '@/constants/kubernetes'
import { isValidK8sLabel } from '@/utils/validators'
import type { Namespace } from 'kubernetes-types/core/v1'
import Button from 'primevue/button'
import InputText from 'primevue/inputtext'
import KeyValueEditor, { type KeyValuePair } from '@/components/shared/KeyValueEditor.vue'
import { computed, inject, onMounted, onUnmounted, ref, type Ref } from 'vue'

const dialogRef = inject<
  | Ref<{
      close: () => void
    }>
  | undefined
>('dialogRef')

const k8sStore = useKubernetesStore()

const name = ref('')
const labels = ref<KeyValuePair[]>([])
const annotations = ref<KeyValuePair[]>([])
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
})

onUnmounted(() => {
  events.off(OrbitEvents.CommandSucceeded, handleCommandSucceeded)
  events.off(OrbitEvents.ErrorOccurred, handleErrorOccurred)
})

const nameErrorMessage = computed(() => {
  const trimmed = name.value.trim()
  if (!trimmed) return null
  if (!isValidK8sLabel(trimmed)) {
    return 'Name must be a valid DNS-1123 label (at most 63 lowercase alphanumeric characters or hyphens, and cannot start or end with a hyphen).'
  }
  const exists = k8sStore.namespaceList.some(
    (ns) => ns.name.toLowerCase() === trimmed.toLowerCase()
  )
  if (exists) {
    return `A Namespace named "${trimmed}" already exists.`
  }
  return null
})

const isFormValid = computed(() => {
  const trimmedName = name.value.trim()
  return Boolean(trimmedName) && !nameErrorMessage.value
})

const handleCancel = () => {
  dialogRef?.value?.close()
}

const handleCreate = async () => {
  if (!isFormValid.value || isCreating.value) return

  const trimmedName = name.value.trim()

  const labelsMap: Record<string, string> = {}
  for (const item of labels.value) {
    const k = item.key.trim()
    if (k) {
      labelsMap[k] = item.value.trim()
    }
  }

  const annotationsMap: Record<string, string> = {}
  for (const item of annotations.value) {
    const k = item.key.trim()
    if (k) {
      annotationsMap[k] = item.value
    }
  }

  const manifest: Namespace = {
    apiVersion: 'v1',
    kind: KUBERNETES_RESOURCE_KIND.Namespace,
    metadata: {
      name: trimmedName,
      ...(Object.keys(labelsMap).length > 0 ? { labels: labelsMap } : {}),
      ...(Object.keys(annotationsMap).length > 0 ? { annotations: annotationsMap } : {})
    }
  }

  isCreating.value = true
  submittedName.value = trimmedName
  try {
    await kubernetesService.createResource({
      namespace: '',
      kind: KUBERNETES_RESOURCE_KIND.Namespace,
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
      Create a new Kubernetes Namespace to partition cluster resources and provide logical
      isolation:
    </p>

    <!-- Name -->
    <div class="flex flex-col gap-1.5">
      <label for="create-namespace-name" class="text-xs font-semibold text-muted-color">
        Name <span class="text-(--danger)">*</span>
      </label>
      <InputText
        id="create-namespace-name"
        v-model="name"
        placeholder="e.g. staging"
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

    <!-- Labels -->
    <KeyValueEditor
      v-model="labels"
      title="Labels (Optional)"
      keyPlaceholder="e.g. environment"
      valuePlaceholder="e.g. staging"
      addLabel="Add Label"
    />

    <!-- Annotations -->
    <KeyValueEditor
      v-model="annotations"
      title="Annotations (Optional)"
      keyPlaceholder="e.g. description"
      valuePlaceholder="e.g. Staging environment"
      addLabel="Add Annotation"
    />

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
