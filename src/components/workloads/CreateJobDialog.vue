<script setup lang="ts">
import { useCreateResourceDialog } from '@/composables/useCreateResourceDialog'
import { kubernetesService } from '@/services/kubernetesService'
import {
  KUBERNETES_JOB_RESTART_POLICIES,
  KUBERNETES_JOB_RESTART_POLICY,
  KUBERNETES_RESOURCE_KIND,
  type KubernetesJobRestartPolicy
} from '@/constants/kubernetes'
import { isValidK8sName, sanitizeK8sLabel } from '@/utils/validators'
import type { Job } from 'kubernetes-types/batch/v1'
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
const command = ref('')
const completions = ref<number>(1)
const parallelism = ref<number>(1)
const backoffLimit = ref<number>(6)
const restartPolicy = ref<KubernetesJobRestartPolicy>(KUBERNETES_JOB_RESTART_POLICY.Never)

const nameErrorMessage = computed(() => {
  const trimmed = name.value.trim()
  if (!trimmed) return null
  if (!isValidK8sName(trimmed)) {
    return 'Name must be a valid DNS-1123 subdomain (lowercase letters, numbers, hyphens, dots).'
  }
  if (isNameTaken(trimmed, k8sStore.jobs)) {
    return `A Job named "${trimmed}" already exists in namespace "${namespace.value}".`
  }
  return null
})

const isFormValid = computed(() => {
  const trimmedName = name.value.trim()
  const trimmedImage = image.value.trim()
  const hasValidName = Boolean(trimmedName) && !nameErrorMessage.value
  const hasValidNamespace = Boolean(namespace.value)
  const hasValidImage = Boolean(trimmedImage)

  return hasValidName && hasValidNamespace && hasValidImage
})

const handleCreate = async () => {
  if (!isFormValid.value || isCreating.value) return

  const trimmedName = name.value.trim()
  const trimmedNamespace = namespace.value.trim()
  const trimmedImage = image.value.trim()
  const trimmedCommand = command.value.trim()
  const containerName = sanitizeK8sLabel(trimmedName)

  const containerObj: Container = {
    name: containerName,
    image: trimmedImage
  }

  if (trimmedCommand) {
    containerObj.command = ['/bin/sh', '-c', trimmedCommand]
  }

  const manifest: Job = {
    apiVersion: 'batch/v1',
    kind: KUBERNETES_RESOURCE_KIND.Job,
    metadata: {
      name: trimmedName,
      namespace: trimmedNamespace,
      labels: {
        app: trimmedName
      }
    },
    spec: {
      completions: completions.value ?? 1,
      parallelism: parallelism.value ?? 1,
      backoffLimit: backoffLimit.value ?? 6,
      template: {
        metadata: {
          labels: {
            app: trimmedName
          }
        },
        spec: {
          restartPolicy: restartPolicy.value,
          containers: [containerObj]
        }
      }
    }
  }

  submitCreation(trimmedName)
  try {
    await kubernetesService.createResource({
      namespace: trimmedNamespace,
      kind: KUBERNETES_RESOURCE_KIND.Job,
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
      Create a new Kubernetes batch Job to execute pods to completion:
    </p>

    <!-- Name -->
    <div class="flex flex-col gap-1.5">
      <label for="create-job-name" class="text-xs font-semibold text-muted-color">
        Name <span class="text-(--danger)">*</span>
      </label>
      <InputText
        id="create-job-name"
        v-model="name"
        placeholder="e.g. data-migration-job"
        fluid
        size="small"
        :invalid="Boolean(name.trim() && nameErrorMessage)"
        aria-describedby="create-job-name-error"
        class="text-xs"
      />
      <small
        v-if="name.trim() && nameErrorMessage"
        id="create-job-name-error"
        class="text-(--danger) text-[11px] leading-tight"
      >
        {{ nameErrorMessage }}
      </small>
    </div>

    <!-- Namespace -->
    <div class="flex flex-col gap-1.5">
      <label for="create-job-namespace" class="text-xs font-semibold text-muted-color">
        Namespace <span class="text-(--danger)">*</span>
      </label>
      <Select
        id="create-job-namespace"
        v-model="namespace"
        :options="namespaceOptions"
        fluid
        size="small"
        class="text-xs"
      />
    </div>

    <!-- Image -->
    <div class="flex flex-col gap-1.5">
      <label for="create-job-image" class="text-xs font-semibold text-muted-color">
        Image <span class="text-(--danger)">*</span>
      </label>
      <InputText
        id="create-job-image"
        v-model="image"
        placeholder="e.g. perl:5.34.0 or busybox:latest"
        fluid
        size="small"
        class="text-xs"
      />
    </div>

    <!-- Command (Optional) -->
    <div class="flex flex-col gap-1.5">
      <label for="create-job-command" class="text-xs font-semibold text-muted-color">
        Command (Optional)
      </label>
      <InputText
        id="create-job-command"
        v-model="command"
        placeholder="e.g. echo hello world"
        fluid
        size="small"
        class="text-xs"
      />
    </div>

    <!-- Completions & Parallelism Row -->
    <div class="grid grid-cols-2 gap-3">
      <div class="flex flex-col gap-1.5">
        <label for="create-job-completions" class="text-xs font-semibold text-muted-color">
          Completions
        </label>
        <InputNumber
          id="create-job-completions"
          v-model="completions"
          :min="1"
          :max="1000"
          showButtons
          fluid
          size="small"
        />
      </div>

      <div class="flex flex-col gap-1.5">
        <label for="create-job-parallelism" class="text-xs font-semibold text-muted-color">
          Parallelism
        </label>
        <InputNumber
          id="create-job-parallelism"
          v-model="parallelism"
          :min="1"
          :max="1000"
          showButtons
          fluid
          size="small"
        />
      </div>
    </div>

    <!-- Backoff Limit & Restart Policy Row -->
    <div class="grid grid-cols-2 gap-3">
      <div class="flex flex-col gap-1.5">
        <label for="create-job-backoff" class="text-xs font-semibold text-muted-color">
          Backoff Limit
        </label>
        <InputNumber
          id="create-job-backoff"
          v-model="backoffLimit"
          :min="0"
          :max="100"
          showButtons
          fluid
          size="small"
        />
      </div>

      <div class="flex flex-col gap-1.5">
        <label for="create-job-restart-policy" class="text-xs font-semibold text-muted-color">
          Restart Policy
        </label>
        <Select
          id="create-job-restart-policy"
          v-model="restartPolicy"
          :options="KUBERNETES_JOB_RESTART_POLICIES"
          fluid
          size="small"
          class="text-xs"
        />
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
