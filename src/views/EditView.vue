<script setup lang="ts">
import { kubernetesService } from '@/services/kubernetesService'
import { events } from '@/services/nativeService'
import { OrbitEvents, type OrbitEventMap } from '@/types/events'
import { useTabsStore } from '@/stores/tabsStore'
import Button from 'primevue/button'
import InputText from 'primevue/inputtext'
import ToggleSwitch from 'primevue/toggleswitch'
import { useToast } from 'primevue/usetoast'
import { computed, defineAsyncComponent, onMounted, onUnmounted, ref, watch } from 'vue'
import type { Component } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import * as yaml from 'yaml'

import { ArrowLeft, Loader2 } from '@lucide/vue'
import KeyValueEditor from '@/components/shared/KeyValueEditor.vue'
import YamlCodeEditorLoading from '@/components/shared/YamlCodeEditorLoading.vue'
import ConfigMapEditForm from '@/components/config/ConfigMapEditForm.vue'
import SecretEditForm from '@/components/config/SecretEditForm.vue'
import IngressEditForm from '@/components/network/IngressEditForm.vue'
import ServiceEditForm from '@/components/network/ServiceEditForm.vue'
import DeploymentEditForm from '@/components/workloads/DeploymentEditForm.vue'
import DaemonSetEditForm from '@/components/workloads/DaemonSetEditForm.vue'
import StatefulSetEditForm from '@/components/workloads/StatefulSetEditForm.vue'
import PodEditForm from '@/components/workloads/PodEditForm.vue'
import ReplicaSetEditForm from '@/components/workloads/ReplicaSetEditForm.vue'
import JobEditForm from '@/components/workloads/JobEditForm.vue'
import NamespaceEditForm from '@/components/namespaces/NamespaceEditForm.vue'
import PersistentVolumeEditForm from '@/components/storage/PersistentVolumeEditForm.vue'
import PersistentVolumeClaimEditForm from '@/components/storage/PersistentVolumeClaimEditForm.vue'
import StorageClassEditForm from '@/components/storage/StorageClassEditForm.vue'
import { useTheme } from '@/composables/useTheme'
import { KUBERNETES_RESOURCE_KIND } from '@/constants/kubernetes'
import type { KubernetesResource } from '@/types/kubernetes'
import { RESOURCE_BREADCRUMB_MAP } from '@/utils/breadcrumb'
import {
  applyMetadataFormValuesToResource,
  buildEditRequestId,
  extractResourceMetadataFormValues,
  matchesEditResponse,
  matchesSaveResult,
  transitionEditMode
} from '@/utils/editSession'

const YamlCodeEditor = defineAsyncComponent({
  loader: () => import('@/components/shared/YamlCodeEditor.vue'),
  loadingComponent: YamlCodeEditorLoading,
  delay: 0
})

const props = withDefaults(
  defineProps<{
    kind?: string
    namespace?: string
    name?: string
  }>(),
  {
    kind: '',
    namespace: '',
    name: ''
  }
)

const router = useRouter()
const route = useRoute()
const tabsStore = useTabsStore()
const toast = useToast()
const { isDark } = useTheme()

const resourceKind = computed(() => (props.kind || (route.params.kind as string) || '').trim())
const resourceNamespace = computed(() =>
  (props.namespace || (route.params.namespace as string) || '').trim()
)
const resourceName = computed(() => (props.name || (route.params.name as string) || '').trim())

const rawData = ref<KubernetesResource | null>(null)
const formComponentMap: Record<string, Component> = {
  [KUBERNETES_RESOURCE_KIND.Deployment]: DeploymentEditForm,
  [KUBERNETES_RESOURCE_KIND.DaemonSet]: DaemonSetEditForm,
  [KUBERNETES_RESOURCE_KIND.StatefulSet]: StatefulSetEditForm,
  [KUBERNETES_RESOURCE_KIND.ReplicaSet]: ReplicaSetEditForm,
  [KUBERNETES_RESOURCE_KIND.Job]: JobEditForm,
  [KUBERNETES_RESOURCE_KIND.Pod]: PodEditForm,
  [KUBERNETES_RESOURCE_KIND.Ingress]: IngressEditForm,
  [KUBERNETES_RESOURCE_KIND.Service]: ServiceEditForm,
  [KUBERNETES_RESOURCE_KIND.ConfigMap]: ConfigMapEditForm,
  [KUBERNETES_RESOURCE_KIND.Secret]: SecretEditForm,
  [KUBERNETES_RESOURCE_KIND.Namespace]: NamespaceEditForm,
  [KUBERNETES_RESOURCE_KIND.PersistentVolume]: PersistentVolumeEditForm,
  [KUBERNETES_RESOURCE_KIND.PersistentVolumeClaim]: PersistentVolumeClaimEditForm,
  [KUBERNETES_RESOURCE_KIND.StorageClass]: StorageClassEditForm
}
const activeFormComponent = computed(() => formComponentMap[resourceKind.value] || null)
const isClusterScoped = computed(() => {
  return Boolean(RESOURCE_BREADCRUMB_MAP[resourceKind.value]?.isClusterScoped)
})
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
  return isYamlMode.value ? isYamlValid.value : !activeFormComponent.value || isChildFormValid.value
})

// Simple form fields
const formValues = ref({
  name: '',
  namespace: '',
  labels: [] as { key: string; value: string }[],
  annotations: [] as { key: string; value: string }[]
})

let currentRequestId = ''
let activeSaveRequestId = ''
let eventHandler: (data: {
  name: string
  kind: string
  namespace?: string
  requestId?: string
  data: Record<string, unknown>
}) => void
let applyFinishedHandler: (payload: OrbitEventMap['resourceApplyFinished']) => void

const applyLoadedResource = (resource: KubernetesResource) => {
  rawData.value = resource
  yamlContent.value = yaml.stringify(resource)
  formValues.value = extractResourceMetadataFormValues(resource)
}

const resetState = () => {
  rawData.value = null
  yamlContent.value = ''
  isYamlMode.value = false
  isLoading.value = true
  isSaving.value = false
  isChildFormValid.value = true
  activeSaveRequestId = ''
  formValues.value = {
    name: '',
    namespace: '',
    labels: [],
    annotations: []
  }
}

const loadResource = () => {
  const kind = resourceKind.value
  const namespace = resourceNamespace.value
  const name = resourceName.value
  if (!kind || !name) {
    return
  }
  currentRequestId = buildEditRequestId(kind, namespace, name)
  isLoading.value = true
  kubernetesService.getResourceRaw({
    namespace,
    kind,
    name,
    requestId: currentRequestId
  })
}

watch(
  () => [resourceKind.value, resourceNamespace.value, resourceName.value],
  () => {
    resetState()
    if (resourceKind.value && resourceName.value) {
      loadResource()
    }
  }
)

onMounted(() => {
  eventHandler = (data: {
    name: string
    kind: string
    namespace?: string
    requestId?: string
    data: Record<string, unknown>
  }) => {
    if (
      matchesEditResponse(
        {
          kind: data.kind,
          name: data.name,
          namespace: data.namespace,
          requestId: data.requestId
        },
        {
          kind: resourceKind.value,
          namespace: resourceNamespace.value,
          name: resourceName.value,
          requestId: currentRequestId
        }
      )
    ) {
      applyLoadedResource(data.data as KubernetesResource)
      isLoading.value = false
    }
  }

  applyFinishedHandler = (payload: OrbitEventMap['resourceApplyFinished']) => {
    if (
      !matchesSaveResult(payload, {
        isSaving: isSaving.value,
        kind: resourceKind.value,
        namespace: resourceNamespace.value,
        name: resourceName.value,
        requestId: activeSaveRequestId
      })
    ) {
      return
    }

    isSaving.value = false
    activeSaveRequestId = ''

    if (payload.success && payload.data) {
      applyLoadedResource(payload.data as KubernetesResource)
    }
  }

  events.on(OrbitEvents.ResourceRawData, eventHandler)
  events.on(OrbitEvents.ResourceApplyFinished, applyFinishedHandler)

  // Fetch initial data
  loadResource()
})

onUnmounted(() => {
  events.off(OrbitEvents.ResourceRawData, eventHandler)
  events.off(OrbitEvents.ResourceApplyFinished, applyFinishedHandler)
})

const goBack = () => {
  const tabId = route.fullPath
  if (tabsStore.tabs.some((t) => t.id === tabId || t.route === tabId)) {
    const nextRoute = tabsStore.closeTabAndResolveRoute(tabId)
    if (nextRoute) {
      void router.replace(nextRoute)
      return
    }
  }
  void router.replace('/')
}

const handleModeChange = (targetYamlMode: boolean) => {
  const result = transitionEditMode({
    targetYamlMode,
    rawData: rawData.value,
    yamlContent: yamlContent.value,
    hasCustomForm: Boolean(activeFormComponent.value)
  })
  if (!result.ok) {
    toast.add({
      severity: 'error',
      summary: 'YAML Parse Error',
      detail: `Cannot switch to Form mode: ${result.errorMessage}`,
      life: 5000
    })
    return
  }
  rawData.value = result.rawData
  yamlContent.value = result.yamlContent
  if (result.formValues) {
    formValues.value = result.formValues
  }
  isYamlMode.value = result.isYamlMode
}

const saveChanges = () => {
  const kind = resourceKind.value
  const namespace = resourceNamespace.value
  const name = resourceName.value
  if (!kind || !name || isSaving.value || isLoading.value) return
  isSaving.value = true
  activeSaveRequestId = buildEditRequestId(kind, namespace, name)
  try {
    let updatedData: KubernetesResource
    if (isYamlMode.value) {
      updatedData = yaml.parse(yamlContent.value) as KubernetesResource
    } else {
      updatedData = rawData.value || {}
    }
    kubernetesService.applyResource({
      namespace,
      kind,
      name,
      data: updatedData,
      requestId: activeSaveRequestId
    })
  } catch (err) {
    toast.add({ severity: 'error', summary: 'YAML Parse Error', detail: String(err), life: 5000 })
    isSaving.value = false
    activeSaveRequestId = ''
  }
}

const handleCustomFormUpdate = (updatedData: KubernetesResource) => {
  rawData.value = updatedData
  yamlContent.value = yaml.stringify(updatedData)
}

const handleFallbackMetadataChange = () => {
  if (!rawData.value || activeFormComponent.value) return
  const updated = applyMetadataFormValuesToResource(rawData.value, formValues.value)
  rawData.value = updated
  yamlContent.value = yaml.stringify(updated)
}
</script>

<template>
  <div class="flex flex-col h-[calc(100vh-8rem)]">
    <!-- Control Bar (Tone Shift) -->
    <div
      class="flex items-center justify-between px-4 py-3 rounded-[var(--ui-radius-card)] bg-(--bg-hover)/40 shrink-0 mb-6"
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
            Edit {{ resourceKind }}
          </h1>
          <span class="text-xs font-medium text-muted-color font-mono">
            {{
              !isClusterScoped && resourceNamespace && resourceNamespace !== '-'
                ? `${resourceNamespace} / ${resourceName}`
                : resourceName
            }}
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
          <ToggleSwitch :model-value="isYamlMode" @update:model-value="handleModeChange" />
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
              <div
                class="p-4 rounded-[var(--ui-radius-card)] bg-(--bg-hover)/30 flex flex-col gap-3"
              >
                <KeyValueEditor
                  v-model="formValues.labels"
                  title="Labels"
                  add-label="Add Label"
                  @update:model-value="handleFallbackMetadataChange"
                />
              </div>
              <div
                class="p-4 rounded-[var(--ui-radius-card)] bg-(--bg-hover)/30 flex flex-col gap-3"
              >
                <KeyValueEditor
                  v-model="formValues.annotations"
                  title="Annotations"
                  add-label="Add Annotation"
                  @update:model-value="handleFallbackMetadataChange"
                />
              </div>
            </div>
          </div>
        </div>
      </template>

      <!-- YAML Mode -->
      <div v-else class="w-full h-full flex-1 min-h-0 pt-2">
        <YamlCodeEditor v-model="yamlContent" :is-dark="isDark" :read-only="isSaving" />
      </div>
    </div>
  </div>
</template>
