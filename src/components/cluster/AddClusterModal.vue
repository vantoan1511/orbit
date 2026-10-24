<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import {
  Button,
  Checkbox,
  Dialog,
  InputText,
  Select,
  Tab,
  TabList,
  TabPanel,
  TabPanels,
  Tabs,
  Textarea,
  useToast
} from 'primevue'
import {
  AlertCircle,
  ChevronDown,
  ChevronRight,
  FileText,
  FolderOpen,
  Globe,
  Plus,
  Server,
  Trash2,
  Upload
} from '@lucide/vue'
import ClusterCustomizationFields from '@/components/cluster/ClusterCustomizationFields.vue'
import { kubernetesService } from '@/services/kubernetesService'
import { filesystem, os } from '@/services/nativeService'
import { useKubernetesStore } from '@/stores/kubernetesStore'
import {
  AUTH_TYPE_OPTIONS,
  type AddClusterCustomization,
  type AddClusterPayload,
  type DiscoveredContextInfo,
  type ManualEndpointConfig
} from '@/types/cluster'
import {
  buildKubeconfigFromEndpoint,
  filterKubeconfigContexts,
  parseKubeconfigYaml
} from '@/utils/kubeconfigParser'

const props = defineProps<{
  visible: boolean
}>()

const emit = defineEmits<{
  (e: 'update:visible', val: boolean): void
  (e: 'added'): void
}>()

const toast = useToast()
const k8sStore = useKubernetesStore()
const existingClusterNames = computed(() => new Set(k8sStore.clusters.map((c) => c.name)))

const activeTab = ref<'file' | 'paste' | 'manual'>('file')
const isSubmitting = ref(false)
const switchContext = ref(true)

// Tab 1: File selection
const selectedFilePath = ref('')
const selectedFileSize = ref<string | null>(null)
const fileRawContent = ref('')
const isDraggingOver = ref(false)

// Tab 2: Paste YAML
const pastedYaml = ref('')
const yamlParseError = ref<string | null>(null)

// Tab 3: Manual Endpoint
function createDefaultManualConfig(): ManualEndpointConfig {
  return {
    contextName: '',
    clusterName: '',
    serverUrl: '',
    authType: 'token',
    token: '',
    clientCertData: '',
    clientKeyData: '',
    username: '',
    password: '',
    insecureSkipTlsVerify: false,
    caCertData: ''
  }
}

const manualConfig = ref<ManualEndpointConfig>(createDefaultManualConfig())

// Preview Contexts
const discoveredContexts = ref<DiscoveredContextInfo[]>([])
const selectedContexts = ref<string[]>([])

// Customization Section
const showCustomization = ref(false)
const alias = ref('')
const selectedColor = ref<string | null>(null)
const tags = ref<string[]>([])

const getTabClass = (tab: 'file' | 'paste' | 'manual'): string =>
  activeTab.value === tab
    ? 'border-(--accent)! text-(--text-primary)! font-medium'
    : 'border-transparent! text-muted-color hover:text-(--text-primary) hover:border-(--border)'

// Reset state
function resetModalState(): void {
  activeTab.value = 'file'
  selectedFilePath.value = ''
  selectedFileSize.value = null
  fileRawContent.value = ''
  pastedYaml.value = ''
  yamlParseError.value = null
  manualConfig.value = createDefaultManualConfig()
  discoveredContexts.value = []
  selectedContexts.value = []
  showCustomization.value = false
  alias.value = ''
  selectedColor.value = null
  tags.value = []
  switchContext.value = true
  isSubmitting.value = false
}

watch(
  () => props.visible,
  (isVisible) => {
    if (isVisible) {
      resetModalState()
    }
  }
)

function updateDiscoveredContexts(contexts: DiscoveredContextInfo[]): void {
  discoveredContexts.value = contexts
  // By default, exclude contexts that already exist in Orbit
  selectedContexts.value = contexts
    .filter((c) => !existingClusterNames.value.has(c.name))
    .map((c) => c.name)
}

// File Selection Handler
async function handleBrowseFile(): Promise<void> {
  try {
    const dialogResult = await os.showOpenDialog('Select Kubeconfig File')
    if (dialogResult && dialogResult[0]) {
      const filePath = dialogResult[0]
      await processSelectedFile(filePath)
    }
  } catch (err) {
    console.error('Failed to open file dialog:', err)
    toast.add({
      severity: 'error',
      summary: 'Error',
      detail: 'Failed to select kubeconfig file.',
      life: 3000
    })
  }
}

async function processSelectedFile(filePath: string): Promise<void> {
  try {
    const content = await filesystem.readTextFile(filePath)
    selectedFilePath.value = filePath
    selectedFileSize.value = `${(content.length / 1024).toFixed(1)} KB`
    fileRawContent.value = content

    const contexts = parseKubeconfigYaml(content)
    updateDiscoveredContexts(contexts)

    if (contexts.length === 0) {
      toast.add({
        severity: 'warn',
        summary: 'No Contexts Found',
        detail: 'The selected file does not appear to contain valid Kubernetes contexts.',
        life: 4000
      })
    }
  } catch (err) {
    console.error('Failed to read kubeconfig file:', err)
    toast.add({
      severity: 'error',
      summary: 'File Read Error',
      detail: 'Could not read the selected kubeconfig file.',
      life: 3000
    })
  }
}

// Drag & Drop handlers
function handleDragOver(e: DragEvent): void {
  e.preventDefault()
  isDraggingOver.value = true
}

function handleDragLeave(e: DragEvent): void {
  e.preventDefault()
  isDraggingOver.value = false
}

async function handleDrop(e: DragEvent): Promise<void> {
  e.preventDefault()
  isDraggingOver.value = false

  const files = e.dataTransfer?.files
  if (files && files.length > 0) {
    const file = files[0]
    if (file) {
      // Web File API reading
      const text = await file.text()
      selectedFilePath.value = file.name
      selectedFileSize.value = `${(file.size / 1024).toFixed(1)} KB`
      fileRawContent.value = text

      const contexts = parseKubeconfigYaml(text)
      updateDiscoveredContexts(contexts)
    }
  }
}

// Paste YAML Watcher
watch(pastedYaml, (newYaml) => {
  if (activeTab.value !== 'paste') return
  if (!newYaml.trim()) {
    yamlParseError.value = null
    updateDiscoveredContexts([])
    return
  }

  const contexts = parseKubeconfigYaml(newYaml)
  if (contexts.length > 0) {
    yamlParseError.value = null
    updateDiscoveredContexts(contexts)
  } else {
    yamlParseError.value = 'No valid Kubernetes contexts detected in YAML'
    updateDiscoveredContexts([])
  }
})

// Manual Config Watcher
watch(
  manualConfig,
  (cfg) => {
    if (activeTab.value !== 'manual') return
    if (!cfg.clusterName.trim() || !cfg.serverUrl.trim()) {
      updateDiscoveredContexts([])
      return
    }

    const effectiveContextName = cfg.contextName.trim() || `${cfg.clusterName.trim()}-context`
    const generatedYaml = buildKubeconfigFromEndpoint({
      ...cfg,
      contextName: effectiveContextName
    })
    const contexts = parseKubeconfigYaml(generatedYaml)
    updateDiscoveredContexts(contexts)
  },
  { deep: true }
)

// Active tab switch handling
watch(activeTab, (tab) => {
  if (tab === 'file') {
    updateDiscoveredContexts(fileRawContent.value ? parseKubeconfigYaml(fileRawContent.value) : [])
  } else if (tab === 'paste') {
    updateDiscoveredContexts(pastedYaml.value ? parseKubeconfigYaml(pastedYaml.value) : [])
  } else if (tab === 'manual') {
    const { clusterName, serverUrl, contextName } = manualConfig.value
    if (clusterName.trim() && serverUrl.trim()) {
      const generated = buildKubeconfigFromEndpoint({
        ...manualConfig.value,
        contextName: contextName.trim() || `${clusterName.trim()}-context`
      })
      updateDiscoveredContexts(parseKubeconfigYaml(generated))
    } else {
      updateDiscoveredContexts([])
    }
  }
})

// Context selection controls
const isAllSelected = computed(() => {
  return (
    discoveredContexts.value.length > 0 &&
    selectedContexts.value.length === discoveredContexts.value.length
  )
})

function toggleSelectAll(): void {
  if (isAllSelected.value) {
    selectedContexts.value = []
  } else {
    selectedContexts.value = discoveredContexts.value.map((c) => c.name)
  }
}

function toggleContextSelection(name: string): void {
  if (selectedContexts.value.includes(name)) {
    selectedContexts.value = selectedContexts.value.filter((n) => n !== name)
  } else {
    selectedContexts.value.push(name)
  }
}

// Submission
const canSubmit = computed(() => {
  return selectedContexts.value.length > 0 && !isSubmitting.value
})

async function handleAddClusterSubmit(): Promise<void> {
  if (!canSubmit.value) return

  isSubmitting.value = true
  try {
    let payloadContent = ''

    if (activeTab.value === 'file') {
      payloadContent = fileRawContent.value
    } else if (activeTab.value === 'paste') {
      payloadContent = pastedYaml.value
    } else if (activeTab.value === 'manual') {
      const { clusterName, contextName } = manualConfig.value
      payloadContent = buildKubeconfigFromEndpoint({
        ...manualConfig.value,
        contextName: contextName.trim() || `${clusterName.trim()}-context`
      })
    }

    // Filter content to only include the checked contexts
    const filteredContent = filterKubeconfigContexts(payloadContent, selectedContexts.value)

    const metadata: AddClusterCustomization = {}
    if (alias.value.trim()) metadata.alias = alias.value.trim()
    if (selectedColor.value) metadata.color = selectedColor.value
    if (tags.value.length > 0) metadata.tags = [...tags.value]

    const payload: AddClusterPayload = {
      kubeconfigContent: filteredContent,
      selectedContexts: [...selectedContexts.value],
      metadata: Object.keys(metadata).length > 0 ? metadata : undefined,
      switchContext: switchContext.value
    }

    await kubernetesService.addCluster(payload)

    toast.add({
      severity: 'success',
      summary: 'Cluster Added',
      detail: `Successfully imported ${selectedContexts.value.length} context(s).`,
      life: 3000
    })

    emit('added')
    emit('update:visible', false)
  } catch (err) {
    console.error('Failed to add cluster:', err)
    toast.add({
      severity: 'error',
      summary: 'Import Failed',
      detail: 'Failed to add the cluster. Check the kubeconfig configuration.',
      life: 4000
    })
  } finally {
    isSubmitting.value = false
  }
}

function handleClose(): void {
  emit('update:visible', false)
}
</script>

<template>
  <Dialog
    :visible="visible"
    modal
    dismissable-mask
    :closable="!isSubmitting"
    data-component="add-cluster-modal"
    class="w-full max-w-2xl border border-(--border) bg-(--bg-card) text-(--text-primary) shadow-[var(--ui-card-shadow,none)] rounded-[var(--ui-radius-card)] overflow-hidden"
    :pt="{
      root: { class: 'p-0 overflow-hidden' },
      mask: { class: 'bg-black/60 backdrop-blur-xs' },
      header: {
        class:
          'border-b border-(--border) px-5 py-3.5 bg-(--bg-sidebar) flex items-center justify-between'
      },
      content: { class: 'p-5 max-h-[75vh] overflow-y-auto flex flex-col gap-6 bg-(--bg-card)' },
      footer: {
        class:
          'border-t border-(--border) px-5 py-3 bg-(--bg-sidebar) flex items-center justify-between'
      },
      closeButton: {
        class:
          'w-7 h-7 p-0 text-muted-color hover:text-(--text-primary) hover:bg-(--bg-hover) rounded-[var(--ui-radius-control)] focus:outline-none focus-visible:ring-1 focus-visible:ring-primary'
      }
    }"
    :close-button-props="{
      severity: 'secondary',
      variant: 'text',
      rounded: true,
      class:
        'w-7 h-7 p-0 text-muted-color hover:text-(--text-primary) hover:bg-(--bg-hover) rounded-[var(--ui-radius-control)] focus:outline-none focus-visible:ring-1 focus-visible:ring-primary'
    }"
    @update:visible="emit('update:visible', $event)"
  >
    <template #header>
      <div class="flex items-center gap-3">
        <div
          class="w-8 h-8 rounded-[var(--ui-radius-control)] bg-(--accent-soft) text-(--accent) flex items-center justify-center"
        >
          <Server class="w-4 h-4" />
        </div>
        <div>
          <h3 class="text-base font-semibold text-(--text-primary)">Add Kubernetes Cluster</h3>
          <p class="text-xs text-muted-color">Connect to a local or remote Kubernetes cluster</p>
        </div>
      </div>
    </template>

    <div class="flex flex-col gap-6">
      <!-- Input Mode Tabs -->
      <Tabs v-model:value="activeTab" class="w-full">
        <TabList class="bg-transparent! border-b! border-(--border)! px-1 flex gap-2 shrink-0">
          <Tab
            value="file"
            class="bg-transparent! py-2 px-3 text-xs cursor-pointer flex items-center gap-2 border-b-2 transition-all rounded-none!"
            :class="getTabClass('file')"
          >
            <FolderOpen class="w-3.5 h-3.5" />
            <span>Kubeconfig File</span>
          </Tab>
          <Tab
            value="paste"
            class="bg-transparent! py-2 px-3 text-xs cursor-pointer flex items-center gap-2 border-b-2 transition-all rounded-none!"
            :class="getTabClass('paste')"
          >
            <FileText class="w-3.5 h-3.5" />
            <span>Paste YAML</span>
          </Tab>
          <Tab
            value="manual"
            class="bg-transparent! py-2 px-3 text-xs cursor-pointer flex items-center gap-2 border-b-2 transition-all rounded-none!"
            :class="getTabClass('manual')"
          >
            <Globe class="w-3.5 h-3.5" />
            <span>Manual Endpoint</span>
          </Tab>
        </TabList>

        <TabPanels class="pt-4 p-0">
          <!-- Tab 1: Kubeconfig File -->
          <TabPanel value="file">
            <div
              class="border-2 border-dashed rounded-[var(--ui-radius-card)] p-6 flex flex-col items-center justify-center gap-3 transition-colors text-center cursor-pointer"
              :class="[
                isDraggingOver
                  ? 'border-(--accent) bg-(--accent-soft)'
                  : 'border-(--border) hover:border-(--border-strong) bg-(--bg-hover)/20'
              ]"
              @dragover="handleDragOver"
              @dragleave="handleDragLeave"
              @drop="handleDrop"
              @click="handleBrowseFile"
            >
              <div
                class="w-10 h-10 rounded-[var(--ui-radius-control)] bg-(--bg-hover) flex items-center justify-center text-muted-color"
              >
                <Upload class="w-5 h-5" />
              </div>
              <div class="flex flex-col gap-1">
                <p class="text-xs font-medium text-(--text-primary)">
                  Drag and drop your kubeconfig file here, or click to browse
                </p>
                <p class="text-[11px] text-muted-color">
                  Supports standard kubeconfig YAML files (.yaml, .yml, config)
                </p>
              </div>
              <Button
                size="small"
                severity="secondary"
                class="flex items-center gap-1.5 text-xs mt-1 rounded-[var(--ui-radius-control)]"
                @click.stop="handleBrowseFile"
              >
                <FolderOpen class="w-3.5 h-3.5" />
                <span>Browse File...</span>
              </Button>

              <div
                v-if="selectedFilePath"
                class="mt-2 px-3 py-1.5 bg-(--bg-card) border border-(--border) rounded-[var(--ui-radius-control)] text-xs text-(--text-primary) flex items-center gap-2 max-w-full truncate"
              >
                <span class="font-medium truncate">{{ selectedFilePath }}</span>
                <span v-if="selectedFileSize" class="text-muted-color text-[11px]"
                  >({{ selectedFileSize }})</span
                >
              </div>
            </div>
          </TabPanel>

          <!-- Tab 2: Paste YAML -->
          <TabPanel value="paste">
            <div class="flex flex-col gap-2">
              <div class="flex items-center justify-between text-xs text-muted-color">
                <span>Paste your Kubeconfig YAML text below:</span>
                <Button
                  v-if="pastedYaml"
                  v-tooltip.top="'Clear YAML'"
                  size="small"
                  variant="text"
                  severity="secondary"
                  class="p-1 h-6 text-xs flex items-center gap-1 text-muted-color hover:text-(--text-primary) rounded-[var(--ui-radius-control)]"
                  @click="pastedYaml = ''"
                >
                  <Trash2 class="w-3 h-3" />
                  <span>Clear</span>
                </Button>
              </div>
              <Textarea
                v-model="pastedYaml"
                rows="8"
                placeholder="apiVersion: v1&#10;kind: Config&#10;clusters: ...&#10;contexts: ..."
                class="w-full font-mono text-xs leading-relaxed border-(--border) rounded-[var(--ui-radius-control)] p-3 bg-(--bg-app)/60"
              />
              <div v-if="yamlParseError" class="flex items-center gap-1.5 text-xs text-(--danger)">
                <AlertCircle class="w-3.5 h-3.5 shrink-0" />
                <span>{{ yamlParseError }}</span>
              </div>
            </div>
          </TabPanel>

          <!-- Tab 3: Manual Endpoint -->
          <TabPanel value="manual">
            <div class="flex flex-col gap-4 text-xs">
              <div class="grid grid-cols-2 gap-3">
                <div class="flex flex-col gap-1.5">
                  <label class="text-xs font-medium text-muted-color"
                    >Cluster Name <span class="text-(--danger)">*</span></label
                  >
                  <InputText
                    v-model="manualConfig.clusterName"
                    placeholder="e.g. k8s-production"
                    size="small"
                    class="w-full text-xs rounded-[var(--ui-radius-control)]"
                  />
                </div>
                <div class="flex flex-col gap-1.5">
                  <label class="text-xs font-medium text-muted-color">Context Name</label>
                  <InputText
                    v-model="manualConfig.contextName"
                    :placeholder="
                      manualConfig.clusterName
                        ? `${manualConfig.clusterName}-context`
                        : 'e.g. prod-admin'
                    "
                    size="small"
                    class="w-full text-xs rounded-[var(--ui-radius-control)]"
                  />
                </div>
              </div>

              <div class="flex flex-col gap-1.5">
                <label class="text-xs font-medium text-muted-color"
                  >API Server URL <span class="text-(--danger)">*</span></label
                >
                <InputText
                  v-model="manualConfig.serverUrl"
                  placeholder="https://192.168.1.100:6443 or https://k8s.example.com:6443"
                  size="small"
                  class="w-full text-xs font-mono rounded-[var(--ui-radius-control)]"
                />
              </div>

              <div class="flex flex-col gap-1.5">
                <label class="text-xs font-medium text-muted-color">Authentication Method</label>
                <Select
                  v-model="manualConfig.authType"
                  :options="AUTH_TYPE_OPTIONS"
                  option-label="label"
                  option-value="value"
                  size="small"
                  class="w-full text-xs rounded-[var(--ui-radius-control)]"
                />
              </div>

              <!-- Auth Type Specific Inputs -->
              <div v-if="manualConfig.authType === 'token'" class="flex flex-col gap-1.5">
                <label class="text-xs font-medium text-muted-color"
                  >Bearer Token <span class="text-(--danger)">*</span></label
                >
                <Textarea
                  v-model="manualConfig.token"
                  rows="3"
                  placeholder="Paste service account bearer token here"
                  class="w-full font-mono text-xs border-(--border) rounded-[var(--ui-radius-control)] p-2.5 bg-(--bg-app)/60"
                />
              </div>

              <div v-else-if="manualConfig.authType === 'certificate'" class="flex flex-col gap-3">
                <div class="flex flex-col gap-1.5">
                  <label class="text-xs font-medium text-muted-color"
                    >Client Certificate Data (Base64) <span class="text-(--danger)">*</span></label
                  >
                  <Textarea
                    v-model="manualConfig.clientCertData"
                    rows="2"
                    placeholder="LS0tLS1CRUdJTiBDRVJUSUZJQ0FURS0tLS0t..."
                    class="w-full font-mono text-xs border-(--border) rounded-[var(--ui-radius-control)] p-2.5 bg-(--bg-app)/60"
                  />
                </div>
                <div class="flex flex-col gap-1.5">
                  <label class="text-xs font-medium text-muted-color"
                    >Client Key Data (Base64) <span class="text-(--danger)">*</span></label
                  >
                  <Textarea
                    v-model="manualConfig.clientKeyData"
                    rows="2"
                    placeholder="LS0tLS1CRUdJTiBSU0EgUFJJVkFURSBLRVktLS0tLQ..."
                    class="w-full font-mono text-xs border-(--border) rounded-[var(--ui-radius-control)] p-2.5 bg-(--bg-app)/60"
                  />
                </div>
              </div>

              <div v-else-if="manualConfig.authType === 'basic'" class="grid grid-cols-2 gap-3">
                <div class="flex flex-col gap-1.5">
                  <label class="text-xs font-medium text-muted-color">Username</label>
                  <InputText
                    v-model="manualConfig.username"
                    placeholder="admin"
                    size="small"
                    class="w-full text-xs rounded-[var(--ui-radius-control)]"
                  />
                </div>
                <div class="flex flex-col gap-1.5">
                  <label class="text-xs font-medium text-muted-color">Password</label>
                  <InputText
                    v-model="manualConfig.password"
                    type="password"
                    size="small"
                    class="w-full text-xs rounded-[var(--ui-radius-control)]"
                  />
                </div>
              </div>

              <div class="flex flex-col gap-1.5">
                <label class="text-xs font-medium text-muted-color"
                  >Certificate Authority Data (Optional Base64)</label
                >
                <Textarea
                  v-model="manualConfig.caCertData"
                  rows="2"
                  placeholder="Leave empty if using Skip TLS verification or system trust root"
                  class="w-full font-mono text-xs border-(--border) rounded-[var(--ui-radius-control)] p-2.5 bg-(--bg-app)/60"
                />
              </div>

              <div class="flex items-center gap-2 pt-1">
                <Checkbox v-model="manualConfig.insecureSkipTlsVerify" binary input-id="skip-tls" />
                <label for="skip-tls" class="text-xs text-muted-color cursor-pointer select-none">
                  Insecure Skip TLS Verification (Self-signed or internal CA)
                </label>
              </div>
            </div>
          </TabPanel>
        </TabPanels>
      </Tabs>

      <!-- Discovered Contexts Preview -->
      <div class="flex flex-col gap-3">
        <div class="flex items-center justify-between">
          <div class="flex items-center gap-2">
            <span class="text-xs font-semibold tracking-wider text-muted-color uppercase">
              Discovered Contexts
            </span>
            <span
              v-if="discoveredContexts.length > 0"
              class="px-1.5 py-0.5 text-[10px] font-semibold rounded-[var(--ui-radius-control)] bg-(--bg-hover) text-muted-color"
            >
              {{ discoveredContexts.length }} found
            </span>
          </div>
          <Button
            v-if="discoveredContexts.length > 0"
            size="small"
            variant="text"
            severity="secondary"
            class="text-xs p-0 h-auto text-(--accent) hover:underline rounded-[var(--ui-radius-control)]"
            @click="toggleSelectAll"
          >
            {{ isAllSelected ? 'Deselect All' : 'Select All' }}
          </Button>
        </div>

        <div
          v-if="discoveredContexts.length === 0"
          class="border border-(--border) rounded-[var(--ui-radius-card)] p-6 bg-(--bg-hover)/40 text-center flex flex-col items-center justify-center gap-1.5 text-muted-color"
        >
          <Server class="w-5 h-5 opacity-40" />
          <p class="text-xs">No contexts discovered yet</p>
          <p class="text-[11px] opacity-75">
            Select a kubeconfig file, paste YAML, or enter endpoint details above
          </p>
        </div>

        <div
          v-else
          class="border border-(--border) rounded-[var(--ui-radius-card)] divide-y divide-(--border) overflow-hidden bg-(--bg-card)"
        >
          <div
            v-for="ctx in discoveredContexts"
            :key="ctx.name"
            class="px-3.5 py-2.5 flex items-center justify-between gap-3 hover:bg-(--bg-hover)/40 transition-colors cursor-pointer"
            @click="toggleContextSelection(ctx.name)"
          >
            <div class="flex items-center gap-3 min-w-0">
              <Checkbox
                :model-value="selectedContexts.includes(ctx.name)"
                binary
                @click.stop="toggleContextSelection(ctx.name)"
              />
              <div class="flex flex-col min-w-0">
                <div class="flex items-center gap-2">
                  <span class="text-xs font-medium text-(--text-primary) truncate">{{
                    ctx.name
                  }}</span>
                  <span
                    v-if="ctx.isCurrentContext"
                    class="px-1.5 py-0.2 text-[10px] font-medium rounded-[var(--ui-radius-control)] bg-(--accent-soft) text-(--accent)"
                  >
                    Current
                  </span>
                  <span
                    v-if="existingClusterNames.has(ctx.name)"
                    class="px-1.5 py-0.5 text-[10px] font-medium rounded-[var(--ui-radius-control)] bg-[var(--warning-soft)] text-[var(--warning)] border border-[var(--warning)]/20"
                  >
                    Already in Orbit
                  </span>
                </div>
                <div class="flex items-center gap-2 text-[11px] text-muted-color truncate">
                  <span class="truncate">{{ ctx.serverUrl || 'No server URL' }}</span>
                  <span>•</span>
                  <span>Cluster: {{ ctx.clusterName }}</span>
                </div>
              </div>
            </div>

            <div class="shrink-0">
              <span
                class="px-2 py-0.5 text-[10px] font-semibold rounded-[var(--ui-radius-control)] bg-(--bg-hover) text-muted-color border border-(--border)"
              >
                {{ ctx.authType }}
              </span>
            </div>
          </div>
        </div>
      </div>

      <!-- Cluster Customization Section -->
      <div
        class="border border-(--border) rounded-[var(--ui-radius-card)] overflow-hidden bg-(--bg-card)"
      >
        <Button
          variant="text"
          severity="secondary"
          class="w-full! px-4! py-3! flex! items-center! justify-between! text-left! text-xs! font-medium! text-(--text-primary)! hover:bg-(--bg-hover)/40! transition-colors! cursor-pointer! select-none! rounded-none! border-none!"
          @click="showCustomization = !showCustomization"
        >
          <div class="flex items-center gap-2">
            <component
              :is="showCustomization ? ChevronDown : ChevronRight"
              class="w-3.5 h-3.5 text-muted-color"
            />
            <span>Cluster Customization (Optional)</span>
          </div>
          <span class="text-[11px] text-muted-color font-normal"
            >Display name, environment color & tags</span
          >
        </Button>

        <div v-if="showCustomization" class="p-4 pt-1 border-t border-(--border)">
          <ClusterCustomizationFields
            v-model:alias="alias"
            v-model:color="selectedColor"
            v-model:tags="tags"
          />
        </div>
      </div>
    </div>

    <!-- Footer Actions -->
    <template #footer>
      <div class="flex items-center gap-2">
        <Checkbox v-model="switchContext" binary input-id="switch-on-add" />
        <label for="switch-on-add" class="text-xs text-muted-color cursor-pointer select-none">
          Switch to this cluster upon import
        </label>
      </div>

      <div class="flex items-center gap-2">
        <Button
          label="Cancel"
          severity="secondary"
          variant="text"
          size="small"
          class="rounded-[var(--ui-radius-control)]"
          :disabled="isSubmitting"
          @click="handleClose"
        />
        <Button
          severity="primary"
          size="small"
          :loading="isSubmitting"
          :disabled="!canSubmit"
          class="flex items-center gap-1.5 px-4 rounded-[var(--ui-radius-control)]"
          @click="handleAddClusterSubmit"
        >
          <Plus class="w-3.5 h-3.5" />
          <span>Add Cluster</span>
        </Button>
      </div>
    </template>
  </Dialog>
</template>
