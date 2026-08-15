<script setup lang="ts">
import StringListEditor from '@/components/shared/StringListEditor.vue'
import { useKubernetesStore } from '@/stores/kubernetesStore'
import Button from 'primevue/button'
import InputText from 'primevue/inputtext'
import Select from 'primevue/select'
import { computed, inject, onMounted, ref, type Ref } from 'vue'

interface CloneDialogData {
  sourceName: string
  sourceNamespace: string
  sourceHosts: string[]
}

interface CloneDialogResult {
  newName: string
  newNamespace: string
  newHosts: string[]
}

const dialogRef = inject<
  | Ref<{
      data: CloneDialogData
      close: (data?: CloneDialogResult) => void
    }>
  | undefined
>('dialogRef')

const k8sStore = useKubernetesStore()
const namespaceOptions = computed(() => k8sStore.namespaceList.map((ns) => ns.name))

const newName = ref('')
const newNamespace = ref('')
const sourceName = ref('')
const hosts = ref<string[]>([])

onMounted(() => {
  if (dialogRef?.value?.data) {
    sourceName.value = dialogRef.value.data.sourceName
    newName.value = `${dialogRef.value.data.sourceName}-copy`
    newNamespace.value = dialogRef.value.data.sourceNamespace
    hosts.value = [...dialogRef.value.data.sourceHosts]
  }
})

const isValid = computed(() => newName.value.trim() !== '' && newNamespace.value !== '')

const handleCancel = () => {
  dialogRef?.value?.close()
}

const handleClone = () => {
  if (!isValid.value) return
  dialogRef?.value?.close({
    newName: newName.value.trim(),
    newNamespace: newNamespace.value,
    newHosts: hosts.value.map((h) => h.trim())
  })
}
</script>

<template>
  <form @submit.prevent="handleClone" class="flex flex-col gap-3.5">
    <p class="text-xs text-muted-color">
      Cloning
      <span class="font-semibold text-primary">{{ sourceName }}</span>
      into a new Ingress:
    </p>

    <div class="flex flex-col gap-1.5">
      <label for="clone-ingress-name" class="text-xs font-semibold text-muted-color"
        >New Name</label
      >
      <InputText
        id="clone-ingress-name"
        v-model="newName"
        fluid
        size="small"
        :invalid="!newName.trim()"
        class="text-xs"
      />
    </div>

    <div class="flex flex-col gap-1.5">
      <label for="clone-ingress-namespace" class="text-xs font-semibold text-muted-color">
        Namespace
      </label>
      <Select
        id="clone-ingress-namespace"
        v-model="newNamespace"
        :options="namespaceOptions"
        fluid
        size="small"
        class="text-xs"
      />
    </div>

    <StringListEditor
      v-model="hosts"
      title="Hosts"
      add-label="Add Host"
      placeholder="e.g. example.com"
    />

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
