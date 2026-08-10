<script setup lang="ts">
import { useKubernetesStore } from '@/stores/kubernetesStore'
import { useLogsStore, type RecentLogInfo } from '@/stores/logsStore'
import { Box, ChevronDown, ChevronRight, Clock, Folder, Layers, Trash2 } from '@lucide/vue'
import Accordion from 'primevue/accordion'
import AccordionContent from 'primevue/accordioncontent'
import AccordionHeader from 'primevue/accordionheader'
import AccordionPanel from 'primevue/accordionpanel'
import Button from 'primevue/button'
import InputText from 'primevue/inputtext'
import { computed, onMounted, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'

const k8sStore = useKubernetesStore()
const logsStore = useLogsStore()
const route = useRoute()
const router = useRouter()

const activeAccordion = ref<string[]>(['0', '1'])
const searchQuery = ref('')

onMounted(() => {
  logsStore.loadRecentLogs()
})

// Expanded state for tree nodes: keys like `ns:default`, `pod:default/my-pod`
const expandedNodes = ref<Record<string, boolean>>({})

const toggleNode = (key: string) => {
  expandedNodes.value[key] = !expandedNodes.value[key]
}

const isNodeExpanded = (key: string) => {
  return !!expandedNodes.value[key]
}

interface TreeContainer {
  name: string
}

interface TreePod {
  name: string
  namespace: string
  containers: TreeContainer[]
}

interface TreeNamespace {
  name: string
  pods: TreePod[]
}

const treeData = computed<TreeNamespace[]>(() => {
  const query = searchQuery.value.trim().toLowerCase()
  const namespaces = k8sStore.namespaces.filter((n) => n !== 'All Namespaces')

  const result: TreeNamespace[] = []

  for (const ns of namespaces) {
    const nsPods = k8sStore.pods.filter((p) => p.namespace === ns)
    const matchingPods: TreePod[] = []

    for (const pod of nsPods) {
      const containers = (pod.containers || []).map((c) => ({ name: c.name }))

      // Filter check
      if (query) {
        const podMatches = pod.name.toLowerCase().includes(query)
        const nsMatches = ns.toLowerCase().includes(query)
        const containerMatches = containers.some((c) => c.name.toLowerCase().includes(query))

        if (podMatches || nsMatches || containerMatches) {
          const filteredContainers =
            containerMatches && !podMatches && !nsMatches
              ? containers.filter((c) => c.name.toLowerCase().includes(query))
              : containers
          matchingPods.push({
            name: pod.name,
            namespace: ns,
            containers: filteredContainers
          })
        }
      } else {
        matchingPods.push({
          name: pod.name,
          namespace: ns,
          containers
        })
      }
    }

    if (!query || matchingPods.length > 0 || ns.toLowerCase().includes(query)) {
      result.push({
        name: ns,
        pods: matchingPods
      })
    }
  }

  return result
})

const isCurrentLogActive = (ns: string, pod: string, container?: string) => {
  if (route.path !== '/logs') return false
  const qNs = route.query.namespace as string
  const qPod = route.query.pod as string
  const qContainer = route.query.container as string

  if (qNs !== ns) return false
  if (qPod !== pod) return false
  if (container && qContainer && qContainer !== 'All') {
    return qContainer === container
  }
  return true
}

const selectLogTarget = (ns: string, pod: string, container: string = 'All') => {
  const podObj = k8sStore.pods.find((p) => p.name === pod && p.namespace === ns)
  let kind = 'Pod'
  let workload = pod

  if (podObj && podObj.controlledBy) {
    const parts = podObj.controlledBy.split('/')
    if (parts.length === 2 && parts[0] && parts[1]) {
      kind = parts[0]
      workload = parts[1]
    } else if (podObj.controlledBy) {
      workload = podObj.controlledBy
    }
  }

  router.push({
    path: '/logs',
    query: {
      namespace: ns,
      kind,
      workload,
      pod,
      container
    }
  })
}

const selectRecentLog = (log: RecentLogInfo) => {
  selectLogTarget(log.namespace, log.pod, log.container)
}

import { highlightMatch } from '@/utils/text'
</script>

<template>
  <div class="flex-1 overflow-y-auto flex flex-col text-sm select-none">
    <div v-if="k8sStore.activeClusterId !== null" class="flex flex-col h-full">
      <Accordion
        :value="activeAccordion"
        multiple
        class="flex-1 flex flex-col"
        :dt="{ panel: { border: { width: '0' } } }"
      >
        <!-- Explorer Accordion -->
        <AccordionPanel value="0" class="border-none">
          <AccordionHeader
            class="py-1.5 px-3 text-xs font-semibold uppercase tracking-wider text-muted-color"
          >
            Explorer
          </AccordionHeader>
          <AccordionContent class="p-0">
            <!-- Filter Input -->
            <div class="p-1.5">
              <InputText
                v-model="searchQuery"
                placeholder="Filter logs..."
                size="small"
                class="w-full text-sm"
              />
            </div>

            <!-- Tree View -->
            <div class="py-0.5">
              <div v-if="treeData.length === 0" class="text-sm text-muted-color px-3 py-1">
                No logs found
              </div>

              <div v-for="ns in treeData" :key="`ns:${ns.name}`" class="flex flex-col">
                <!-- Namespace Row -->
                <div
                  class="flex items-center gap-1.5 px-3 py-0.5 hover:bg-surface-100 dark:hover:bg-surface-800 cursor-pointer text-sm font-medium text-surface-700 dark:text-surface-300"
                  @click="toggleNode(`ns:${ns.name}`)"
                >
                  <component
                    :is="
                      isNodeExpanded(`ns:${ns.name}`) || searchQuery ? ChevronDown : ChevronRight
                    "
                    class="w-3.5 h-3.5 shrink-0 text-muted-color"
                  />
                  <Folder class="w-3.5 h-3.5 shrink-0 text-amber-500" />
                  <span class="truncate" v-html="highlightMatch(ns.name, searchQuery)"></span>
                  <span class="ml-auto text-xs text-muted-color">({{ ns.pods.length }})</span>
                </div>

                <!-- Namespace Pods -->
                <div
                  v-if="isNodeExpanded(`ns:${ns.name}`) || searchQuery"
                  class="flex flex-col pl-4"
                >
                  <div
                    v-for="pod in ns.pods"
                    :key="`pod:${ns.name}/${pod.name}`"
                    class="flex flex-col"
                  >
                    <!-- Pod Row -->
                    <div
                      :class="[
                        isCurrentLogActive(ns.name, pod.name)
                          ? 'bg-primary-50 dark:bg-primary-950/40 text-primary font-semibold'
                          : 'text-surface-600 dark:text-surface-400 hover:bg-surface-100 dark:hover:bg-surface-800',
                        'flex items-center gap-1.5 px-3 py-0.5 cursor-pointer text-sm'
                      ]"
                      @click="toggleNode(`pod:${ns.name}/${pod.name}`)"
                    >
                      <component
                        :is="
                          isNodeExpanded(`pod:${ns.name}/${pod.name}`) || searchQuery
                            ? ChevronDown
                            : ChevronRight
                        "
                        class="w-3.5 h-3.5 shrink-0 text-muted-color"
                      />
                      <Box class="w-3.5 h-3.5 shrink-0 text-blue-500" />
                      <span
                        class="truncate flex-1"
                        @click.stop="selectLogTarget(ns.name, pod.name)"
                        v-html="highlightMatch(pod.name, searchQuery)"
                      >
                      </span>
                    </div>

                    <!-- Pod Containers -->
                    <div
                      v-if="isNodeExpanded(`pod:${ns.name}/${pod.name}`) || searchQuery"
                      class="flex flex-col pl-4"
                    >
                      <div
                        v-for="c in pod.containers"
                        :key="`container:${ns.name}/${pod.name}/${c.name}`"
                        :class="[
                          isCurrentLogActive(ns.name, pod.name, c.name)
                            ? 'bg-primary-50 dark:bg-primary-950/40 text-primary font-semibold'
                            : 'text-surface-500 dark:text-surface-400 hover:bg-surface-100 dark:hover:bg-surface-800',
                          'flex items-center gap-1.5 px-3 py-0.5 cursor-pointer text-sm'
                        ]"
                        @click="selectLogTarget(ns.name, pod.name, c.name)"
                      >
                        <Layers class="w-3.5 h-3.5 shrink-0 text-emerald-500" />
                        <span class="truncate" v-html="highlightMatch(c.name, searchQuery)"></span>
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </AccordionContent>
        </AccordionPanel>

        <!-- Recent Accordion -->
        <AccordionPanel value="1" class="border-none">
          <AccordionHeader
            class="py-1.5 px-3 text-xs font-semibold uppercase tracking-wider text-muted-color flex justify-between items-center"
          >
            <span>Recent</span>
          </AccordionHeader>
          <AccordionContent class="p-0">
            <div class="py-0.5">
              <div
                v-if="logsStore.recentLogs.length === 0"
                class="text-sm text-muted-color px-3 py-1"
              >
                No recent logs
              </div>

              <div
                v-for="log in logsStore.recentLogs"
                :key="`recent:${log.namespace}/${log.pod}/${log.container}`"
                :class="[
                  isCurrentLogActive(log.namespace, log.pod, log.container)
                    ? 'bg-primary-50 dark:bg-primary-950/40 text-primary font-semibold'
                    : 'text-surface-600 dark:text-surface-400 hover:bg-surface-100 dark:hover:bg-surface-800',
                  'flex items-center gap-2 px-3 py-0.5 cursor-pointer text-sm group'
                ]"
                @click="selectRecentLog(log)"
              >
                <Clock class="w-3.5 h-3.5 shrink-0 text-muted-color" />
                <div class="flex flex-col truncate flex-1 min-w-0">
                  <span class="truncate font-medium">{{ log.pod }}</span>
                  <span class="text-xs text-muted-color truncate">
                    {{ log.namespace }} &bull; {{ log.container }}
                  </span>
                </div>
              </div>

              <div v-if="logsStore.recentLogs.length > 0" class="p-1.5 flex justify-end">
                <Button
                  severity="secondary"
                  variant="text"
                  size="small"
                  class="text-xs h-6"
                  @click="logsStore.clearRecentLogs()"
                >
                  <Trash2 class="w-3 h-3 mr-1" />
                  Clear Recent
                </Button>
              </div>
            </div>
          </AccordionContent>
        </AccordionPanel>
      </Accordion>
    </div>
    <p v-else class="text-sm text-muted-color p-2">Select or add a cluster to view logs.</p>
  </div>
</template>
