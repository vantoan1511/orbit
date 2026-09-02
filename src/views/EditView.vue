<script setup lang="ts">
import { kubernetesService } from '@/services/kubernetesService'
import { events } from '@/services/nativeService'
import { OrbitEvents } from '@/types/events'
import VueMonacoEditor from '@guolao/vue-monaco-editor'
import Button from 'primevue/button'
import InputText from 'primevue/inputtext'
import ToggleSwitch from 'primevue/toggleswitch'
import { useToast } from 'primevue/usetoast'
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import type { Component } from 'vue'
import { useRouter } from 'vue-router'
import * as yaml from 'yaml'

import { ArrowLeft, Loader2 } from '@lucide/vue'
import KeyValueEditor from '@/components/shared/KeyValueEditor.vue'
import ConfigMapEditForm from '@/components/config/ConfigMapEditForm.vue'
import SecretEditForm from '@/components/config/SecretEditForm.vue'
import IngressEditForm from '@/components/network/IngressEditForm.vue'
import ServiceEditForm from '@/components/network/ServiceEditForm.vue'
import DeploymentEditForm from '@/components/workloads/DeploymentEditForm.vue'
import PodEditForm from '@/components/workloads/PodEditForm.vue'
import NamespaceEditForm from '@/components/namespaces/NamespaceEditForm.vue'
import { useTheme } from '@/composables/useTheme'
import { KUBERNETES_RESOURCE_KIND } from '@/constants/kubernetes'
import type { KubernetesResource } from '@/types/kubernetes'

const props = defineProps<{
  kind: string
  namespace: string
  name: string
}>()

const router = useRouter()
const toast = useToast()
const { isDark } = useTheme()

const rawData = ref<KubernetesResource | null>(null)
const formComponentMap: Record<string, Component> = {
  [KUBERNETES_RESOURCE_KIND.Deployment]: DeploymentEditForm,
  [KUBERNETES_RESOURCE_KIND.Pod]: PodEditForm,
  [KUBERNETES_RESOURCE_KIND.Ingress]: IngressEditForm,
  [KUBERNETES_RESOURCE_KIND.Service]: ServiceEditForm,
  [KUBERNETES_RESOURCE_KIND.ConfigMap]: ConfigMapEditForm,
  [KUBERNETES_RESOURCE_KIND.Secret]: SecretEditForm,
  [KUBERNETES_RESOURCE_KIND.Namespace]: NamespaceEditForm
}
const activeFormComponent = computed(() => formComponentMap[props.kind] || null)
const yamlContent = ref('')
const isLoading = ref(true)
const isSaving = ref(false)
const isYamlMode = ref(false)
const isChildFormValid = ref(true)

const isYamlValid = computed(() => {
  if (!yamlContent.value.trim()) return false
  try {
    yaml.parse(yamlContent.value)
    return true
  } catch {
    return false
  }
})

const isFormValid = computed(() => {
  return isYamlMode.value ? isYamlValid.value : isChildFormValid.value
})

// Simple form fields
const formValues = ref({
  name: '',
  namespace: '',
  labels: [] as { key: string; value: string }[],
  annotations: [] as { key: string; value: string }[]
})

let eventHandler: (data: { name: string; kind: string; data: KubernetesResource }) => void
let errorHandler: (data: { message: string }) => void
let successHandler: (data: { message?: string }) => void

onMounted(() => {
  eventHandler = (data: { name: string; kind: string; data: KubernetesResource }) => {
    if (data.name === props.name && data.kind === props.kind) {
      rawData.value = data.data
      yamlContent.value = yaml.stringify(rawData.value)

      // Populate basic form fields
      const metadata = rawData.value?.metadata || {}
      formValues.value.name = metadata.name || ''
      formValues.value.namespace = metadata.namespace || ''

      const lbls = metadata.labels || {}
      formValues.value.labels = Object.entries(lbls).map(([key, value]) => ({
        key,
        value: String(value)
      }))

      const anns = metadata.annotations || {}
      formValues.value.annotations = Object.entries(anns).map(([key, value]) => ({
        key,
        value: String(value)
      }))

      isLoading.value = false
    }
  }

  errorHandler = () => {
    // Error toast is handled globally by App.vue
    isSaving.value = false
  }

  successHandler = () => {
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

const handleModeToggle = () => {
  if (!isYamlMode.value) {
    // Toggled from YAML to Form mode -> sync YAML back to rawData
    try {
      const parsed = yaml.parse(yamlContent.value) as KubernetesResource
      if (parsed && typeof parsed === 'object') {
        rawData.value = parsed
      }
    } catch (err) {
      toast.add({
        severity: 'error',
        summary: 'YAML Parse Error',
        detail: `Cannot switch to Form mode: ${err}`,
        life: 5000
      })
      isYamlMode.value = true // Keep in YAML mode so user can fix syntax
    }
  } else {
    // Toggled from Form to YAML mode -> ensure YAML string reflects latest rawData
    if (rawData.value) {
      yamlContent.value = yaml.stringify(rawData.value)
    }
  }
}

const saveChanges = () => {
  if (isSaving.value || isLoading.value) return
  isSaving.value = true
  try {
    let updatedData: KubernetesResource
    if (isYamlMode.value) {
      updatedData = yaml.parse(yamlContent.value) as KubernetesResource
    } else {
      updatedData = rawData.value || {}
    }
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

const handleCustomFormUpdate = (updatedData: KubernetesResource) => {
  rawData.value = updatedData
  yamlContent.value = yaml.stringify(updatedData)
}

// Watch form fields to sync to YAML for fallback non-custom resources
watch(
  () => formValues.value,
  (newVal) => {
    if (!rawData.value || activeFormComponent.value) return
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
  <div class="flex flex-col h-[calc(100vh-8rem)]">
    <!-- Control Bar (Tone Shift) -->
    <div
      class="flex items-center justify-between px-4 py-3 rounded-lg bg-(--bg-hover)/40 shrink-0 mb-6"
    >
      <div class="flex items-center gap-3">
        <Button
          rounded
          variant="text"
          severity="secondary"
          size="small"
          aria-label="Back"
          @click="goBack"
        >
          <template #icon>
            <ArrowLeft class="w-4 h-4" />
          </template>
        </Button>
        <div class="flex flex-col">
          <h1 class="text-lg font-bold tracking-tight text-primary leading-tight">
            Edit {{ props.kind }}
          </h1>
          <span class="text-xs font-medium text-muted-color font-mono">
            {{ props.namespace }} / {{ props.name }}
          </span>
        </div>
      </div>

      <!-- Mode Toggle & Action Buttons -->
      <div class="flex items-center gap-6">
        <div class="flex items-center gap-2.5">
          <span
            class="text-xs font-medium transition-colors"
            :class="!isYamlMode ? 'text-primary font-semibold' : 'text-muted-color'"
          >
            Form
          </span>
          <ToggleSwitch v-model="isYamlMode" @change="handleModeToggle" />
          <span
            class="text-xs font-medium transition-colors"
            :class="isYamlMode ? 'text-primary font-semibold' : 'text-muted-color'"
          >
            YAML
          </span>
        </div>

        <div class="flex items-center gap-2">
          <Button label="Cancel" variant="text" severity="secondary" size="small" @click="goBack" />
          <Button
            size="small"
            :loading="isSaving"
            :disabled="isSaving || isLoading || !isFormValid"
            :label="isSaving ? 'Saving...' : 'Save Changes'"
            @click="saveChanges"
          />
        </div>
      </div>
    </div>

    <!-- Content Area -->
    <div v-if="isLoading" class="flex-1 flex items-center justify-center">
      <Loader2 class="w-8 h-8 text-muted-color animate-spin" />
    </div>
    <div v-else class="flex-1 min-h-0 flex flex-col overflow-hidden">
      <!-- Form Mode -->
      <template v-if="!isYamlMode">
        <div v-if="activeFormComponent" class="w-full h-full overflow-hidden flex flex-col">
          <component
            :is="activeFormComponent"
            :raw-data="rawData"
            @update:raw-data="handleCustomFormUpdate"
            @update:is-valid="(val: boolean) => (isChildFormValid = val)"
          />
        </div>

        <!-- Non-Deployment Fallback Form with Asymmetric 2-Column Section Layout -->
        <div v-else class="w-full h-full overflow-y-auto pt-2 px-0 flex flex-col gap-10">
          <!-- Section 1: Resource Identity -->
          <div class="grid grid-cols-1 md:grid-cols-12 gap-6 max-w-7xl">
            <div class="md:col-span-4 flex flex-col gap-1">
              <span class="text-xs font-semibold tracking-wider text-muted-color uppercase">
                Resource Identity
              </span>
              <p class="text-xs text-muted-color leading-relaxed">
                Core identity parameters defined in metadata. These fields are immutable.
              </p>
            </div>
            <div class="md:col-span-8 flex flex-col gap-3">
              <div class="flex flex-col gap-1.5">
                <label class="text-xs font-medium text-muted-color">Resource Name</label>
                <InputText disabled v-model="formValues.name" size="small" fluid />
              </div>
              <div class="flex flex-col gap-1.5">
                <label class="text-xs font-medium text-muted-color">Namespace</label>
                <InputText disabled v-model="formValues.namespace" size="small" fluid />
              </div>
            </div>
          </div>

          <!-- Section 2: Labels & Annotations -->
          <div class="grid grid-cols-1 md:grid-cols-12 gap-6 max-w-7xl">
            <div class="md:col-span-4 flex flex-col gap-1">
              <span class="text-xs font-semibold tracking-wider text-muted-color uppercase">
                Metadata & Tags
              </span>
              <p class="text-xs text-muted-color leading-relaxed">
                Key-value pairs for organizational indexing, selectors, and Kubernetes annotations.
              </p>
            </div>
            <div class="md:col-span-8 flex flex-col gap-6">
              <div class="p-4 rounded-lg bg-(--bg-hover)/30 flex flex-col gap-3">
                <KeyValueEditor v-model="formValues.labels" title="Labels" add-label="Add Label" />
              </div>
              <div class="p-4 rounded-lg bg-(--bg-hover)/30 flex flex-col gap-3">
                <KeyValueEditor
                  v-model="formValues.annotations"
                  title="Annotations"
                  add-label="Add Annotation"
                />
              </div>
            </div>
          </div>
        </div>
      </template>

      <!-- YAML Mode -->
      <div v-else class="w-full h-full flex-1 min-h-0 pt-2">
        <vue-monaco-editor
          v-model:value="yamlContent"
          :theme="isDark ? 'vs-dark' : 'vs'"
          language="yaml"
          height="100%"
          width="100%"
          :options="{
            automaticLayout: true,
            minimap: { enabled: true },
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
