<script setup lang="ts">
import { ref } from 'vue'
import Drawer from 'primevue/drawer'
import Tabs from 'primevue/tabs'
import TabList from 'primevue/tablist'
import Tab from 'primevue/tab'
import TabPanels from 'primevue/tabpanels'
import TabPanel from 'primevue/tabpanel'
import { Clock, Tag, Server, Shield, Activity, FileCode, ExternalLink } from '@lucide/vue'
import type { ServiceInfo } from './mockServices'

const props = defineProps<{
  visible: boolean
  service: ServiceInfo | null
}>()

const emit = defineEmits<{
  (e: 'update:visible', value: boolean): void
}>()

const activeTab = ref('overview')

const generateYaml = (s: ServiceInfo) => {
  return `apiVersion: v1
kind: Service
metadata:
  name: ${s.name}
  namespace: ${s.namespace}
  uid: ${s.uid}
  creationTimestamp: "${s.created}"
  labels:
${Object.entries(s.labels)
  .map(([k, v]) => `    ${k}: ${v}`)
  .join('\n')}
spec:
  clusterIP: ${s.clusterIP}
  type: ${s.type}
  sessionAffinity: ${s.sessionAffinity}
  internalTrafficPolicy: ${s.internalTrafficPolicy}
  selector:
${Object.entries(s.selector)
  .map(([k, v]) => `    ${k}: ${v}`)
  .join('\n')}
  ports:
${s.portsList
  .map(
    (p) => `  - port: ${p.port}
    protocol: ${p.protocol}
    targetPort: ${p.targetPort}${p.nodePort ? `\n    nodePort: ${p.nodePort}` : ''}`
  )
  .join('\n')}
status:
  loadBalancer: ${s.type === 'LoadBalancer' ? `\n    ingress:\n    - ip: ${s.externalIP}` : '{}'}
`
}

const getTypeBadgeClass = (type: string) => {
  switch (type) {
    case 'LoadBalancer':
      return 'bg-purple-500/10 text-purple-400 border border-purple-500/20'
    case 'ClusterIP':
      return 'bg-emerald-500/10 text-emerald-400 border border-emerald-500/20'
    case 'NodePort':
      return 'bg-blue-500/10 text-blue-400 border border-blue-500/20'
    case 'ExternalName':
      return 'bg-amber-500/10 text-amber-400 border border-amber-500/20'
    default:
      return 'bg-gray-500/10 text-gray-400 border border-gray-500/20'
  }
}
</script>

<template>
  <Drawer
    :visible="props.visible"
    @update:visible="emit('update:visible', $event)"
    position="right"
    class="w-full sm:max-w-lg border-l border-(--border) bg-(--bg-card) p-0"
    :header="props.service?.name || 'Service Details'"
    :style="{ width: '36rem' }"
  >
    <template #header>
      <div class="flex items-center gap-3 w-full" v-if="props.service">
        <div class="flex items-center gap-1.5">
          <span class="w-2.5 h-2.5 rounded-full bg-emerald-500"></span>
          <span class="text-xs font-bold uppercase tracking-wider text-(--text-muted)">
            Active
          </span>
        </div>
        <div
          class="text-xs text-(--text-muted) font-mono bg-(--bg-hover) px-2 py-0.5 rounded border border-(--border)"
        >
          ns/{{ props.service.namespace }}
        </div>
        <div
          class="text-[10px] font-semibold uppercase tracking-wider font-ui border px-2 py-0.5 rounded"
          :class="getTypeBadgeClass(props.service.type)"
        >
          {{ props.service.type }}
        </div>
      </div>
    </template>

    <div v-if="props.service" class="h-full flex flex-col">
      <!-- Title Section -->
      <div class="p-6 border-b border-(--border) bg-(--bg-hover)/50">
        <h2
          class="text-lg font-bold text-(--text-primary) font-ui truncate mb-1"
          :title="props.service.name"
        >
          {{ props.service.name }}
        </h2>
        <div class="text-xs text-(--text-muted) flex items-center gap-2">
          <Clock class="w-3.5 h-3.5" />
          <span>Age: {{ props.service.age }}</span>
        </div>
      </div>

      <!-- Tab Layout -->
      <div class="flex-1 flex flex-col min-h-0">
        <Tabs v-model:value="activeTab" class="flex-1 flex flex-col">
          <TabList class="border-b border-(--border) px-6 bg-(--bg-card)">
            <Tab
              value="overview"
              class="py-3 px-4 text-xs font-bold uppercase tracking-wider flex items-center gap-1"
            >
              <Server class="w-3.5 h-3.5" />
              <span>Overview</span>
            </Tab>
            <Tab
              value="endpoints"
              class="py-3 px-4 text-xs font-bold uppercase tracking-wider flex items-center gap-1"
            >
              <Shield class="w-3.5 h-3.5" />
              <span>Endpoints</span>
            </Tab>
            <Tab
              value="ports"
              class="py-3 px-4 text-xs font-bold uppercase tracking-wider flex items-center gap-1"
            >
              <Activity class="w-3.5 h-3.5" />
              <span>Ports</span>
            </Tab>
            <Tab
              value="yaml"
              class="py-3 px-4 text-xs font-bold uppercase tracking-wider flex items-center gap-1"
            >
              <FileCode class="w-3.5 h-3.5" />
              <span>YAML</span>
            </Tab>
          </TabList>

          <TabPanels class="p-6 flex-1 overflow-y-auto min-h-0">
            <!-- OVERVIEW PANEL -->
            <TabPanel value="overview" class="space-y-6">
              <!-- General Info Grid -->
              <div class="space-y-4">
                <h3 class="text-xs font-bold text-(--text-muted) uppercase tracking-wider">
                  General
                </h3>
                <div
                  class="border border-(--border) rounded-xl overflow-hidden divide-y divide-(--border) bg-(--bg-hover)/10 text-xs"
                >
                  <div class="grid grid-cols-3 p-3">
                    <span class="text-(--text-secondary) font-semibold">Namespace</span>
                    <span class="col-span-2 font-mono text-(--text-primary)">{{
                      props.service.namespace
                    }}</span>
                  </div>
                  <div class="grid grid-cols-3 p-3">
                    <span class="text-(--text-secondary) font-semibold">Cluster IP</span>
                    <span class="col-span-2 font-mono text-(--text-primary)">{{
                      props.service.clusterIP
                    }}</span>
                  </div>
                  <div class="grid grid-cols-3 p-3">
                    <span class="text-(--text-secondary) font-semibold">External IP</span>
                    <span
                      class="col-span-2 font-mono text-(--text-primary) flex items-center gap-1.5"
                    >
                      {{ props.service.externalIP }}
                      <ExternalLink
                        v-if="props.service.externalIP !== '-'"
                        class="w-3 h-3 text-violet-400 cursor-pointer"
                      />
                    </span>
                  </div>
                  <div class="grid grid-cols-3 p-3">
                    <span class="text-(--text-secondary) font-semibold">Type</span>
                    <span class="col-span-2 font-mono text-(--text-primary)">{{
                      props.service.type
                    }}</span>
                  </div>
                  <div class="grid grid-cols-3 p-3">
                    <span class="text-(--text-secondary) font-semibold">Session Affinity</span>
                    <span class="col-span-2 text-(--text-primary)">{{
                      props.service.sessionAffinity
                    }}</span>
                  </div>
                  <div class="grid grid-cols-3 p-3">
                    <span class="text-(--text-secondary) font-semibold"
                      >Internal Traffic Policy</span
                    >
                    <span class="col-span-2 text-(--text-primary)">{{
                      props.service.internalTrafficPolicy
                    }}</span>
                  </div>
                  <div class="grid grid-cols-3 p-3">
                    <span class="text-(--text-secondary) font-semibold">Created</span>
                    <span class="col-span-2 text-(--text-primary)">{{
                      props.service.created
                    }}</span>
                  </div>
                  <div class="grid grid-cols-3 p-3">
                    <span class="text-(--text-secondary) font-semibold">Age</span>
                    <span class="col-span-2 text-(--text-primary)">{{ props.service.age }}</span>
                  </div>
                  <div class="grid grid-cols-3 p-3">
                    <span class="text-(--text-secondary) font-semibold">UID</span>
                    <span class="col-span-2 font-mono text-[10px] text-(--text-primary)">{{
                      props.service.uid
                    }}</span>
                  </div>
                </div>
              </div>

              <!-- Selector Section -->
              <div class="space-y-3">
                <h3
                  class="text-xs font-bold text-(--text-muted) uppercase tracking-wider flex items-center gap-1.5"
                >
                  <Tag class="w-3.5 h-3.5" />
                  <span>Selector</span>
                </h3>
                <div class="flex flex-wrap gap-2">
                  <div
                    v-for="(val, key) in props.service.selector"
                    :key="key"
                    class="flex items-center text-xs border border-violet-500/20 bg-violet-500/5 text-violet-400 rounded-lg overflow-hidden"
                  >
                    <span
                      class="px-2 py-1 bg-violet-500/10 border-r border-violet-500/20 font-medium"
                      >{{ key }}</span
                    >
                    <span class="px-2 py-1 font-mono">{{ val }}</span>
                  </div>
                  <span
                    v-if="Object.keys(props.service.selector).length === 0"
                    class="text-xs text-(--text-muted) italic"
                    >None</span
                  >
                </div>
              </div>

              <!-- Labels Section -->
              <div class="space-y-3">
                <h3
                  class="text-xs font-bold text-(--text-muted) uppercase tracking-wider flex items-center gap-1.5"
                >
                  <Tag class="w-3.5 h-3.5" />
                  <span>Labels</span>
                </h3>
                <div class="flex flex-wrap gap-2">
                  <div
                    v-for="(val, key) in props.service.labels"
                    :key="key"
                    class="flex items-center text-xs border border-(--border) bg-(--bg-hover)/20 rounded-lg overflow-hidden"
                  >
                    <span
                      class="px-2 py-1 bg-(--bg-hover)/40 border-r border-(--border) text-(--text-secondary) font-medium"
                      >{{ key }}</span
                    >
                    <span class="px-2 py-1 font-mono text-(--text-primary)">{{ val }}</span>
                  </div>
                </div>
              </div>
            </TabPanel>

            <!-- ENDPOINTS PANEL -->
            <TabPanel value="endpoints" class="space-y-4">
              <h3 class="text-xs font-bold text-(--text-muted) uppercase tracking-wider">
                Target Endpoints
              </h3>
              <div class="border border-(--border) rounded-xl p-4 bg-(--bg-hover)/10">
                <ul class="space-y-2" v-if="props.service.endpointsList.length > 0">
                  <li
                    v-for="(ep, idx) in props.service.endpointsList"
                    :key="idx"
                    class="font-mono text-xs text-(--text-primary) flex items-center gap-2"
                  >
                    <span class="w-1.5 h-1.5 rounded-full bg-emerald-500"></span>
                    {{ ep }}
                  </li>
                </ul>
                <div v-else class="text-xs text-(--text-muted) italic">
                  No active endpoints (ExternalName or selectorless service).
                </div>
              </div>
            </TabPanel>

            <!-- PORTS PANEL -->
            <TabPanel value="ports" class="space-y-4">
              <h3 class="text-xs font-bold text-(--text-muted) uppercase tracking-wider">
                Port Mappings
              </h3>
              <div
                class="border border-(--border) rounded-xl overflow-hidden bg-(--bg-hover)/10 text-xs"
              >
                <table class="w-full text-left border-collapse">
                  <thead>
                    <tr
                      class="bg-(--bg-hover)/40 border-b border-(--border) text-(--text-muted) font-semibold"
                    >
                      <th class="p-3">Port</th>
                      <th class="p-3">Target Port</th>
                      <th class="p-3">Protocol</th>
                      <th class="p-3">Node Port</th>
                    </tr>
                  </thead>
                  <tbody class="divide-y divide-(--border)">
                    <tr
                      v-for="(port, idx) in props.service.portsList"
                      :key="idx"
                      class="text-(--text-secondary) hover:bg-(--bg-hover)/10"
                    >
                      <td class="p-3 font-mono text-(--text-primary)">{{ port.port }}</td>
                      <td class="p-3 font-mono">{{ port.targetPort }}</td>
                      <td class="p-3 font-semibold">{{ port.protocol }}</td>
                      <td class="p-3 font-mono">{{ port.nodePort || '-' }}</td>
                    </tr>
                  </tbody>
                </table>
              </div>
            </TabPanel>

            <!-- YAML PANEL -->
            <TabPanel value="yaml" class="h-full flex flex-col gap-2">
              <div
                class="flex-1 min-h-64 border border-(--border) rounded-xl bg-zinc-950 p-4 overflow-y-auto"
              >
                <pre class="font-mono text-[10px] text-zinc-300 leading-relaxed">{{
                  generateYaml(props.service)
                }}</pre>
              </div>
            </TabPanel>
          </TabPanels>
        </Tabs>
      </div>
    </div>
  </Drawer>
</template>

<style scoped>
:deep(.p-tabs) {
  display: flex;
  flex-direction: column;
  height: 100%;
}
:deep(.p-tablist-content) {
  border-bottom: none !important;
}
:deep(.p-tablist-tab-inline) {
  border-bottom: none !important;
}
</style>
