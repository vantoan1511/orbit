<script setup lang="ts">
import KeyValueEditor, { type KeyValuePair } from '@/components/shared/KeyValueEditor.vue'
import type { ConfigMap } from 'kubernetes-types/core/v1'
import Tab from 'primevue/tab'
import TabList from 'primevue/tablist'
import TabPanel from 'primevue/tabpanel'
import TabPanels from 'primevue/tabpanels'
import Tabs from 'primevue/tabs'
import { computed, ref, toRaw, watch } from 'vue'

const props = defineProps<{
  rawData: ConfigMap | null
}>()

const emit = defineEmits<{
  (e: 'update:rawData', value: ConfigMap): void
  (e: 'update:isValid', value: boolean): void
}>()

const activeTab = ref('data')

const dataItems = ref<KeyValuePair[]>([])
const labels = ref<KeyValuePair[]>([])
const annotations = ref<KeyValuePair[]>([])

let isEmitting = false

const isFormValid = computed(() => {
  return true
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

const syncFromRawData = (data: ConfigMap | null) => {
  if (!data || isEmitting) return

  // Metadata
  labels.value = kvObjectToArray(data.metadata?.labels)
  annotations.value = kvObjectToArray(data.metadata?.annotations)

  // Data
  dataItems.value = kvObjectToArray(data.data)
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

  const rawObj = JSON.parse(JSON.stringify(toRaw(props.rawData))) as ConfigMap
  if (!rawObj.metadata) rawObj.metadata = {}

  // 1. Metadata
  rawObj.metadata.labels = kvArrayToObject(labels.value)
  rawObj.metadata.annotations = kvArrayToObject(annotations.value)

  // 2. Data
  const dataObj = kvArrayToObject(dataItems.value)
  if (Object.keys(dataObj).length > 0) {
    rawObj.data = dataObj
  } else {
    delete rawObj.data
  }

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
        <Tab value="data" class="text-xs font-medium">Data</Tab>
        <Tab value="metadata" class="text-xs font-medium">Metadata</Tab>
      </TabList>

      <TabPanels class="flex-1 overflow-y-auto pt-6 px-0">
        <!-- DATA TAB -->
        <TabPanel value="data" class="flex flex-col gap-6 max-w-7xl">
          <div class="grid grid-cols-1 md:grid-cols-12 gap-6">
            <div class="md:col-span-3 flex flex-col gap-1">
              <span class="text-xs font-semibold tracking-wider text-muted-color uppercase">
                Configuration Data
              </span>
              <p class="text-xs text-muted-color leading-relaxed">
                Add and edit configuration key-value entries.
              </p>
            </div>
            <div class="md:col-span-9 flex flex-col gap-6">
              <div class="p-4 rounded-lg bg-(--bg-hover)/30 flex flex-col gap-3">
                <KeyValueEditor
                  v-model="dataItems"
                  title="ConfigMap Data (Key-Value pairs)"
                  key-placeholder="Key"
                  value-placeholder="Value"
                  add-label="Add entry"
                  @update:model-value="handleFieldChange"
                />
              </div>
            </div>
          </div>
        </TabPanel>

        <!-- METADATA TAB -->
        <TabPanel value="metadata" class="flex flex-col gap-10 max-w-7xl">
          <div class="grid grid-cols-1 md:grid-cols-12 gap-6">
            <div class="md:col-span-3 flex flex-col gap-1">
              <span class="text-xs font-semibold tracking-wider text-muted-color uppercase">
                ConfigMap Metadata
              </span>
              <p class="text-xs text-muted-color leading-relaxed">
                Labels and annotations attached to this ConfigMap resource.
              </p>
            </div>
            <div class="md:col-span-9 flex flex-col gap-6">
              <div class="p-4 rounded-lg bg-(--bg-hover)/30 flex flex-col gap-3">
                <KeyValueEditor
                  v-model="labels"
                  title="ConfigMap Labels"
                  add-label="Add Label"
                  @update:model-value="handleFieldChange"
                />
              </div>
              <div class="p-4 rounded-lg bg-(--bg-hover)/30 flex flex-col gap-3">
                <KeyValueEditor
                  v-model="annotations"
                  title="ConfigMap Annotations"
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
