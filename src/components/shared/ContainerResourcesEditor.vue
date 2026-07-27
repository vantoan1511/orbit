<script setup lang="ts">
import InputText from 'primevue/inputtext'

withDefaults(
  defineProps<{
    cpuRequest: string
    memoryRequest: string
    cpuLimit: string
    memoryLimit: string
    disabled?: boolean
  }>(),
  {
    disabled: false
  }
)

const emit = defineEmits<{
  (e: 'update:cpuRequest', val: string): void
  (e: 'update:memoryRequest', val: string): void
  (e: 'update:cpuLimit', val: string): void
  (e: 'update:memoryLimit', val: string): void
}>()
</script>

<template>
  <div class="flex flex-col gap-3">
    <h3 class="text-xs font-semibold text-(--text-primary) uppercase tracking-wider">
      Resource Requests & Limits
    </h3>

    <div class="grid grid-cols-2 gap-4">
      <div class="flex flex-col gap-1.5">
        <label class="text-xs font-medium text-(--text-secondary)">CPU Request</label>
        <InputText
          :model-value="cpuRequest"
          placeholder="e.g. 100m or 0.5"
          :disabled="disabled"
          class="px-2.5 py-1.5 bg-(--bg-primary) border border-(--border) rounded-md text-xs text-(--text-primary)"
          @update:model-value="(val) => emit('update:cpuRequest', val ?? '')"
        />
      </div>

      <div class="flex flex-col gap-1.5">
        <label class="text-xs font-medium text-(--text-secondary)">Memory Request</label>
        <InputText
          :model-value="memoryRequest"
          placeholder="e.g. 128Mi or 1Gi"
          :disabled="disabled"
          class="px-2.5 py-1.5 bg-(--bg-primary) border border-(--border) rounded-md text-xs text-(--text-primary)"
          @update:model-value="(val) => emit('update:memoryRequest', val ?? '')"
        />
      </div>

      <div class="flex flex-col gap-1.5">
        <label class="text-xs font-medium text-(--text-secondary)">CPU Limit</label>
        <InputText
          :model-value="cpuLimit"
          placeholder="e.g. 500m or 1"
          :disabled="disabled"
          class="px-2.5 py-1.5 bg-(--bg-primary) border border-(--border) rounded-md text-xs text-(--text-primary)"
          @update:model-value="(val) => emit('update:cpuLimit', val ?? '')"
        />
      </div>

      <div class="flex flex-col gap-1.5">
        <label class="text-xs font-medium text-(--text-secondary)">Memory Limit</label>
        <InputText
          :model-value="memoryLimit"
          placeholder="e.g. 512Mi or 2Gi"
          :disabled="disabled"
          class="px-2.5 py-1.5 bg-(--bg-primary) border border-(--border) rounded-md text-xs text-(--text-primary)"
          @update:model-value="(val) => emit('update:memoryLimit', val ?? '')"
        />
      </div>
    </div>
  </div>
</template>
