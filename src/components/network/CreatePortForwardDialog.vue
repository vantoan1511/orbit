<script setup lang="ts">
import { KUBERNETES_RESOURCE_KIND } from '@/constants/kubernetes'
import { kubernetesService } from '@/services/kubernetesService'
import { useKubernetesStore } from '@/stores/kubernetesStore'
import { getAvailablePorts } from '@/utils/portForward'
import Button from 'primevue/button'
import InputNumber from 'primevue/inputnumber'
import Select from 'primevue/select'
import { useToast } from 'primevue/usetoast'
import { computed, inject, ref, watch, type Ref } from 'vue'

const dialogRef = inject<
  | Ref<{
      close: () => void
    }>
  | undefined
>('dialogRef')

const toast = useToast()
const k8sStore = useKubernetesStore()

const supportedKinds = [
  KUBERNETES_RESOURCE_KIND.Service,
  KUBERNETES_RESOURCE_KIND.Pod,
  KUBERNETES_RESOURCE_KIND.Deployment,
  KUBERNETES_RESOURCE_KIND.StatefulSet,
  KUBERNETES_RESOURCE_KIND.ReplicaSet
]

const namespaceOptions = computed(() => {
  const list = k8sStore.namespaceList.map((ns) => ns.name)
  if (list.length === 0) return ['default']
  return list
})

const namespace = ref<string>(
  namespaceOptions.value.includes('default') ? 'default' : (namespaceOptions.value[0] ?? 'default')
)

const kind = ref<string>(KUBERNETES_RESOURCE_KIND.Service)
const selectedResourceName = ref<string>('')
const availablePorts = ref<number[]>([])
const remotePort = ref<number | null>(null)
const localPort = ref<number | null>(null)
const isSubmitting = ref(false)

const resourceOptions = computed<string[]>(() => {
  const currentNs = namespace.value
  const currentKind = kind.value

  switch (currentKind) {
    case KUBERNETES_RESOURCE_KIND.Service:
      return k8sStore.services
        .filter((s) => !currentNs || s.namespace === currentNs)
        .map((s) => s.name)
    case KUBERNETES_RESOURCE_KIND.Pod:
      return k8sStore.pods.filter((p) => !currentNs || p.namespace === currentNs).map((p) => p.name)
    case KUBERNETES_RESOURCE_KIND.Deployment:
      return k8sStore.deployments
        .filter((d) => !currentNs || d.namespace === currentNs)
        .map((d) => d.name)
    case KUBERNETES_RESOURCE_KIND.StatefulSet:
      return k8sStore.statefulSets
        .filter((s) => !currentNs || s.namespace === currentNs)
        .map((s) => s.name)
    case KUBERNETES_RESOURCE_KIND.ReplicaSet:
      return k8sStore.replicaSets
        .filter((r) => !currentNs || r.namespace === currentNs)
        .map((r) => r.name)
    default:
      return []
  }
})

// When namespace or kind changes, reset resource selection
watch([namespace, kind], () => {
  selectedResourceName.value = ''
  availablePorts.value = []
  remotePort.value = null
  localPort.value = null
})

// When selected resource changes, auto-detect available ports
watch(selectedResourceName, (newName) => {
  if (!newName) {
    availablePorts.value = []
    remotePort.value = null
    localPort.value = null
    return
  }

  const currentNs = namespace.value
  const currentKind = kind.value
  let targetObj: unknown = null

  switch (currentKind) {
    case KUBERNETES_RESOURCE_KIND.Service:
      targetObj = k8sStore.services.find(
        (s) => (!currentNs || s.namespace === currentNs) && s.name === newName
      )
      break
    case KUBERNETES_RESOURCE_KIND.Pod:
      targetObj = k8sStore.pods.find(
        (p) => (!currentNs || p.namespace === currentNs) && p.name === newName
      )
      break
    case KUBERNETES_RESOURCE_KIND.Deployment:
      targetObj = k8sStore.deployments.find(
        (d) => (!currentNs || d.namespace === currentNs) && d.name === newName
      )
      break
    case KUBERNETES_RESOURCE_KIND.StatefulSet:
      targetObj = k8sStore.statefulSets.find(
        (s) => (!currentNs || s.namespace === currentNs) && s.name === newName
      )
      break
    case KUBERNETES_RESOURCE_KIND.ReplicaSet:
      targetObj = k8sStore.replicaSets.find(
        (r) => (!currentNs || r.namespace === currentNs) && r.name === newName
      )
      break
  }

  const extracted = getAvailablePorts(targetObj, currentKind)
  availablePorts.value = extracted

  if (extracted.length > 0) {
    const defaultPort = extracted[0] ?? 8080
    remotePort.value = defaultPort
    localPort.value = defaultPort
  } else {
    remotePort.value = 8080
    localPort.value = 8080
  }
})

const isLocalPortInUse = computed(() => {
  if (localPort.value === null) return false
  return k8sStore.activePortForwards.some((pf) => pf.localPort === localPort.value)
})

const isValid = computed(() => {
  return (
    selectedResourceName.value.trim() !== '' &&
    remotePort.value !== null &&
    remotePort.value >= 1 &&
    remotePort.value <= 65535 &&
    localPort.value !== null &&
    localPort.value >= 1 &&
    localPort.value <= 65535 &&
    !isLocalPortInUse.value
  )
})

const handleCancel = () => {
  dialogRef?.value?.close()
}

const handleStart = async () => {
  if (!isValid.value || localPort.value === null || remotePort.value === null) return

  isSubmitting.value = true
  try {
    await kubernetesService.startPortForward({
      namespace: namespace.value,
      kind: kind.value,
      name: selectedResourceName.value,
      localPort: localPort.value,
      remotePort: remotePort.value
    })

    toast.add({
      severity: 'success',
      summary: 'Port Forward Started',
      detail: `Forwarding 127.0.0.1:${localPort.value} -> ${remotePort.value} for ${selectedResourceName.value}`,
      life: 5000
    })

    dialogRef?.value?.close()
  } catch (err) {
    toast.add({
      severity: 'error',
      summary: 'Port Forward Failed',
      detail: err instanceof Error ? err.message : 'Failed to start port forwarding',
      life: 5000
    })
  } finally {
    isSubmitting.value = false
  }
}
</script>

<template>
  <form class="flex flex-col gap-4" @submit.prevent="handleStart">
    <p class="text-xs text-muted-color">
      Configure a local port forwarding tunnel to a Kubernetes resource:
    </p>

    <!-- Namespace & Kind -->
    <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
      <div class="flex flex-col gap-1.5">
        <label for="pf-namespace" class="text-xs font-semibold text-muted-color">
          Namespace <span class="text-(--danger)">*</span>
        </label>
        <Select
          id="pf-namespace"
          v-model="namespace"
          :options="namespaceOptions"
          placeholder="Select Namespace"
          fluid
          size="small"
        />
      </div>

      <div class="flex flex-col gap-1.5">
        <label for="pf-kind" class="text-xs font-semibold text-muted-color">
          Resource Kind <span class="text-(--danger)">*</span>
        </label>
        <Select
          id="pf-kind"
          v-model="kind"
          :options="supportedKinds"
          placeholder="Select Kind"
          fluid
          size="small"
        />
      </div>
    </div>

    <!-- Resource Name -->
    <div class="flex flex-col gap-1.5">
      <label for="pf-resource" class="text-xs font-semibold text-muted-color">
        Target Resource <span class="text-(--danger)">*</span>
      </label>
      <Select
        id="pf-resource"
        v-model="selectedResourceName"
        :options="resourceOptions"
        :disabled="resourceOptions.length === 0"
        filter
        :placeholder="
          resourceOptions.length === 0
            ? `No ${kind.toLowerCase()}s in this namespace`
            : `Select ${kind.toLowerCase()}...`
        "
        fluid
        size="small"
      />
    </div>

    <!-- Ports -->
    <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
      <div class="flex flex-col gap-1.5">
        <label for="pf-remote-port" class="text-xs font-semibold text-muted-color">
          Container Port (Remote) <span class="text-(--danger)">*</span>
        </label>
        <Select
          v-if="availablePorts.length > 0"
          id="pf-remote-port"
          v-model="remotePort"
          :options="availablePorts"
          editable
          placeholder="8080"
          fluid
          size="small"
          class="text-xs font-mono"
        />
        <InputNumber
          v-else
          id="pf-remote-port"
          v-model="remotePort"
          :min="1"
          :max="65535"
          :useGrouping="false"
          placeholder="8080"
          fluid
          size="small"
          class="text-xs font-mono"
        />
      </div>

      <div class="flex flex-col gap-1.5">
        <label for="pf-local-port" class="text-xs font-semibold text-muted-color">
          Host Port (Local) <span class="text-(--danger)">*</span>
        </label>
        <InputNumber
          id="pf-local-port"
          v-model="localPort"
          :min="1"
          :max="65535"
          :useGrouping="false"
          placeholder="8080"
          fluid
          size="small"
          class="text-xs font-mono"
          :invalid="isLocalPortInUse"
        />
        <small v-if="isLocalPortInUse" class="text-(--danger) text-[11px] leading-tight">
          Port {{ localPort }} is already forwarded.
        </small>
      </div>
    </div>

    <!-- Actions -->
    <div class="flex justify-end gap-2 pt-2">
      <Button
        type="button"
        label="Cancel"
        severity="secondary"
        variant="text"
        size="small"
        @click="handleCancel"
      />
      <Button
        type="submit"
        label="Start Forwarding"
        icon="pi pi-play"
        severity="primary"
        size="small"
        :loading="isSubmitting"
        :disabled="!isValid || isSubmitting"
      />
    </div>
  </form>
</template>
