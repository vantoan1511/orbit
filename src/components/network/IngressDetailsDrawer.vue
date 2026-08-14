<script setup lang="ts">
import KeyValueBadgeList from '@/components/shared/KeyValueBadgeList.vue'
import WorkloadEventsTab from '@/components/workloads/WorkloadEventsTab.vue'
import WorkloadYamlTab from '@/components/workloads/WorkloadYamlTab.vue'
import { kubernetesService } from '@/services/kubernetesService'
import { events } from '@/services/nativeService'
import { useKubernetesStore } from '@/stores/kubernetesStore'
import { OrbitEvents } from '@/types/events'
import type { IngressInfo } from '@/types/kubernetes'
import { Activity, Clock, FileCode, Globe, Network, Server } from '@lucide/vue'
import { storeToRefs } from 'pinia'
import Drawer from 'primevue/drawer'
import Tab from 'primevue/tab'
import TabList from 'primevue/tablist'
import TabPanel from 'primevue/tabpanel'
import TabPanels from 'primevue/tabpanels'
import Tabs from 'primevue/tabs'
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import * as yaml from 'yaml'

const props = defineProps<{
  visible: boolean
  ingress: IngressInfo | null
}>()

const emit = defineEmits<{
  (e: 'update:visible', value: boolean): void
}>()

const k8sStore = useKubernetesStore()
const { events: clusterEvents } = storeToRefs(k8sStore)

const activeTab = ref('overview')

const ingressEvents = computed(() => {
  if (!props.ingress) return []
  const ns = props.ingress.namespace
  const name = props.ingress.name
  return clusterEvents.value.filter((ev) => {
    return ev.namespace === ns && ev.objectName === name
  })
})

interface ParsedIngressRule {
  host: string
  path: string
  backend: string
}

// Parse rules summary strings "host -> path (backend:port)" into structured objects
const parsedRules = computed<ParsedIngressRule[]>(() => {
  if (!props.ingress?.rulesSummary) return []
  return props.ingress.rulesSummary.map((rule) => {
    const match = rule.match(/^(.*?)\s*->\s*(.*?)\s*\((.*?)\)$/)
    if (match && match[1] && match[2] && match[3]) {
      return {
        host: match[1],
        path: match[2],
        backend: match[3]
      }
    }
    return {
      host: props.ingress?.hosts || '*',
      path: rule,
      backend: '-'
    }
  })
})

const generateFallbackYaml = (ing: IngressInfo) => {
  return `apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: ${ing.name}
  namespace: ${ing.namespace}
  uid: ${ing.uid}
  creationTimestamp: "${ing.created}"
  labels:
${Object.entries(ing.labels)
  .map(([k, v]) => `    ${k}: ${v}`)
  .join('\n')}
  annotations:
${Object.entries(ing.annotations)
  .map(([k, v]) => `    ${k}: ${v}`)
  .join('\n')}
spec:
  ingressClassName: ${ing.className || 'default'}
  rules:
${parsedRules.value
  .map((r) => {
    const [svcName, svcPort] = (r.backend || '').split(':')
    return `  - host: ${r.host}
    http:
      paths:
      - path: ${r.path}
        pathType: ImplementationSpecific
        backend:
          service:
            name: ${svcName || 'service'}
            port:
              number: ${svcPort || '80'}`
  })
  .join('\n')}
status:
  loadBalancer:
    ingress:
    - ip: ${ing.address}
`
}

const rawYamlData = ref<string | null>(null)
const isYamlLoading = ref<boolean>(false)
const copied = ref<boolean>(false)

const fetchRawYaml = async () => {
  if (!props.ingress || !props.visible) return
  isYamlLoading.value = true
  try {
    await kubernetesService.getResourceRaw({
      namespace: props.ingress.namespace,
      kind: 'Ingress',
      name: props.ingress.name
    })
  } catch (e) {
    console.error('Failed to fetch raw Ingress YAML:', e)
    isYamlLoading.value = false
  }
}

const handleRawData = (payload: { name?: string; kind?: string; data?: unknown }) => {
  if (!props.visible || !props.ingress) return
  if (payload?.name === props.ingress.name && (payload?.kind === 'Ingress' || !payload?.kind)) {
    if (payload.data) {
      rawYamlData.value =
        typeof payload.data === 'string' ? payload.data : JSON.stringify(payload.data)
    }
    isYamlLoading.value = false
  }
}

onMounted(() => {
  events.on(OrbitEvents.ResourceRawData, handleRawData)
})

onUnmounted(() => {
  events.off(OrbitEvents.ResourceRawData, handleRawData)
})

watch(
  () => [props.visible, props.ingress?.name, props.ingress?.namespace],
  ([newVisible]) => {
    if (newVisible && props.ingress) {
      rawYamlData.value = null
      if (activeTab.value === 'yaml') {
        fetchRawYaml()
      }
    }
  }
)

watch(activeTab, (newTab) => {
  if (newTab === 'yaml' && !rawYamlData.value && !isYamlLoading.value) {
    fetchRawYaml()
  }
})

const displayedYaml = computed(() => {
  if (!rawYamlData.value) {
    return props.ingress ? generateFallbackYaml(props.ingress) : ''
  }
  try {
    const parsed = JSON.parse(rawYamlData.value)
    return yaml.stringify(parsed)
  } catch {
    return rawYamlData.value
  }
})

const copyYaml = async () => {
  if (!displayedYaml.value) return
  try {
    await navigator.clipboard.writeText(displayedYaml.value)
    copied.value = true
    setTimeout(() => {
      copied.value = false
    }, 2000)
  } catch (e) {
    console.error('Failed to copy Ingress YAML:', e)
  }
}
</script>

<template>
  <Drawer
    :visible="props.visible"
    @update:visible="emit('update:visible', $event)"
    position="right"
    class="w-full sm:max-w-lg border-l border-(--border) bg-(--bg-card) p-0"
    :header="props.ingress?.name || 'Ingress Details'"
    :style="{ width: '36rem' }"
  >
    <template #header>
      <div class="flex items-center gap-3 w-full" v-if="props.ingress">
        <div class="flex items-center gap-1.5">
          <span class="w-2.5 h-2.5 rounded-full bg-emerald-500"></span>
          <span class="text-xs font-bold uppercase tracking-wider text-muted-color"> Active </span>
        </div>
        <div
          class="text-xs text-muted-color font-mono bg-(--bg-hover) px-2 py-0.5 rounded border border-(--border)"
        >
          ns/{{ props.ingress.namespace }}
        </div>
        <div
          v-if="props.ingress.className"
          class="text-[10px] font-semibold uppercase tracking-wider font-ui border border-purple-500/20 bg-purple-500/10 text-purple-400 px-2 py-0.5 rounded"
        >
          {{ props.ingress.className }}
        </div>
      </div>
    </template>

    <div v-if="props.ingress" class="h-full flex flex-col">
      <!-- Title Section -->
      <div class="p-6 border-b border-(--border) bg-(--bg-hover)/50">
        <h2
          class="text-lg font-bold text-primary font-ui truncate mb-1"
          :title="props.ingress.name"
        >
          {{ props.ingress.name }}
        </h2>
        <div class="text-xs text-muted-color flex items-center gap-2">
          <Clock class="w-3.5 h-3.5" />
          <span>Age: {{ props.ingress.age }}</span>
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
              value="rules"
              class="py-3 px-4 text-xs font-bold uppercase tracking-wider flex items-center gap-1"
            >
              <Network class="w-3.5 h-3.5" />
              <span>Rules ({{ props.ingress.rulesSummary.length }})</span>
            </Tab>
            <Tab
              value="events"
              class="py-3 px-4 text-xs font-bold uppercase tracking-wider flex items-center gap-1"
            >
              <Activity class="w-3.5 h-3.5" />
              <span>Events ({{ ingressEvents.length }})</span>
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
                <h3 class="text-xs font-bold text-muted-color uppercase tracking-wider">General</h3>
                <div
                  class="border border-(--border) rounded-xl overflow-hidden divide-y divide-(--border) bg-(--bg-hover)/10 text-xs"
                >
                  <div class="grid grid-cols-3 p-3">
                    <span class="text-muted-color font-semibold">Namespace</span>
                    <span class="col-span-2 font-mono text-primary">{{
                      props.ingress.namespace
                    }}</span>
                  </div>
                  <div class="grid grid-cols-3 p-3">
                    <span class="text-muted-color font-semibold">Class</span>
                    <span class="col-span-2 font-mono text-primary">{{
                      props.ingress.className || '-'
                    }}</span>
                  </div>
                  <div class="grid grid-cols-3 p-3">
                    <span class="text-muted-color font-semibold">Hosts</span>
                    <span class="col-span-2 font-mono text-violet-400 whitespace-pre-line">{{
                      props.ingress.hosts
                    }}</span>
                  </div>
                  <div class="grid grid-cols-3 p-3">
                    <span class="text-muted-color font-semibold">Address</span>
                    <span class="col-span-2 font-mono text-primary">{{
                      props.ingress.address
                    }}</span>
                  </div>
                  <div class="grid grid-cols-3 p-3">
                    <span class="text-muted-color font-semibold">Ports</span>
                    <span class="col-span-2 font-mono text-primary">{{ props.ingress.ports }}</span>
                  </div>
                  <div class="grid grid-cols-3 p-3">
                    <span class="text-muted-color font-semibold">Created</span>
                    <span class="col-span-2 text-primary">{{ props.ingress.created }}</span>
                  </div>
                  <div class="grid grid-cols-3 p-3">
                    <span class="text-muted-color font-semibold">Age</span>
                    <span class="col-span-2 text-primary">{{ props.ingress.age }}</span>
                  </div>
                  <div class="grid grid-cols-3 p-3">
                    <span class="text-muted-color font-semibold">UID</span>
                    <span class="col-span-2 font-mono text-[10px] text-primary">{{
                      props.ingress.uid
                    }}</span>
                  </div>
                </div>
              </div>

              <!-- Rules Summary Section -->
              <div class="space-y-3" v-if="parsedRules.length > 0">
                <h3
                  class="text-xs font-bold text-muted-color uppercase tracking-wider flex items-center gap-1.5"
                >
                  <Globe class="w-3.5 h-3.5" />
                  <span>Routing Rules</span>
                </h3>
                <div class="space-y-2">
                  <div
                    v-for="(r, idx) in parsedRules"
                    :key="idx"
                    class="p-3 rounded-lg border border-(--border) bg-(--bg-hover)/30 text-xs font-mono flex items-center justify-between gap-2"
                  >
                    <div class="flex items-center gap-2 truncate">
                      <span class="text-violet-400 font-semibold truncate">{{ r.host }}</span>
                      <span class="text-muted-color">→</span>
                      <span class="text-primary truncate">{{ r.path }}</span>
                    </div>
                    <div class="text-emerald-400 font-medium shrink-0">
                      {{ r.backend }}
                    </div>
                  </div>
                </div>
              </div>

              <!-- Labels Section -->
              <KeyValueBadgeList :items="props.ingress.labels" title="Labels" variant="tag" />

              <!-- Annotations Section -->
              <KeyValueBadgeList
                :items="props.ingress.annotations"
                title="Annotations"
                variant="list"
              />
            </TabPanel>

            <!-- RULES PANEL -->
            <TabPanel value="rules" class="space-y-4">
              <h3 class="text-xs font-bold text-muted-color uppercase tracking-wider">
                Ingress Rules Breakdown
              </h3>
              <div
                v-if="parsedRules.length > 0"
                class="border border-(--border) rounded-xl overflow-hidden bg-(--bg-hover)/10 text-xs"
              >
                <table class="w-full text-left border-collapse">
                  <thead>
                    <tr
                      class="bg-(--bg-hover)/40 border-b border-(--border) text-muted-color font-semibold"
                    >
                      <th class="p-3">Host</th>
                      <th class="p-3">Path</th>
                      <th class="p-3">Backend Target</th>
                    </tr>
                  </thead>
                  <tbody class="divide-y divide-(--border)">
                    <tr
                      v-for="(rule, idx) in parsedRules"
                      :key="idx"
                      class="text-muted-color hover:bg-(--bg-hover)/10"
                    >
                      <td class="p-3 font-mono text-violet-400 font-medium">{{ rule.host }}</td>
                      <td class="p-3 font-mono text-primary">{{ rule.path }}</td>
                      <td class="p-3 font-mono text-emerald-400 font-semibold">
                        {{ rule.backend }}
                      </td>
                    </tr>
                  </tbody>
                </table>
              </div>
              <div v-else class="text-xs text-muted-color italic">
                No routing rules configured for this ingress.
              </div>
            </TabPanel>

            <!-- EVENTS PANEL -->
            <TabPanel value="events">
              <WorkloadEventsTab :events="ingressEvents" />
            </TabPanel>

            <!-- YAML PANEL -->
            <TabPanel value="yaml">
              <WorkloadYamlTab
                :displayed-yaml="displayedYaml"
                :is-yaml-loading="isYamlLoading"
                :copied="copied"
                @copy-yaml="copyYaml"
              />
            </TabPanel>
          </TabPanels>
        </Tabs>
      </div>
    </div>
  </Drawer>
</template>
