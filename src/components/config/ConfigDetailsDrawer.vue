<script setup lang="ts">
import { ref, watch } from 'vue'
import Drawer from 'primevue/drawer'
import Tabs from 'primevue/tabs'
import TabList from 'primevue/tablist'
import Tab from 'primevue/tab'
import TabPanels from 'primevue/tabpanels'
import TabPanel from 'primevue/tabpanel'
import { Clock, Tag, Server, Shield, FileCode, Eye, EyeOff } from '@lucide/vue'
import type { ConfigMapInfo, SecretInfo } from '@/types/kubernetes'

const props = defineProps<{
  visible: boolean
  resource: ConfigMapInfo | SecretInfo | null
}>()

const emit = defineEmits<{
  (e: 'update:visible', value: boolean): void
}>()

const activeTab = ref('overview')
const revealedKeys = ref<Record<string, boolean>>({})

// Reset states when resource changes
watch(
  () => props.resource,
  () => {
    revealedKeys.value = {}
    activeTab.value = 'overview'
  }
)

const isSecret = (res: ConfigMapInfo | SecretInfo): res is SecretInfo => {
  return 'type' in res
}

const toggleRevealKey = (key: string) => {
  revealedKeys.value[key] = !revealedKeys.value[key]
}

const decodeSecretValue = (val: string) => {
  try {
    return atob(val)
  } catch {
    return val // Return raw if not valid base64
  }
}

const generateYaml = (res: ConfigMapInfo | SecretInfo) => {
  const isSec = isSecret(res)
  const baseYaml = `apiVersion: v1
kind: ${isSec ? 'Secret' : 'ConfigMap'}
metadata:
  name: ${res.name}
  namespace: ${res.namespace}
  uid: ${res.resourceVersion}3df1-4ae8-8288-ee2b${res.resourceVersion}
  creationTimestamp: "${res.created}"
  resourceVersion: "${res.resourceVersion}"
  labels:
${Object.entries(res.labels)
  .map(([k, v]) => `    ${k}: ${v}`)
  .join('\n')}
${isSec ? `type: ${res.type}\n` : ''}data:
${Object.entries(res.data)
  .map(
    ([k, v]) =>
      `  ${k}: ${
        isSec
          ? v
          : `|-\n` +
            v
              .split('\n')
              .map((line) => `    ${line}`)
              .join('\n')
      }`
  )
  .join('\n')}
`
  return baseYaml
}
</script>

<template>
  <Drawer
    :visible="props.visible"
    @update:visible="emit('update:visible', $event)"
    position="right"
    class="w-full sm:max-w-lg border-l border-(--border) bg-(--bg-card) p-0"
    :header="props.resource?.name || 'Resource Details'"
    :style="{ width: '36rem' }"
  >
    <template #header>
      <div class="flex items-center gap-3 w-full" v-if="props.resource">
        <div class="flex items-center gap-1.5">
          <span class="w-2.5 h-2.5 rounded-full bg-emerald-500"></span>
          <span class="text-xs font-bold uppercase tracking-wider text-(--text-muted)">
            Active
          </span>
        </div>
        <div
          class="text-xs text-(--text-muted) font-mono bg-(--bg-hover) px-2 py-0.5 rounded border border-(--border)"
        >
          ns/{{ props.resource.namespace }}
        </div>
        <div
          v-if="isSecret(props.resource)"
          class="text-[10px] font-semibold uppercase tracking-wider font-ui border px-2 py-0.5 rounded bg-rose-500/10 text-rose-400 border-rose-500/20"
        >
          Secret
        </div>
        <div
          v-else
          class="text-[10px] font-semibold uppercase tracking-wider font-ui border px-2 py-0.5 rounded bg-sky-500/10 text-sky-400 border-sky-500/20"
        >
          ConfigMap
        </div>
      </div>
    </template>

    <div v-if="props.resource" class="h-full flex flex-col">
      <!-- Title Section -->
      <div class="p-6 border-b border-(--border) bg-(--bg-hover)/50">
        <h2
          class="text-lg font-bold text-(--text-primary) font-ui truncate mb-1"
          :title="props.resource.name"
        >
          {{ props.resource.name }}
        </h2>
        <div class="text-xs text-(--text-muted) flex items-center gap-2">
          <Clock class="w-3.5 h-3.5" />
          <span>Age: {{ props.resource.age }}</span>
        </div>
      </div>

      <!-- Tab Layout -->
      <div class="flex-1 flex flex-col min-h-0">
        <Tabs v-model:value="activeTab" class="flex-1 flex flex-col">
          <TabList class="border-b border-(--border) px-6 bg-(--bg-card)">
            <Tab
              value="overview"
              class="py-3 px-4 text-xs font-bold uppercase tracking-wider flex items-center gap-1.5"
            >
              <Server class="w-3.5 h-3.5" />
              <span>Overview</span>
            </Tab>
            <Tab
              value="data"
              class="py-3 px-4 text-xs font-bold uppercase tracking-wider flex items-center gap-1.5"
            >
              <Shield class="w-3.5 h-3.5" />
              <span>Data</span>
            </Tab>
            <Tab
              value="yaml"
              class="py-3 px-4 text-xs font-bold uppercase tracking-wider flex items-center gap-1.5"
            >
              <FileCode class="w-3.5 h-3.5" />
              <span>YAML</span>
            </Tab>
          </TabList>

          <TabPanels class="flex-1 overflow-y-auto p-6">
            <!-- Overview Panel -->
            <TabPanel value="overview">
              <div class="flex flex-col gap-6">
                <!-- General Section -->
                <div>
                  <h3 class="text-xs font-bold uppercase text-(--text-muted) tracking-wider mb-3">
                    General
                  </h3>
                  <div class="border border-(--border) rounded-lg overflow-hidden bg-(--bg-card)">
                    <div class="grid grid-cols-3 border-b border-(--border) p-3">
                      <div class="text-xs text-(--text-secondary) font-medium">Namespace</div>
                      <div class="col-span-2 text-xs text-(--text-primary) font-mono">
                        {{ props.resource.namespace }}
                      </div>
                    </div>
                    <div class="grid grid-cols-3 border-b border-(--border) p-3">
                      <div class="text-xs text-(--text-secondary) font-medium">Labels</div>
                      <div class="col-span-2 flex flex-wrap gap-1">
                        <span
                          v-for="(val, key) in props.resource.labels"
                          :key="key"
                          class="font-mono text-[10px] bg-(--bg-hover) text-(--text-secondary) border border-(--border) px-1.5 py-0.5 rounded"
                        >
                          {{ key }}={{ val }}
                        </span>
                        <span
                          v-if="Object.keys(props.resource.labels).length === 0"
                          class="text-xs text-(--text-muted)"
                        >
                          None
                        </span>
                      </div>
                    </div>
                    <div class="grid grid-cols-3 border-b border-(--border) p-3">
                      <div class="text-xs text-(--text-secondary) font-medium">Annotations</div>
                      <div class="col-span-2 text-xs text-(--text-primary)">
                        {{ props.resource.annotations }}
                      </div>
                    </div>
                    <div class="grid grid-cols-3 border-b border-(--border) p-3">
                      <div class="text-xs text-(--text-secondary) font-medium">Created</div>
                      <div class="col-span-2 text-xs text-(--text-primary)">
                        {{ props.resource.created }}
                      </div>
                    </div>
                    <div class="grid grid-cols-3 border-b border-(--border) p-3">
                      <div class="text-xs text-(--text-secondary) font-medium">Age</div>
                      <div class="col-span-2 text-xs text-(--text-primary)">
                        {{ props.resource.age }}
                      </div>
                    </div>
                    <div class="grid grid-cols-3 p-3">
                      <div class="text-xs text-(--text-secondary) font-medium">
                        Resource Version
                      </div>
                      <div class="col-span-2 text-xs text-(--text-primary) font-mono">
                        {{ props.resource.resourceVersion }}
                      </div>
                    </div>
                  </div>
                </div>

                <!-- Details Section -->
                <div>
                  <h3 class="text-xs font-bold uppercase text-(--text-muted) tracking-wider mb-3">
                    Details
                  </h3>
                  <div class="border border-(--border) rounded-lg overflow-hidden bg-(--bg-card)">
                    <div class="grid grid-cols-3 border-b border-(--border) p-3">
                      <div class="text-xs text-(--text-secondary) font-medium">Data Keys</div>
                      <div class="col-span-2 text-xs text-(--text-primary)">
                        {{ props.resource.keysCount }}
                      </div>
                    </div>
                    <div class="grid grid-cols-3 border-b border-(--border) p-3">
                      <div class="text-xs text-(--text-secondary) font-medium">Size</div>
                      <div class="col-span-2 text-xs text-(--text-primary) font-mono">
                        {{ props.resource.size }}
                      </div>
                    </div>
                    <div class="grid grid-cols-3 border-b border-(--border) p-3">
                      <div class="text-xs text-(--text-secondary) font-medium">Mounted In</div>
                      <div class="col-span-2 text-xs text-(--text-primary)">
                        {{ props.resource.mountedPods }} pods
                      </div>
                    </div>
                    <div class="grid grid-cols-3 p-3">
                      <div class="text-xs text-(--text-secondary) font-medium">Immutable</div>
                      <div class="col-span-2 text-xs text-(--text-primary)">
                        {{ props.resource.immutable ? 'True' : 'False' }}
                      </div>
                    </div>
                  </div>
                </div>

                <!-- Used By Section -->
                <div>
                  <div class="flex items-center justify-between mb-3">
                    <h3 class="text-xs font-bold uppercase text-(--text-muted) tracking-wider">
                      Used By
                    </h3>
                    <a
                      href="#"
                      class="text-xs text-violet-400 hover:text-violet-300 font-semibold flex items-center gap-1"
                    >
                      <span>View all ({{ props.resource.mountedPods }})</span>
                      <span class="text-[10px]">&rarr;</span>
                    </a>
                  </div>
                  <div
                    class="border border-(--border) rounded-lg overflow-hidden bg-(--bg-card) divide-y divide-(--border)"
                  >
                    <div
                      v-for="pod in props.resource.usedBy"
                      :key="pod.name"
                      class="flex items-center justify-between p-3"
                    >
                      <div class="flex items-center gap-2">
                        <Tag class="w-3.5 h-3.5 text-violet-400" />
                        <span class="text-xs font-mono text-(--text-primary) truncate max-w-72">
                          {{ pod.name }}
                        </span>
                      </div>
                      <span
                        class="text-[10px] font-semibold px-2 py-0.5 rounded-full"
                        :class="[
                          pod.status === 'Running'
                            ? 'bg-emerald-500/10 text-emerald-400 border border-emerald-500/20'
                            : 'bg-zinc-500/10 text-zinc-400 border border-zinc-500/20'
                        ]"
                      >
                        {{ pod.status }}
                      </span>
                    </div>
                    <div
                      v-if="props.resource.usedBy.length === 0"
                      class="p-6 text-center text-xs text-(--text-muted)"
                    >
                      No pods currently referencing this configuration.
                    </div>
                  </div>
                </div>
              </div>
            </TabPanel>

            <!-- Data Panel -->
            <TabPanel value="data">
              <div class="flex flex-col gap-4">
                <div
                  v-for="(value, key) in props.resource.data"
                  :key="key"
                  class="border border-(--border) rounded-lg bg-(--bg-hover)/20 p-4"
                >
                  <div
                    class="flex items-center justify-between border-b border-(--border)/50 pb-2 mb-2"
                  >
                    <span class="text-xs font-bold font-mono text-violet-400">{{ key }}</span>

                    <button
                      v-if="isSecret(props.resource)"
                      @click="toggleRevealKey(key)"
                      class="p-1 rounded text-(--text-muted) hover:text-(--text-primary) hover:bg-(--bg-hover) transition-all duration-200"
                      title="Toggle visibility"
                    >
                      <EyeOff v-if="revealedKeys[key]" class="w-3.5 h-3.5" />
                      <Eye v-else class="w-3.5 h-3.5" />
                    </button>
                  </div>

                  <pre
                    class="text-xs font-mono p-2 rounded bg-(--bg-card) border border-(--border) overflow-x-auto whitespace-pre-wrap break-all select-all text-(--text-primary)"
                    >{{
                      isSecret(props.resource)
                        ? revealedKeys[key]
                          ? decodeSecretValue(value)
                          : '••••••••••••••••'
                        : value
                    }}</pre>
                </div>
              </div>
            </TabPanel>

            <!-- YAML Panel -->
            <TabPanel value="yaml">
              <pre
                class="text-xs font-mono p-4 rounded-lg bg-(--bg-card) border border-(--border) text-(--text-primary) overflow-x-auto whitespace-pre select-all"
                >{{ generateYaml(props.resource) }}</pre>
            </TabPanel>
          </TabPanels>
        </Tabs>
      </div>
    </div>
  </Drawer>
</template>
