<script setup lang="ts">
import Button from 'primevue/button'
import InputNumber from 'primevue/inputnumber'
import { computed, inject, onMounted, ref, type Ref } from 'vue'
import { validateHpaBounds } from '@/utils/hpa.ts'

const dialogRef = inject<
  | Ref<{
      data: {
        name?: string
        namespace?: string
        minReplicas?: number
        maxReplicas?: number
      }
      close: (data?: { minReplicas: number; maxReplicas: number }) => void
    }>
  | undefined
>('dialogRef')

const name = ref<string>('')
const minReplicas = ref<number>(1)
const maxReplicas = ref<number>(10)

onMounted(() => {
  if (dialogRef?.value?.data) {
    name.value = dialogRef.value.data.name ?? ''
    minReplicas.value = dialogRef.value.data.minReplicas ?? 1
    maxReplicas.value = dialogRef.value.data.maxReplicas ?? 10
  }
})

const validation = computed(() => validateHpaBounds(minReplicas.value, maxReplicas.value))

const handleCancel = () => {
  dialogRef?.value?.close()
}

const handleSave = () => {
  if (!validation.value.valid) return
  dialogRef?.value?.close({
    minReplicas: minReplicas.value,
    maxReplicas: maxReplicas.value
  })
}
</script>

<template>
  <form @submit.prevent="handleSave" class="flex flex-col gap-4">
    <p class="text-xs text-muted-color">
      Configure replica bounds for
      <span class="font-semibold text-primary">{{ name }}</span
      >:
    </p>

    <div class="grid grid-cols-2 gap-3">
      <div class="flex flex-col gap-1.5">
        <label for="min-replicas" class="text-xs font-semibold text-muted-color"
          >Min Replicas</label
        >
        <InputNumber
          v-model="minReplicas"
          inputId="min-replicas"
          :min="1"
          :max="maxReplicas"
          showButtons
          class="w-full"
          inputClass="text-center font-mono font-medium rounded-[var(--ui-radius-control)]"
        />
      </div>

      <div class="flex flex-col gap-1.5">
        <label for="max-replicas" class="text-xs font-semibold text-muted-color"
          >Max Replicas</label
        >
        <InputNumber
          v-model="maxReplicas"
          inputId="max-replicas"
          :min="minReplicas"
          :max="1000"
          showButtons
          class="w-full"
          inputClass="text-center font-mono font-medium rounded-[var(--ui-radius-control)]"
        />
      </div>
    </div>

    <p v-if="!validation.valid" class="text-xs text-(--danger)">
      {{ validation.error }}
    </p>

    <div class="flex justify-end gap-2 pt-2">
      <Button
        type="button"
        label="Cancel"
        severity="secondary"
        variant="text"
        class="rounded-[var(--ui-radius-control)]"
        @click="handleCancel"
      />
      <Button
        type="submit"
        label="Save"
        severity="primary"
        class="rounded-[var(--ui-radius-control)]"
        :disabled="!validation.valid"
      />
    </div>
  </form>
</template>
