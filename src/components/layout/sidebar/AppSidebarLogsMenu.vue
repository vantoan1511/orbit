<script setup lang="ts">
import { useKubernetesStore } from '@/stores/kubernetesStore'
import { useLogsStore, type RecentLogInfo } from '@/stores/logsStore'
import { highlightMatch } from '@/utils/text'
import { Box, ChevronDown, ChevronRight, Clock, Folder, Trash2 } from '@lucide/vue'
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

// Expanded state for namespace tree nodes: keys like `ns:default`
const expandedNodes = ref<Record<string, boolean>>({})

const toggleNode = (key: string) => {
  expandedNodes.value[key] = !expandedNodes.value[key]
}

const isNodeExpanded = (key: string) => {
  return !!expandedNodes.value[key]
}

interface TreeWorkload {
  name: string
  kind: string
  namespace: string
}

interface TreeNamespace {
  name: string
  workloads: TreeWorkload[]
}

const treeData = computed<TreeNamespace[]>(() => {
  const query = searchQuery.value.trim().toLowerCase()
  const namespaces = k8sStore.namespaces.filter((n) => n !== 'All Namespaces')

  const result: TreeNamespace[] = []

  for (const ns of namespaces) {
    const allWorkloads: TreeWorkload[] = []

    // Deployments
    for (const d of k8sStore.deployments) {
      if (d.namespace === ns) {
        allWorkloads.push({ name: d.name, kind: 'Deployment', namespace: ns })
      }
    }

    // StatefulSets
    for (const s of k8sStore.statefulSets) {
      if (s.namespace === ns) {
        allWorkloads.push({ name: s.name, kind: 'StatefulSet', namespace: ns })
      }
    }

    // DaemonSets
    for (const d of k8sStore.daemonSets) {
      if (d.namespace === ns) {
        allWorkloads.push({ name: d.name, kind: 'DaemonSet', namespace: ns })
      }
    }

    // ReplicaSets (standalone only)
    const deploymentNames = new Set(
      k8sStore.deployments.filter((d) => d.namespace === ns).map((d) => d.name)
    )
    for (const r of k8sStore.replicaSets) {
      if (
        r.namespace === ns &&
        !Array.from(deploymentNames).some((depName) => r.name.startsWith(depName + '-'))
      ) {
        allWorkloads.push({ name: r.name, kind: 'ReplicaSet', namespace: ns })
      }
    }

    // Jobs
    for (const j of k8sStore.jobs) {
      if (j.namespace === ns) {
        allWorkloads.push({ name: j.name, kind: 'Job', namespace: ns })
      }
    }

    // CronJobs
    for (const c of k8sStore.cronJobs) {
      if (c.namespace === ns) {
        allWorkloads.push({ name: c.name, kind: 'CronJob', namespace: ns })
      }
    }

    // Standalone Pods (pods without controlledBy)
    for (const p of k8sStore.pods) {
      if (p.namespace === ns && (!p.controlledBy || p.controlledBy === '')) {
        allWorkloads.push({ name: p.name, kind: 'Pod', namespace: ns })
      }
    }

    // Sort workloads alphabetically by name
    allWorkloads.sort((a, b) => a.name.localeCompare(b.name))

    const matchingWorkloads: TreeWorkload[] = []

    for (const w of allWorkloads) {
      if (query) {
        const workloadMatches = w.name.toLowerCase().includes(query)
        const nsMatches = ns.toLowerCase().includes(query)
        if (workloadMatches || nsMatches) {
          matchingWorkloads.push(w)
        }
      } else {
        matchingWorkloads.push(w)
      }
    }

    if (!query || matchingWorkloads.length > 0 || ns.toLowerCase().includes(query)) {
      result.push({
        name: ns,
        workloads: matchingWorkloads
      })
    }
  }

  return result
})

const getWorkloadColorClass = (kind: string) => {
  switch (kind) {
    case 'Deployment':
      return 'text-deployment'
    case 'DaemonSet':
      return 'text-daemonset'
    case 'StatefulSet':
      return 'text-statefulset'
    case 'Job':
    case 'CronJob':
      return 'text-job'
    case 'ReplicaSet':
      return 'text-replicaset'
    case 'Pod':
      return 'text-pod'
    default:
      return 'text-primary'
  }
}

const isCurrentLogActive = (ns: string, workload: string, kind?: string) => {
  if (route.path !== '/logs') return false
  const qNs = route.query.namespace as string
  const qWorkload = route.query.workload as string
  const qKind = route.query.kind as string

  if (qNs !== ns) return false
  if (qWorkload !== workload) return false
  if (kind && qKind && qKind !== kind) return false
  return true
}

const selectLogTarget = (ns: string, workload: string, kind: string) => {
  router.push({
    path: '/logs',
    query: {
      namespace: ns,
      kind,
      workload,
      pod: 'All',
      container: 'All'
    }
  })
}

const selectRecentLog = (log: RecentLogInfo) => {
  router.push({
    path: '/logs',
    query: {
      namespace: log.namespace,
      kind: log.workloadKind,
      workload: log.workloadName,
      pod: log.pod || 'All',
      container: log.container || 'All'
    }
  })
}
</script>

<template>
  <div class="flex-1 overflow-hidden flex flex-col text-sm select-none">
    <div v-if="k8sStore.activeClusterId !== null" class="flex flex-col h-full min-h-0">
      <Accordion
        :value="activeAccordion"
        multiple
        class="flex-1 flex flex-col min-h-0 !gap-0"
        :dt="{
          panel: { border: { width: '0', color: 'transparent' }, borderRadius: '0' },
          header: {
            border: { width: '0', color: 'transparent' },
            borderRadius: '0',
            padding: '0',
            background: 'transparent',
            hoverBackground: 'transparent',
            activeBackground: 'transparent',
            activeHoverBackground: 'transparent'
          },
          content: { border: { width: '0', color: 'transparent' }, borderRadius: '0', padding: '0' }
        }"
      >
        <!-- Explorer Accordion -->
        <AccordionPanel
          value="0"
          class="!border-none !rounded-none flex flex-col min-h-0 transition-[flex]"
          :class="[activeAccordion.includes('0') ? 'flex-1' : 'shrink-0']"
        >
          <AccordionHeader
            class="h-8 py-1.5 px-3 text-xs font-semibold uppercase tracking-wider text-muted-color hover:text-primary flex items-center justify-between shrink-0 !rounded-none border border-transparent hover:border-(--border) cursor-pointer !bg-transparent hover:!bg-transparent focus:!bg-transparent active:!bg-transparent"
          >
            <span>Explorer</span>
          </AccordionHeader>
          <AccordionContent
            class="p-0 flex-1 flex flex-col min-h-0"
            :pt="{ content: 'p-0 flex flex-col h-full min-h-0' }"
          >
            <!-- Filter Input -->
            <div class="p-1.5 shrink-0">
              <InputText
                v-model="searchQuery"
                placeholder="Filter logs..."
                size="small"
                class="w-full text-sm"
              />
            </div>

            <!-- Tree View -->
            <div class="py-0.5 overflow-x-auto overflow-y-auto flex-1 min-h-0">
              <div class="w-max min-w-full">
                <div v-if="treeData.length === 0" class="text-sm text-muted-color px-3 py-1">
                  No logs found
                </div>

                <div v-for="ns in treeData" :key="`ns:${ns.name}`" class="flex flex-col">
                  <!-- Namespace Row -->
                  <div
                    class="flex items-center gap-1.5 px-3 py-0.5 border border-transparent hover:border-(--border) cursor-pointer text-sm font-medium text-primary rounded-sm"
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
                    <span class="ml-auto text-xs text-muted-color"
                      >({{ ns.workloads.length }})</span
                    >
                  </div>

                  <!-- Namespace Workloads -->
                  <div
                    v-if="isNodeExpanded(`ns:${ns.name}`) || searchQuery"
                    class="flex flex-col pl-4"
                  >
                    <div
                      v-for="workload in ns.workloads"
                      :key="`workload:${ns.name}/${workload.kind}/${workload.name}`"
                      :class="[
                        isCurrentLogActive(ns.name, workload.name, workload.kind)
                          ? 'bg-primary-50 dark:bg-primary-950/40 text-primary font-semibold border-primary/30'
                          : 'text-muted-color hover:text-primary hover:border-(--border)',
                        'flex items-center gap-1.5 px-3 py-0.5 cursor-pointer text-sm border border-transparent rounded-sm'
                      ]"
                      @click="selectLogTarget(ns.name, workload.name, workload.kind)"
                    >
                      <Box
                        class="w-3.5 h-3.5 shrink-0"
                        :class="getWorkloadColorClass(workload.kind)"
                      />
                      <span
                        class="truncate flex-1"
                        v-html="highlightMatch(workload.name, searchQuery)"
                      >
                      </span>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </AccordionContent>
        </AccordionPanel>

        <!-- Recent Accordion -->
        <AccordionPanel
          value="1"
          class="!border-none !rounded-none flex flex-col min-h-0 border-t border-(--border) transition-[flex]"
          :class="[activeAccordion.includes('1') ? 'flex-1' : 'shrink-0']"
        >
          <AccordionHeader
            as="div"
            class="h-8 py-1.5 px-3 text-xs font-semibold uppercase tracking-wider text-muted-color hover:text-primary flex justify-between items-center shrink-0 !rounded-none border border-transparent hover:border-(--border) cursor-pointer !bg-transparent hover:!bg-transparent focus:!bg-transparent active:!bg-transparent"
          >
            <div class="flex items-center justify-between flex-1 mr-2">
              <span>Recent</span>
              <Button
                v-if="logsStore.recentLogs.length > 0"
                severity="secondary"
                variant="text"
                size="small"
                class="w-5! h-5! p-0! shrink-0 text-muted-color hover:text-primary"
                title="Clear recent logs"
                aria-label="Clear recent logs"
                @click.stop="logsStore.clearRecentLogs()"
              >
                <template #icon>
                  <Trash2 class="w-3.5 h-3.5" />
                </template>
              </Button>
            </div>
          </AccordionHeader>
          <AccordionContent
            class="p-0 flex-1 flex flex-col min-h-0"
            :pt="{ content: 'p-0 flex flex-col h-full min-h-0' }"
          >
            <div class="py-0.5 overflow-x-auto overflow-y-auto flex-1 min-h-0">
              <div class="w-max min-w-full">
                <div
                  v-if="logsStore.recentLogs.length === 0"
                  class="text-sm text-muted-color px-3 py-1"
                >
                  No recent logs
                </div>

                <div
                  v-for="log in logsStore.recentLogs"
                  :key="`recent:${log.namespace}/${log.workloadKind}/${log.workloadName}`"
                  :class="[
                    isCurrentLogActive(log.namespace, log.workloadName, log.workloadKind)
                      ? 'bg-primary-50 dark:bg-primary-950/40 text-primary font-semibold border-primary/30'
                      : 'text-muted-color hover:text-primary hover:border-(--border)',
                    'flex items-center gap-2 px-3 py-0.5 cursor-pointer text-sm group border border-transparent rounded-sm'
                  ]"
                  @click="selectRecentLog(log)"
                >
                  <Clock class="w-3.5 h-3.5 shrink-0 text-muted-color" />
                  <div class="flex flex-col truncate flex-1 min-w-0">
                    <span class="truncate font-medium">{{ log.workloadName }}</span>
                    <span class="text-xs text-muted-color truncate">
                      {{ log.namespace }} &bull; {{ log.workloadKind }}
                    </span>
                  </div>
                </div>
              </div>
            </div>
          </AccordionContent>
        </AccordionPanel>
      </Accordion>
    </div>
    <p v-else class="text-sm text-muted-color p-2">Select or add a cluster to view logs.</p>
  </div>
</template>
