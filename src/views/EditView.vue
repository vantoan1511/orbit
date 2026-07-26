<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from 'vue'
import { useRouter } from 'vue-router'
import { kubernetesService } from '@/services/kubernetesService'
import { events } from '@/services/nativeService'
import { OrbitEvents } from '@/types/events'
import { useToast } from 'primevue/usetoast'
import * as yaml from 'yaml'
import VueMonacoEditor from '@guolao/vue-monaco-editor'

import DeploymentEditForm from '@/components/workloads/DeploymentEditForm.vue'

const props = defineProps<{
  kind: string
  namespace: string
  name: string
}>()

const router = useRouter()
const toast = useToast()

const rawData = ref<Record<string, unknown> | null>(null)
const yamlContent = ref('')
const isLoading = ref(true)
const isSaving = ref(false)

// Simple form fields
const formValues = ref({
  name: '',
  namespace: '',
  labels: [] as { key: string; value: string }[],
  annotations: [] as { key: string; value: string }[]
})

let eventHandler: (data: { name: string; kind: string; data: Record<string, unknown> }) => void
let errorHandler: (data: { message: string }) => void
let successHandler: (data: { message?: string }) => void

onMounted(() => {
  eventHandler = (data: { name: string; kind: string; data: Record<string, unknown> }) => {
    if (data.name === props.name && data.kind === props.kind) {
      rawData.value = data.data
      yamlContent.value = yaml.stringify(rawData.value)

      // Populate basic form fields
      const metadata = (rawData.value?.metadata as Record<string, unknown>) || {}
      formValues.value.name = typeof metadata.name === 'string' ? metadata.name : ''
      formValues.value.namespace = typeof metadata.namespace === 'string' ? metadata.namespace : ''

      const lbls = (metadata.labels as Record<string, unknown>) || {}
      formValues.value.labels = Object.entries(lbls).map(([key, value]) => ({
        key,
        value: String(value)
      }))

      const anns = (metadata.annotations as Record<string, unknown>) || {}
      formValues.value.annotations = Object.entries(anns).map(([key, value]) => ({
        key,
        value: String(value)
      }))

      isLoading.value = false
    }
  }

  errorHandler = (data: { message: string }) => {
    toast.add({ severity: 'error', summary: 'Error', detail: data.message, life: 5000 })
    isSaving.value = false
  }

  successHandler = (data: { message?: string }) => {
    toast.add({
      severity: 'success',
      summary: 'Success',
      detail: data.message || 'Resource updated successfully',
      life: 3000
    })
    isSaving.value = false
    router.back()
  }

  events.on(OrbitEvents.ResourceRawData, eventHandler)
  events.on(OrbitEvents.ErrorOccurred, errorHandler)
  events.on(OrbitEvents.CommandSucceeded, successHandler)

  // Fetch initial data
  kubernetesService.getResourceRaw({
    namespace: props.namespace,
    kind: props.kind,
    name: props.name
  })
})

onUnmounted(() => {
  events.off(OrbitEvents.ResourceRawData, eventHandler)
  events.off(OrbitEvents.ErrorOccurred, errorHandler)
  events.off(OrbitEvents.CommandSucceeded, successHandler)
})

const goBack = () => {
  router.back()
}

const saveChanges = () => {
  isSaving.value = true
  try {
    const updatedData = yaml.parse(yamlContent.value) as Record<string, unknown>
    kubernetesService.applyResource({
      namespace: props.namespace,
      kind: props.kind,
      name: props.name,
      data: updatedData
    })
  } catch (err) {
    toast.add({ severity: 'error', summary: 'YAML Parse Error', detail: String(err), life: 5000 })
    isSaving.value = false
  }
}

const handleDeploymentFormUpdate = (updatedData: Record<string, unknown>) => {
  rawData.value = updatedData
  yamlContent.value = yaml.stringify(updatedData)
}

// Watch form fields to sync to YAML for non-Deployment resources
watch(
  () => formValues.value,
  (newVal) => {
    if (!rawData.value || props.kind === 'Deployment') return
    try {
      const currentData = yaml.parse(yamlContent.value) || {}
      if (!currentData.metadata) currentData.metadata = {}

      currentData.metadata.labels = newVal.labels.reduce(
        (acc, { key, value }) => {
          if (key) acc[key] = value
          return acc
        },
        {} as Record<string, string>
      )

      currentData.metadata.annotations = newVal.annotations.reduce(
        (acc, { key, value }) => {
          if (key) acc[key] = value
          return acc
        },
        {} as Record<string, string>
      )

      yamlContent.value = yaml.stringify(currentData)
    } catch {
      // Ignore parse errors while typing
    }
  },
  { deep: true }
)
</script>

<template>
  <div class="flex flex-col h-full bg-(--bg-card)">
    <!-- Header -->
    <div class="flex items-center justify-between p-4 border-b border-(--border)">
      <div class="flex items-center gap-3">
        <button
          @click="goBack"
          class="p-2 hover:bg-(--bg-hover) rounded-md transition-colors cursor-pointer"
        >
          <i class="pi pi-arrow-left text-(--text-secondary)"></i>
        </button>
        <div>
          <h1 class="text-lg font-semibold text-(--text-primary)">Edit {{ props.kind }}</h1>
          <p class="text-sm text-(--text-secondary)">{{ props.namespace }} / {{ props.name }}</p>
        </div>
      </div>
      <div class="flex items-center gap-2">
        <button
          @click="goBack"
          class="px-4 py-2 text-sm font-medium text-(--text-secondary) hover:text-(--text-primary) transition-colors cursor-pointer"
        >
          Cancel
        </button>
        <button
          @click="saveChanges"
          :disabled="isSaving || isLoading"
          class="px-4 py-2 text-sm font-medium bg-blue-600 text-white rounded-md hover:bg-blue-700 disabled:opacity-50 transition-colors cursor-pointer"
        >
          {{ isSaving ? 'Saving...' : 'Save Changes' }}
        </button>
      </div>
    </div>

    <!-- Content -->
    <div v-if="isLoading" class="flex-1 flex items-center justify-center">
      <i class="pi pi-spin pi-spinner text-3xl text-(--text-secondary)"></i>
    </div>
    <div v-else class="flex-1 flex overflow-hidden">
      <!-- Left side: Form -->
      <div
        v-if="props.kind === 'Deployment'"
        class="w-1/2 min-w-105 max-w-160 border-r border-(--border) overflow-y-auto flex flex-col"
      >
        <DeploymentEditForm :raw-data="rawData" @update:raw-data="handleDeploymentFormUpdate" />
      </div>

      <div
        v-else
        class="w-1/3 min-w-75 border-r border-(--border) overflow-y-auto p-4 flex flex-col gap-6"
      >
        <div>
          <h2 class="text-sm font-semibold text-(--text-primary) mb-3 uppercase tracking-wider">
            Metadata
          </h2>

          <div class="flex flex-col gap-4">
            <div class="flex flex-col gap-1.5">
              <label class="text-xs font-medium text-(--text-secondary)">Name</label>
              <input
                type="text"
                disabled
                v-model="formValues.name"
                class="px-3 py-2 bg-(--bg-primary) border border-(--border) rounded-md text-sm text-(--text-secondary) opacity-70"
              />
            </div>

            <div class="flex flex-col gap-1.5">
              <label class="text-xs font-medium text-(--text-secondary)">Namespace</label>
              <input
                type="text"
                disabled
                v-model="formValues.namespace"
                class="px-3 py-2 bg-(--bg-primary) border border-(--border) rounded-md text-sm text-(--text-secondary) opacity-70"
              />
            </div>

            <!-- Labels -->
            <div class="flex flex-col gap-2 mt-2">
              <div class="flex items-center justify-between">
                <label class="text-xs font-medium text-(--text-secondary)">Labels</label>
                <button
                  @click="formValues.labels.push({ key: '', value: '' })"
                  class="text-xs text-blue-500 hover:text-blue-400 cursor-pointer flex items-center gap-1"
                >
                  <i class="pi pi-plus text-[10px]"></i> Add
                </button>
              </div>
              <div
                v-for="(lbl, idx) in formValues.labels"
                :key="'lbl-' + idx"
                class="flex items-center gap-2"
              >
                <input
                  type="text"
                  v-model="lbl.key"
                  placeholder="Key"
                  class="w-1/2 px-2 py-1.5 bg-(--bg-primary) border border-(--border) rounded-md text-xs text-(--text-primary)"
                />
                <span class="text-(--text-secondary)">=</span>
                <input
                  type="text"
                  v-model="lbl.value"
                  placeholder="Value"
                  class="w-1/2 px-2 py-1.5 bg-(--bg-primary) border border-(--border) rounded-md text-xs text-(--text-primary)"
                />
                <button
                  @click="formValues.labels.splice(idx, 1)"
                  class="p-1 text-red-400 hover:text-red-300 cursor-pointer"
                >
                  <i class="pi pi-times text-xs"></i>
                </button>
              </div>
            </div>

            <!-- Annotations -->
            <div class="flex flex-col gap-2 mt-2">
              <div class="flex items-center justify-between">
                <label class="text-xs font-medium text-(--text-secondary)">Annotations</label>
                <button
                  @click="formValues.annotations.push({ key: '', value: '' })"
                  class="text-xs text-blue-500 hover:text-blue-400 cursor-pointer flex items-center gap-1"
                >
                  <i class="pi pi-plus text-[10px]"></i> Add
                </button>
              </div>
              <div
                v-for="(ann, idx) in formValues.annotations"
                :key="'ann-' + idx"
                class="flex items-center gap-2"
              >
                <input
                  type="text"
                  v-model="ann.key"
                  placeholder="Key"
                  class="w-1/2 px-2 py-1.5 bg-(--bg-primary) border border-(--border) rounded-md text-xs text-(--text-primary)"
                />
                <span class="text-(--text-secondary)">=</span>
                <input
                  type="text"
                  v-model="ann.value"
                  placeholder="Value"
                  class="w-1/2 px-2 py-1.5 bg-(--bg-primary) border border-(--border) rounded-md text-xs text-(--text-primary)"
                />
                <button
                  @click="formValues.annotations.splice(idx, 1)"
                  class="p-1 text-red-400 hover:text-red-300 cursor-pointer"
                >
                  <i class="pi pi-times text-xs"></i>
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Right side: YAML Editor -->
      <div class="flex-1 h-full bg-[#1e1e1e]">
        <vue-monaco-editor
          v-model:value="yamlContent"
          theme="vs-dark"
          language="yaml"
          :options="{
            minimap: { enabled: false },
            fontSize: 13,
            lineHeight: 22,
            scrollBeyondLastLine: false,
            roundedSelection: false,
            padding: { top: 16 },
            readOnly: isSaving
          }"
          class="h-full w-full"
        />
      </div>
    </div>
  </div>
</template>
