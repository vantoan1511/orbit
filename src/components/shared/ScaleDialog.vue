<script setup lang="ts">
import Button from 'primevue/button'
import InputNumber from 'primevue/inputnumber'
import { inject, onMounted, ref } from 'vue'

const dialogRef = inject<any>('dialogRef')
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
  <div class="flex flex-col gap-4 p-1">
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

    <div class="scale-dialog-footer">
      <Button label="Cancel" severity="secondary" @click="handleCancel" />
      <Button label="Scale" severity="primary" @click="handleScale" />
    </div>
  </div>
</template>
