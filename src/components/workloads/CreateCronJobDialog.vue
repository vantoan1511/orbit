<script setup lang="ts">
import { useCreateResourceDialog } from '@/composables/useCreateResourceDialog'
import { kubernetesService } from '@/services/kubernetesService'
import {
  KUBERNETES_CONCURRENCY_POLICIES,
  KUBERNETES_CONCURRENCY_POLICY,
  KUBERNETES_JOB_RESTART_POLICIES,
  KUBERNETES_JOB_RESTART_POLICY,
  KUBERNETES_RESOURCE_KIND,
  type KubernetesConcurrencyPolicy,
  type KubernetesJobRestartPolicy
} from '@/constants/kubernetes'
import { isValidCron, isValidK8sName, sanitizeK8sLabel } from '@/utils/validators'
import type { CronJob } from 'kubernetes-types/batch/v1'
import type { Container } from 'kubernetes-types/core/v1'
import Button from 'primevue/button'
import InputText from 'primevue/inputtext'
import Select from 'primevue/select'
import ToggleSwitch from 'primevue/toggleswitch'
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
const schedule = ref('*/5 * * * *')
const image = ref('')
const command = ref('')
const concurrencyPolicy = ref<KubernetesConcurrencyPolicy>(KUBERNETES_CONCURRENCY_POLICY.Allow)
const restartPolicy = ref<KubernetesJobRestartPolicy>(KUBERNETES_JOB_RESTART_POLICY.OnFailure)
const suspend = ref(false)

const nameErrorMessage = computed(() => {
  const trimmed = name.value.trim()
  if (!trimmed) return null
  if (!isValidK8sName(trimmed)) {
    return 'Name must be a valid DNS-1123 subdomain (lowercase letters, numbers, hyphens, dots).'
  }
  if (isNameTaken(trimmed, k8sStore.cronJobs)) {
    return `A CronJob named "${trimmed}" already exists in namespace "${namespace.value}".`
  }
  return null
})

const scheduleErrorMessage = computed(() => {
  const trimmed = schedule.value.trim()
  if (!trimmed) return null
  if (!isValidCron(trimmed)) {
    return 'Invalid schedule format. Expected standard 5-part cron syntax (e.g. */5 * * * *).'
  }
  return null
})

const isFormValid = computed(() => {
  const trimmedName = name.value.trim()
  const trimmedSchedule = schedule.value.trim()
  const trimmedImage = image.value.trim()
  const hasValidName = Boolean(trimmedName) && !nameErrorMessage.value
  const hasValidNamespace = Boolean(namespace.value)
  const hasValidSchedule = Boolean(trimmedSchedule) && !scheduleErrorMessage.value
  const hasValidImage = Boolean(trimmedImage)

  return hasValidName && hasValidNamespace && hasValidSchedule && hasValidImage
})

const handleCreate = async () => {
  if (!isFormValid.value || isCreating.value) return

  const trimmedName = name.value.trim()
  const trimmedNamespace = namespace.value.trim()
  const trimmedSchedule = schedule.value.trim()
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

  const manifest: CronJob = {
    apiVersion: 'batch/v1',
    kind: KUBERNETES_RESOURCE_KIND.CronJob,
    metadata: {
      name: trimmedName,
      namespace: trimmedNamespace,
      labels: {
        app: trimmedName
      }
    },
    spec: {
      schedule: trimmedSchedule,
      suspend: suspend.value,
      concurrencyPolicy: concurrencyPolicy.value,
      jobTemplate: {
        spec: {
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
    }
  }

  submitCreation(trimmedName)
  try {
    await kubernetesService.createResource({
      namespace: trimmedNamespace,
      kind: KUBERNETES_RESOURCE_KIND.CronJob,
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
      Create a new Kubernetes CronJob to execute scheduled jobs:
    </p>

    <!-- Name -->
    <div class="flex flex-col gap-1.5">
      <label for="create-cronjob-name" class="text-xs font-semibold text-muted-color">
        Name <span class="text-(--danger)">*</span>
      </label>
      <InputText
        id="create-cronjob-name"
        v-model="name"
        placeholder="e.g. database-backup"
        fluid
        size="small"
        :invalid="Boolean(name.trim() && nameErrorMessage)"
        aria-describedby="create-cronjob-name-error"
        class="text-xs"
      />
      <small
        v-if="name.trim() && nameErrorMessage"
        id="create-cronjob-name-error"
        class="text-(--danger) text-[11px] leading-tight"
      >
        {{ nameErrorMessage }}
      </small>
    </div>

    <!-- Namespace -->
    <div class="flex flex-col gap-1.5">
      <label for="create-cronjob-namespace" class="text-xs font-semibold text-muted-color">
        Namespace <span class="text-(--danger)">*</span>
      </label>
      <Select
        id="create-cronjob-namespace"
        v-model="namespace"
        :options="namespaceOptions"
        fluid
        size="small"
        class="text-xs"
      />
    </div>

    <!-- Schedule (Cron Expression) -->
    <div class="flex flex-col gap-1.5">
      <label for="create-cronjob-schedule" class="text-xs font-semibold text-muted-color">
        Schedule (Cron Expression) <span class="text-(--danger)">*</span>
      </label>
      <InputText
        id="create-cronjob-schedule"
        v-model="schedule"
        placeholder="e.g. */5 * * * * or 0 0 * * *"
        fluid
        size="small"
        :invalid="Boolean(schedule.trim() && scheduleErrorMessage)"
        aria-describedby="create-cronjob-schedule-error"
        class="text-xs font-mono"
      />
      <small
        v-if="schedule.trim() && scheduleErrorMessage"
        id="create-cronjob-schedule-error"
        class="text-(--danger) text-[11px] leading-tight"
      >
        {{ scheduleErrorMessage }}
      </small>
      <span v-else class="text-[11px] text-muted-color">
        Standard 5-field syntax: minute, hour, day of month, month, day of week (e.g.
        <code class="font-mono">*/5 * * * *</code>)
      </span>
    </div>

    <!-- Image -->
    <div class="flex flex-col gap-1.5">
      <label for="create-cronjob-image" class="text-xs font-semibold text-muted-color">
        Image <span class="text-(--danger)">*</span>
      </label>
      <InputText
        id="create-cronjob-image"
        v-model="image"
        placeholder="e.g. curlimages/curl:latest or busybox:latest"
        fluid
        size="small"
        class="text-xs"
      />
    </div>

    <!-- Command (Optional) -->
    <div class="flex flex-col gap-1.5">
      <label for="create-cronjob-command" class="text-xs font-semibold text-muted-color">
        Command (Optional)
      </label>
      <InputText
        id="create-cronjob-command"
        v-model="command"
        placeholder="e.g. curl -s http://service-host/health"
        fluid
        size="small"
        class="text-xs"
      />
    </div>

    <!-- Concurrency Policy & Restart Policy Row -->
    <div class="grid grid-cols-2 gap-3">
      <div class="flex flex-col gap-1.5">
        <label for="create-cronjob-concurrency" class="text-xs font-semibold text-muted-color">
          Concurrency Policy
        </label>
        <Select
          id="create-cronjob-concurrency"
          v-model="concurrencyPolicy"
          :options="KUBERNETES_CONCURRENCY_POLICIES"
          fluid
          size="small"
          class="text-xs"
        />
      </div>

      <div class="flex flex-col gap-1.5">
        <label for="create-cronjob-restart-policy" class="text-xs font-semibold text-muted-color">
          Restart Policy
        </label>
        <Select
          id="create-cronjob-restart-policy"
          v-model="restartPolicy"
          :options="KUBERNETES_JOB_RESTART_POLICIES"
          fluid
          size="small"
          class="text-xs"
        />
      </div>
    </div>

    <!-- Suspend Toggle -->
    <div class="flex items-center justify-between py-1">
      <div class="flex flex-col">
        <span class="text-xs font-semibold text-muted-color">Suspend Execution</span>
        <span class="text-[11px] text-muted-color">Pause future job executions upon creation</span>
      </div>
      <ToggleSwitch v-model="suspend" />
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
