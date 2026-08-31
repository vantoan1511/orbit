<script setup lang="ts">
import KeyValueEditor, { type KeyValuePair } from '@/components/shared/KeyValueEditor.vue'
import { decodeBase64, encodeBase64 } from '@/utils/text'
import type { Secret } from 'kubernetes-types/core/v1'
import Select from 'primevue/select'
import Tab from 'primevue/tab'
import TabList from 'primevue/tablist'
import TabPanel from 'primevue/tabpanel'
import TabPanels from 'primevue/tabpanels'
import Tabs from 'primevue/tabs'
import { computed, ref, toRaw, watch } from 'vue'

const props = defineProps<{
  rawData: Secret | null
}>()

const emit = defineEmits<{
  (e: 'update:rawData', value: Secret): void
  (e: 'update:isValid', value: boolean): void
}>()

const activeTab = ref('data')

const secretTypeOptions = [
  'Opaque',
  'kubernetes.io/tls',
  'kubernetes.io/dockerconfigjson',
  'kubernetes.io/basic-auth',
  'kubernetes.io/ssh-auth',
  'kubernetes.io/service-account-token'
]

const secretType = ref<string>('Opaque')
const dataItems = ref<KeyValuePair[]>([])
const labels = ref<KeyValuePair[]>([])
const annotations = ref<KeyValuePair[]>([])

let isEmitting = false

const isFormValid = computed(() => {
  return Boolean(secretType.value.trim())
})

watch(
  isFormValid,
  (val) => {
    emit('update:isValid', val)
  },
  { immediate: true }
)

const kvObjectToArray = (obj: Record<string, string> | undefined): KeyValuePair[] => {
  if (!obj || typeof obj !== 'object') return []
  return Object.entries(obj).map(([key, value]) => ({ key, value: String(value ?? '') }))
}

const kvArrayToObject = (arr: KeyValuePair[]): Record<string, string> => {
  const res: Record<string, string> = {}
  for (const item of arr) {
    if (item.key.trim()) {
      res[item.key.trim()] = item.value
    }
  }
  return res
}

const syncFromRawData = (data: Secret | null) => {
  if (!data || isEmitting) return

  secretType.value = data.type || 'Opaque'

  // Metadata
  labels.value = kvObjectToArray(data.metadata?.labels)
  annotations.value = kvObjectToArray(data.metadata?.annotations)

  // Data: decode Base64 to plain text key-value pairs
  const decodedPairs: KeyValuePair[] = []
  const rawDataEntries = data.data || {}
  for (const [k, v] of Object.entries(rawDataEntries)) {
    decodedPairs.push({ key: k, value: decodeBase64(v) })
  }

  // Also merge any existing stringData if present
  if (data.stringData) {
    for (const [k, v] of Object.entries(data.stringData)) {
      const existing = decodedPairs.find((p) => p.key === k)
      if (existing) {
        existing.value = v
      } else {
        decodedPairs.push({ key: k, value: v })
      }
    }
  }

  dataItems.value = decodedPairs
}

watch(
  () => props.rawData,
  (newData) => {
    syncFromRawData(newData)
  },
  { immediate: true, deep: true }
)

const emitUpdate = () => {
  if (!props.rawData) return
  isEmitting = true

  const rawObj = JSON.parse(JSON.stringify(toRaw(props.rawData))) as Secret
  if (!rawObj.metadata) rawObj.metadata = {}

  // 1. Secret Type
  rawObj.type = secretType.value

  // 2. Metadata
  rawObj.metadata.labels = kvArrayToObject(labels.value)
  rawObj.metadata.annotations = kvArrayToObject(annotations.value)

  // 3. Data: encode plain text back to Base64
  const encoded: Record<string, string> = {}
  for (const item of dataItems.value) {
    if (item.key.trim()) {
      encoded[item.key.trim()] = encodeBase64(item.value)
    }
  }

  if (Object.keys(encoded).length > 0) {
    rawObj.data = encoded
  } else {
    delete rawObj.data
  }
  delete rawObj.stringData

  emit('update:rawData', rawObj)

  setTimeout(() => {
    isEmitting = false
  }, 50)
}

const handleFieldChange = () => {
  if (!isEmitting) {
    emitUpdate()
  }
}
</script>

<template>
  <div class="flex flex-col h-full w-full">
    <Tabs v-model:value="activeTab" class="w-full flex flex-col h-full">
      <TabList>
        <Tab value="data" class="text-xs font-medium">Data (Decoded)</Tab>
        <Tab value="metadata" class="text-xs font-medium">Metadata & Settings</Tab>
      </TabList>

      <TabPanels class="flex-1 overflow-y-auto pt-6 px-0">
        <!-- DATA TAB -->
        <TabPanel value="data" class="flex flex-col gap-6 max-w-7xl">
          <div class="grid grid-cols-1 md:grid-cols-12 gap-6">
            <div class="md:col-span-3 flex flex-col gap-1">
              <span class="text-xs font-semibold tracking-wider text-muted-color uppercase">
                Secret Payload
              </span>
              <p class="text-xs text-muted-color leading-relaxed">
                Values are automatically decoded for editing and re-encoded in Base64 on save.
              </p>
            </div>
            <div class="md:col-span-9 flex flex-col gap-6">
              <div class="p-4 rounded-lg bg-(--bg-hover)/30 flex flex-col gap-3">
                <KeyValueEditor
                  v-model="dataItems"
                  title="Secret Data (Key-Value pairs)"
                  key-placeholder="Key"
                  value-placeholder="Plain text value"
                  add-label="Add entry"
                  @update:model-value="handleFieldChange"
                />
              </div>
            </div>
          </div>
        </TabPanel>

        <!-- METADATA TAB -->
        <TabPanel value="metadata" class="flex flex-col gap-10 max-w-7xl">
          <!-- Secret Type Section -->
          <div class="grid grid-cols-1 md:grid-cols-12 gap-6">
            <div class="md:col-span-3 flex flex-col gap-1">
              <span class="text-xs font-semibold tracking-wider text-muted-color uppercase">
                Secret Configuration
              </span>
              <p class="text-xs text-muted-color leading-relaxed">
                Secret type determines how Kubernetes validates and mounts credentials.
              </p>
            </div>
            <div class="md:col-span-9 flex flex-col gap-3">
              <div class="flex flex-col gap-1.5 max-w-sm">
                <label class="text-xs font-medium text-muted-color">Secret Type</label>
                <Select
                  v-model="secretType"
                  :options="secretTypeOptions"
                  editable
                  size="small"
                  fluid
                  class="text-xs"
                  @change="handleFieldChange"
                />
              </div>
            </div>
          </div>

          <!-- Metadata Section -->
          <div class="grid grid-cols-1 md:grid-cols-12 gap-6">
            <div class="md:col-span-3 flex flex-col gap-1">
              <span class="text-xs font-semibold tracking-wider text-muted-color uppercase">
                Secret Metadata
              </span>
              <p class="text-xs text-muted-color leading-relaxed">
                Labels and annotations attached to this Secret resource.
              </p>
            </div>
            <div class="md:col-span-9 flex flex-col gap-6">
              <div class="p-4 rounded-lg bg-(--bg-hover)/30 flex flex-col gap-3">
                <KeyValueEditor
                  v-model="labels"
                  title="Secret Labels"
                  add-label="Add Label"
                  @update:model-value="handleFieldChange"
                />
              </div>
              <div class="p-4 rounded-lg bg-(--bg-hover)/30 flex flex-col gap-3">
                <KeyValueEditor
                  v-model="annotations"
                  title="Secret Annotations"
                  add-label="Add Annotation"
                  @update:model-value="handleFieldChange"
                />
              </div>
            </div>
          </div>
        </TabPanel>
      </TabPanels>
    </Tabs>
  </div>
</template>
