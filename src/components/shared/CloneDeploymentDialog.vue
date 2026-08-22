<script setup lang="ts">
import { useKubernetesStore } from '@/stores/kubernetesStore'
import { isValidK8sName } from '@/utils/validators'
import Button from 'primevue/button'
import InputText from 'primevue/inputtext'
import Select from 'primevue/select'
import { computed, inject, onMounted, ref, type Ref } from 'vue'

interface CloneDialogData {
  sourceName: string
  sourceNamespace: string
}

interface CloneDialogResult {
  newName: string
  newNamespace: string
}

const dialogRef = inject<
  | Ref<{
      data: CloneDialogData
      close: (data?: CloneDialogResult) => void
    }>
  | undefined
>('dialogRef')

const k8sStore = useKubernetesStore()
const namespaceOptions = computed(() => {
  const list = k8sStore.namespaceList.map((ns) => ns.name)
  if (list.length === 0) return ['default']
  return list
})

const newName = ref('')
const newNamespace = ref('')
const sourceName = ref('')

onMounted(() => {
  if (dialogRef?.value?.data) {
    sourceName.value = dialogRef.value.data.sourceName
    newName.value = `${dialogRef.value.data.sourceName}-copy`
    newNamespace.value = dialogRef.value.data.sourceNamespace || 'default'
  }
})

const nameErrorMessage = computed(() => {
  const trimmed = newName.value.trim()
  if (!trimmed) return null
  if (!isValidK8sName(trimmed)) {
    return 'Name must be a valid DNS-1123 subdomain (lowercase letters, numbers, hyphens, dots).'
  }
  const exists = k8sStore.deployments.some(
    (dep) =>
      dep.namespace === newNamespace.value && dep.name.toLowerCase() === trimmed.toLowerCase()
  )
  if (exists) {
    return `A Deployment named "${trimmed}" already exists in namespace "${newNamespace.value}".`
  }
  return null
})

const isValid = computed(
  () => Boolean(newName.value.trim()) && !nameErrorMessage.value && Boolean(newNamespace.value)
)

const handleCancel = () => {
  dialogRef?.value?.close()
}

const handleClone = () => {
  if (!isValid.value) return
  dialogRef?.value?.close({
    newName: newName.value.trim(),
    newNamespace: newNamespace.value
  })
}
</script>

<template>
  <form @submit.prevent="handleClone" class="flex flex-col gap-3.5">
    <p class="text-xs text-muted-color">
      Cloning
      <span class="font-semibold text-primary">{{ sourceName }}</span>
      into a new Deployment:
    </p>

    <div class="flex flex-col gap-1.5">
      <label for="clone-deployment-name" class="text-xs font-semibold text-muted-color">
        New Name
      </label>
      <InputText
        id="clone-deployment-name"
        v-model="newName"
        fluid
        size="small"
        :invalid="Boolean(newName.trim() && nameErrorMessage)"
        class="text-xs"
      />
      <small
        v-if="newName.trim() && nameErrorMessage"
        class="text-(--danger) text-[11px] leading-tight"
      >
        {{ nameErrorMessage }}
      </small>
    </div>

    <div class="flex flex-col gap-1.5">
      <label for="clone-deployment-namespace" class="text-xs font-semibold text-muted-color">
        Namespace
      </label>
      <Select
        id="clone-deployment-namespace"
        v-model="newNamespace"
        :options="namespaceOptions"
        fluid
        size="small"
        class="text-xs"
      />
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
      <Button type="submit" label="Clone" severity="primary" size="small" :disabled="!isValid" />
    </div>
  </form>
</template>
