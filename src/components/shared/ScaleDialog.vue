<script setup lang="ts">
import Button from 'primevue/button'
import InputNumber from 'primevue/inputnumber'
import { inject, onMounted, ref, type Ref } from 'vue'

const dialogRef = inject<
  | Ref<{
      data: {
        currentReplicas?: number
        name?: string
        kind?: string
      }
      close: (data?: number) => void
    }>
  | undefined
>('dialogRef')
const replicas = ref<number>(1)
const resourceName = ref<string>('')
const resourceKind = ref<string>('')

onMounted(() => {
  if (dialogRef?.value?.data) {
    replicas.value = dialogRef.value.data.currentReplicas ?? 1
    resourceName.value = dialogRef.value.data.name ?? ''
    resourceKind.value = dialogRef.value.data.kind ?? ''
  }
})

const handleCancel = () => {
  dialogRef?.value?.close()
}

const handleScale = () => {
  dialogRef?.value?.close(replicas.value)
}
</script>

<template>
  <form @submit.prevent="handleScale" class="flex flex-col gap-4 p-1">
    <p class="text-xs text-(--text-secondary)">
      Scale <span class="font-semibold text-(--text-primary)">{{ resourceName }}</span> ({{
        resourceKind
      }}) to the desired number of replicas:
    </p>

    <div class="flex flex-col gap-2">
      <label for="replicas" class="text-xs font-semibold text-(--text-secondary)">Replicas</label>
      <InputNumber
        v-model="replicas"
        inputId="replicas"
        :min="0"
        :max="100"
        showButtons
        class="w-full"
        inputClass="text-sm bg-(--bg-card) border-(--border) text-center"
      />
    </div>

    <div class="flex justify-end gap-2 mt-4">
      <Button
        type="button"
        label="Cancel"
        severity="secondary"
        size="small"
        @click="handleCancel"
      />
      <Button type="submit" label="Scale" severity="primary" size="small" />
    </div>
  </form>
</template>
