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
  Textarea
} from 'primevue'
import { useToast } from 'primevue'
import {
  AlertCircle,
  Check,
  ChevronDown,
  ChevronRight,
  FileText,
  FolderOpen,
  Globe,
  Plus,
  Server,
  Tag as TagIcon,
  Trash2,
  Upload,
  X
} from '@lucide/vue'
import { kubernetesService } from '@/services/kubernetesService'
import { filesystem, os } from '@/services/nativeService'
import type {
  AddClusterCustomization,
  AddClusterPayload,
  DiscoveredContextInfo,
  ManualEndpointConfig
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
const manualConfig = ref<ManualEndpointConfig>({
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
})

const AUTH_TYPE_OPTIONS: Array<{
  label: string
  value: 'token' | 'certificate' | 'basic' | 'none'
}> = [
  { label: 'Bearer Token', value: 'token' },
  { label: 'Client Certificate & Key', value: 'certificate' },
  { label: 'Basic Authentication', value: 'basic' },
  { label: 'None / Anonymous', value: 'none' }
]

// Preview Contexts
const discoveredContexts = ref<DiscoveredContextInfo[]>([])
const selectedContexts = ref<string[]>([])

// Customization Section
const showCustomization = ref(false)
const alias = ref('')
const selectedColor = ref<string | null>(null)
const tags = ref<string[]>([])
const newTagInput = ref('')
const description = ref('')

const COLOR_PRESETS = [
  { label: 'Red (Production)', hex: '#ef4444' },
  { label: 'Orange', hex: '#f97316' },
  { label: 'Amber (Staging)', hex: '#f59e0b' },
  { label: 'Emerald (Dev)', hex: '#10b981' },
  { label: 'Cyan (Test)', hex: '#06b6d4' },
  { label: 'Blue', hex: '#3b82f6' },
  { label: 'Purple', hex: '#8b5cf6' },
  { label: 'Slate (Neutral)', hex: '#64748b' }
] as const

// Reset state
function resetModalState(): void {
  activeTab.value = 'file'
  selectedFilePath.value = ''
  selectedFileSize.value = null
  fileRawContent.value = ''
  pastedYaml.value = ''
  yamlParseError.value = null
  manualConfig.value = {
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
  discoveredContexts.value = []
  selectedContexts.value = []
  showCustomization.value = false
  alias.value = ''
  selectedColor.value = null
  tags.value = []
  newTagInput.value = ''
  description.value = ''
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
  // By default, select all discovered contexts
  selectedContexts.value = contexts.map((c) => c.name)
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
    if (fileRawContent.value) {
      updateDiscoveredContexts(parseKubeconfigYaml(fileRawContent.value))
    } else {
      updateDiscoveredContexts([])
    }
  } else if (tab === 'paste') {
    if (pastedYaml.value) {
      updateDiscoveredContexts(parseKubeconfigYaml(pastedYaml.value))
    } else {
      updateDiscoveredContexts([])
    }
  } else if (tab === 'manual') {
    if (manualConfig.value.clusterName.trim() && manualConfig.value.serverUrl.trim()) {
      const generated = buildKubeconfigFromEndpoint({
        ...manualConfig.value,
        contextName:
          manualConfig.value.contextName.trim() ||
          `${manualConfig.value.clusterName.trim()}-context`
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

// Tag Management
function addTag(): void {
  const raw = newTagInput.value.trim().toLowerCase()
  if (!raw) return
  const tokens = raw
    .split(',')
    .map((t) => t.trim().replace(/^#+/, ''))
    .filter(Boolean)
  for (const token of tokens) {
    if (!tags.value.includes(token)) {
      tags.value.push(token)
    }
  }
  newTagInput.value = ''
}

function handleTagKeydown(e: KeyboardEvent): void {
  if (e.key === 'Enter' || e.key === ',') {
    e.preventDefault()
    addTag()
  }
}

function removeTag(tagToRemove: string): void {
  tags.value = tags.value.filter((t) => t !== tagToRemove)
}

function selectColor(hex: string): void {
  if (selectedColor.value === hex) {
    selectedColor.value = null
  } else {
    selectedColor.value = hex
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
      payloadContent = buildKubeconfigFromEndpoint({
        ...manualConfig.value,
        contextName:
          manualConfig.value.contextName.trim() ||
          `${manualConfig.value.clusterName.trim()}-context`
      })
    }

    // Filter content to only include the checked contexts
    const filteredContent = filterKubeconfigContexts(payloadContent, selectedContexts.value)

    const metadata: AddClusterCustomization = {}
    if (alias.value.trim()) metadata.alias = alias.value.trim()
    if (selectedColor.value) metadata.color = selectedColor.value
    if (tags.value.length > 0) metadata.tags = [...tags.value]
    if (description.value.trim()) metadata.description = description.value.trim()

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
    class="w-full max-w-2xl bg-(--bg-card) border border-(--border) shadow-lg rounded-xl overflow-hidden"
    :pt="{
      root: { class: 'p-0 overflow-hidden' },
      header: {
        class: 'px-6 pt-5 pb-3 border-b border-(--border) flex items-center justify-between'
      },
      content: { class: 'p-6 max-h-[75vh] overflow-y-auto flex flex-col gap-6' },
      footer: {
        class:
          'px-6 py-4 border-t border-(--border) bg-(--bg-app)/50 flex items-center justify-between'
      }
    }"
    @update:visible="emit('update:visible', $event)"
  >
    <template #header>
      <div class="flex items-center gap-3">
        <div
          class="w-8 h-8 rounded-lg bg-(--accent-soft) text-(--accent) flex items-center justify-center"
        >
          <Server class="w-4 h-4" />
        </div>
        <div>
          <h3 class="text-base font-semibold text-primary">Add Kubernetes Cluster</h3>
          <p class="text-xs text-muted-color">Connect to a local or remote Kubernetes cluster</p>
        </div>
      </div>
    </template>

    <div class="flex flex-col gap-6">
      <!-- Input Mode Tabs -->
      <Tabs v-model:value="activeTab" class="w-full">
        <TabList class="bg-(--bg-hover)/30 border border-(--border) rounded-lg p-1 flex gap-1">
          <Tab
            value="file"
            class="flex-1 py-1.5 text-xs font-medium cursor-pointer rounded-md flex items-center justify-center gap-2"
          >
            <FolderOpen class="w-3.5 h-3.5" />
            <span>Kubeconfig File</span>
          </Tab>
          <Tab
            value="paste"
            class="flex-1 py-1.5 text-xs font-medium cursor-pointer rounded-md flex items-center justify-center gap-2"
          >
            <FileText class="w-3.5 h-3.5" />
            <span>Paste YAML</span>
          </Tab>
          <Tab
            value="manual"
            class="flex-1 py-1.5 text-xs font-medium cursor-pointer rounded-md flex items-center justify-center gap-2"
          >
            <Globe class="w-3.5 h-3.5" />
            <span>Manual Endpoint</span>
          </Tab>
        </TabList>

        <TabPanels class="pt-4 p-0">
          <!-- Tab 1: Kubeconfig File -->
          <TabPanel value="file">
            <div
              class="border-2 border-dashed rounded-lg p-6 flex flex-col items-center justify-center gap-3 transition-colors text-center cursor-pointer"
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
                class="w-10 h-10 rounded-full bg-(--bg-hover) flex items-center justify-center text-muted-color"
              >
                <Upload class="w-5 h-5" />
              </div>
              <div class="flex flex-col gap-1">
                <p class="text-xs font-medium text-primary">
                  Drag and drop your kubeconfig file here, or click to browse
                </p>
                <p class="text-[11px] text-muted-color">
                  Supports standard kubeconfig YAML files (.yaml, .yml, config)
                </p>
              </div>
              <Button
                size="small"
                severity="secondary"
                class="flex items-center gap-1.5 text-xs mt-1"
                @click.stop="handleBrowseFile"
              >
                <FolderOpen class="w-3.5 h-3.5" />
                <span>Browse File...</span>
              </Button>

              <div
                v-if="selectedFilePath"
                class="mt-2 px-3 py-1.5 bg-(--bg-card) border border-(--border) rounded text-xs text-primary flex items-center gap-2 max-w-full truncate"
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
                  class="p-1 h-6 text-xs flex items-center gap-1 text-muted-color hover:text-primary"
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
                class="w-full font-mono text-xs leading-relaxed border-(--border) rounded-lg p-3 bg-(--bg-app)"
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
                  <label class="font-medium text-primary"
                    >Cluster Name <span class="text-(--danger)">*</span></label
                  >
                  <InputText
                    v-model="manualConfig.clusterName"
                    placeholder="e.g. k8s-production"
                    size="small"
                    class="w-full text-xs"
                  />
                </div>
                <div class="flex flex-col gap-1.5">
                  <label class="font-medium text-primary">Context Name</label>
                  <InputText
                    v-model="manualConfig.contextName"
                    :placeholder="
                      manualConfig.clusterName
                        ? `${manualConfig.clusterName}-context`
                        : 'e.g. prod-admin'
                    "
                    size="small"
                    class="w-full text-xs"
                  />
                </div>
              </div>

              <div class="flex flex-col gap-1.5">
                <label class="font-medium text-primary"
                  >API Server URL <span class="text-(--danger)">*</span></label
                >
                <InputText
                  v-model="manualConfig.serverUrl"
                  placeholder="https://192.168.1.100:6443 or https://k8s.example.com:6443"
                  size="small"
                  class="w-full text-xs font-mono"
                />
              </div>

              <div class="flex flex-col gap-1.5">
                <label class="font-medium text-primary">Authentication Method</label>
                <Select
                  v-model="manualConfig.authType"
                  :options="AUTH_TYPE_OPTIONS"
                  option-label="label"
                  option-value="value"
                  size="small"
                  class="w-full text-xs"
                />
              </div>

              <!-- Auth Type Specific Inputs -->
              <div v-if="manualConfig.authType === 'token'" class="flex flex-col gap-1.5">
                <label class="font-medium text-primary"
                  >Bearer Token <span class="text-(--danger)">*</span></label
                >
                <Textarea
                  v-model="manualConfig.token"
                  rows="3"
                  placeholder="Paste service account bearer token here"
                  class="w-full font-mono text-xs border-(--border) rounded-lg p-2.5 bg-(--bg-app)"
                />
              </div>

              <div v-else-if="manualConfig.authType === 'certificate'" class="flex flex-col gap-3">
                <div class="flex flex-col gap-1.5">
                  <label class="font-medium text-primary"
                    >Client Certificate Data (Base64) <span class="text-(--danger)">*</span></label
                  >
                  <Textarea
                    v-model="manualConfig.clientCertData"
                    rows="2"
                    placeholder="LS0tLS1CRUdJTiBDRVJUSUZJQ0FURS0tLS0t..."
                    class="w-full font-mono text-xs border-(--border) rounded-lg p-2.5 bg-(--bg-app)"
                  />
                </div>
                <div class="flex flex-col gap-1.5">
                  <label class="font-medium text-primary"
                    >Client Key Data (Base64) <span class="text-(--danger)">*</span></label
                  >
                  <Textarea
                    v-model="manualConfig.clientKeyData"
                    rows="2"
                    placeholder="LS0tLS1CRUdJTiBSU0EgUFJJVkFURSBLRVktLS0tLQ..."
                    class="w-full font-mono text-xs border-(--border) rounded-lg p-2.5 bg-(--bg-app)"
                  />
                </div>
              </div>

              <div v-else-if="manualConfig.authType === 'basic'" class="grid grid-cols-2 gap-3">
                <div class="flex flex-col gap-1.5">
                  <label class="font-medium text-primary">Username</label>
                  <InputText
                    v-model="manualConfig.username"
                    placeholder="admin"
                    size="small"
                    class="w-full text-xs"
                  />
                </div>
                <div class="flex flex-col gap-1.5">
                  <label class="font-medium text-primary">Password</label>
                  <InputText
                    v-model="manualConfig.password"
                    type="password"
                    size="small"
                    class="w-full text-xs"
                  />
                </div>
              </div>

              <div class="flex flex-col gap-1.5">
                <label class="font-medium text-primary"
                  >Certificate Authority Data (Optional Base64)</label
                >
                <Textarea
                  v-model="manualConfig.caCertData"
                  rows="2"
                  placeholder="Leave empty if using Skip TLS verification or system trust root"
                  class="w-full font-mono text-xs border-(--border) rounded-lg p-2.5 bg-(--bg-app)"
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
            <span class="text-xs font-semibold text-primary uppercase tracking-wider">
              Discovered Contexts
            </span>
            <span
              v-if="discoveredContexts.length > 0"
              class="px-1.5 py-0.5 text-[10px] font-semibold rounded bg-(--bg-hover) text-muted-color"
            >
              {{ discoveredContexts.length }} found
            </span>
          </div>
          <Button
            v-if="discoveredContexts.length > 0"
            size="small"
            variant="text"
            severity="secondary"
            class="text-xs p-0 h-auto text-(--accent) hover:underline"
            @click="toggleSelectAll"
          >
            {{ isAllSelected ? 'Deselect All' : 'Select All' }}
          </Button>
        </div>

        <div
          v-if="discoveredContexts.length === 0"
          class="border border-(--border) rounded-lg p-6 bg-(--bg-hover)/20 text-center flex flex-col items-center justify-center gap-1.5 text-muted-color"
        >
          <Server class="w-5 h-5 opacity-40" />
          <p class="text-xs">No contexts discovered yet</p>
          <p class="text-[11px] opacity-75">
            Select a kubeconfig file, paste YAML, or enter endpoint details above
          </p>
        </div>

        <div
          v-else
          class="border border-(--border) rounded-lg divide-y divide-(--border) overflow-hidden bg-(--bg-card)"
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
                  <span class="text-xs font-medium text-primary truncate">{{ ctx.name }}</span>
                  <span
                    v-if="ctx.isCurrentContext"
                    class="px-1.5 py-0.2 text-[10px] font-medium rounded bg-(--accent-soft) text-(--accent)"
                  >
                    Current
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
                class="px-2 py-0.5 text-[10px] font-semibold rounded bg-(--bg-hover) text-muted-color border border-(--border)"
              >
                {{ ctx.authType }}
              </span>
            </div>
          </div>
        </div>
      </div>

      <!-- Cluster Customization Section -->
      <div class="border border-(--border) rounded-lg overflow-hidden bg-(--bg-card)">
        <button
          type="button"
          class="w-full px-4 py-3 flex items-center justify-between text-left text-xs font-medium text-primary hover:bg-(--bg-hover)/40 transition-colors cursor-pointer select-none"
          @click="showCustomization = !showCustomization"
        >
          <div class="flex items-center gap-2">
            <component
              :is="showCustomization ? ChevronDown : ChevronRight"
              class="w-3.5 h-3.5 text-muted-color"
            />
            <span>Cluster Customization (Optional)</span>
          </div>
          <span class="text-[11px] text-muted-color">Display name, environment color & tags</span>
        </button>

        <div
          v-if="showCustomization"
          class="p-4 pt-1 border-t border-(--border) flex flex-col gap-4 text-xs"
        >
          <!-- Friendly Alias -->
          <div class="flex flex-col gap-1.5">
            <label class="font-medium text-primary">Display Name / Alias</label>
            <InputText
              v-model="alias"
              placeholder="e.g. Staging US-West"
              size="small"
              class="w-full text-xs"
            />
          </div>

          <!-- Color Preset Swatches -->
          <div class="flex flex-col gap-1.5">
            <label class="font-medium text-primary">Environment Color</label>
            <div class="flex items-center gap-2 flex-wrap">
              <button
                v-for="preset in COLOR_PRESETS"
                :key="preset.hex"
                v-tooltip.top="preset.label"
                type="button"
                class="w-6 h-6 rounded-full border border-(--border) flex items-center justify-center transition-transform hover:scale-110 cursor-pointer"
                :style="{ backgroundColor: preset.hex }"
                @click="selectColor(preset.hex)"
              >
                <Check v-if="selectedColor === preset.hex" class="w-3.5 h-3.5 text-white" />
              </button>
              <Button
                v-if="selectedColor"
                v-tooltip.top="'Clear Color'"
                size="small"
                variant="text"
                severity="secondary"
                class="p-0 text-[11px] text-muted-color hover:text-primary ml-1"
                @click="selectedColor = null"
              >
                Clear
              </Button>
            </div>
          </div>

          <!-- Tags -->
          <div class="flex flex-col gap-1.5">
            <label class="font-medium text-primary">Tags</label>
            <div
              class="flex flex-wrap items-center gap-1.5 min-h-7 p-1.5 border border-(--border) rounded-lg bg-(--bg-app)"
            >
              <span
                v-for="t in tags"
                :key="t"
                class="px-2 py-0.5 rounded bg-(--bg-hover) text-primary text-[11px] flex items-center gap-1 border border-(--border)"
              >
                <TagIcon class="w-2.5 h-2.5 text-muted-color" />
                <span>{{ t }}</span>
                <button
                  type="button"
                  class="text-muted-color hover:text-(--danger) cursor-pointer"
                  @click="removeTag(t)"
                >
                  <X class="w-3 h-3" />
                </button>
              </span>
              <InputText
                v-model="newTagInput"
                placeholder="+ Add tag (Enter)..."
                size="small"
                class="flex-1 min-w-24 text-[11px] p-0 border-none bg-transparent shadow-none focus:outline-none"
                @keydown="handleTagKeydown"
              />
            </div>
          </div>
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
          :disabled="isSubmitting"
          @click="handleClose"
        />
        <Button
          severity="primary"
          size="small"
          :loading="isSubmitting"
          :disabled="!canSubmit"
          class="flex items-center gap-1.5 px-4"
          @click="handleAddClusterSubmit"
        >
          <Plus class="w-3.5 h-3.5" />
          <span>Add Cluster</span>
        </Button>
      </div>
    </template>
  </Dialog>
</template>
