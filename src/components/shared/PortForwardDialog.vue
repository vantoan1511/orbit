<script setup lang="ts">
import Select from 'primevue/select'
import { computed, inject, onMounted, ref, type Ref } from 'vue'

interface PortForwardDialogData {
  sourceName: string
  sourceNamespace: string
  kind: string
  availablePorts?: number[]
}

export interface PortForwardDialogResult {
  localPort: number
  remotePort: number
}

const dialogRef = inject<
  | Ref<{
      data: PortForwardDialogData
      close: (data?: PortForwardDialogResult) => void
    }>
  | undefined
>('dialogRef')

const sourceName = ref('')
const sourceNamespace = ref('')
const kind = ref('Deployment')

const localPort = ref<number | null>(8080)
const availablePorts = ref<number[]>([])
const remotePortInput = ref<string | number | null>(8080)

const remotePort = computed<number | null>(() => {
  if (remotePortInput.value === null || remotePortInput.value === '') return null
  const parsed = Number(remotePortInput.value)
  return isNaN(parsed) ? null : parsed
})

const remotePortNumber = computed({
  get: () => remotePort.value,
  set: (val: number | null) => {
    remotePortInput.value = val
  }
})

onMounted(() => {
  if (dialogRef?.value?.data) {
    sourceName.value = dialogRef.value.data.sourceName
    sourceNamespace.value = dialogRef.value.data.sourceNamespace || 'default'
    kind.value = dialogRef.value.data.kind || 'Deployment'

    if (dialogRef.value.data.availablePorts && dialogRef.value.data.availablePorts.length > 0) {
      availablePorts.value = dialogRef.value.data.availablePorts
      const firstPort = availablePorts.value[0] ?? 8080
      remotePortInput.value = firstPort
      localPort.value = firstPort
    }
  }
})

const isValid = computed(() => {
  return (
    localPort.value !== null &&
    localPort.value >= 1 &&
    localPort.value <= 65535 &&
    remotePort.value !== null &&
    remotePort.value >= 1 &&
    remotePort.value <= 65535
  )
})

const handleCancel = () => {
  dialogRef?.value?.close()
}

const handleStart = () => {
  if (!isValid.value || localPort.value === null || remotePort.value === null) return
  dialogRef?.value?.close({
    localPort: localPort.value,
    remotePort: remotePort.value
  })
}
</script>

<template>
  <form @submit.prevent="handleStart" class="flex flex-col gap-4">
    <p class="text-xs text-muted-color">
      Forwarding ports for
      <span class="font-mono text-primary font-semibold"
        >{{ kind.toLowerCase() }}/{{ sourceName }}</span
      >
      in namespace
      <span class="font-mono text-primary font-semibold">{{ sourceNamespace }}</span
      >:
    </p>

    <div class="grid grid-cols-2 gap-3">
      <div class="flex flex-col gap-1.5">
        <label for="host-port" class="text-xs font-semibold text-muted-color">
          Host Port (Local)
        </label>
        <InputNumber
          id="host-port"
          v-model="localPort"
          :min="1"
          :max="65535"
          :useGrouping="false"
          placeholder="8080"
          fluid
          size="small"
          class="text-xs font-mono"
          autofocus
        />
      </div>

      <div class="flex flex-col gap-1.5">
        <label for="container-port" class="text-xs font-semibold text-muted-color">
          Container Port (Remote)
        </label>
        <Select
          v-if="availablePorts.length > 0"
          id="container-port"
          v-model="remotePortInput"
          :options="availablePorts"
          editable
          placeholder="8080"
          fluid
          size="small"
          class="text-xs font-mono"
        />
        <InputNumber
          v-else
          id="container-port"
          v-model="remotePortNumber"
          :min="1"
          :max="65535"
          :useGrouping="false"
          placeholder="8080"
          fluid
          size="small"
          class="text-xs font-mono"
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
        @click="handleCancel"
      />
      <Button
        type="submit"
        label="Start Forwarding"
        icon="pi pi-external-link"
        severity="primary"
        size="small"
        :disabled="!isValid"
      />
    </div>
  </form>
</template>
