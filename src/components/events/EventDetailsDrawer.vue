<script setup lang="ts">
import { ref } from 'vue'
import Drawer from 'primevue/drawer'
import Tabs from 'primevue/tabs'
import TabList from 'primevue/tablist'
import Tab from 'primevue/tab'
import TabPanels from 'primevue/tabpanels'
import TabPanel from 'primevue/tabpanel'
import { Clock, Tag as TagIcon, Server, FileCode } from '@lucide/vue'
import type { EventInfo } from '@/types/kubernetes'

const props = defineProps<{
  visible: boolean
  event: EventInfo | null
}>()

const emit = defineEmits<{
  (e: 'update:visible', value: boolean): void
}>()

const activeTab = ref('overview')

const generateYaml = (e: EventInfo) => {
  return `apiVersion: v1
kind: Event
metadata:
  name: ${e.objectName}.${e.uid.substring(0, 8)}
  namespace: ${e.namespace}
  uid: ${e.uid}
  creationTimestamp: "${e.firstSeen}"
  labels:
${Object.entries(e.labels)
  .map(([k, v]) => `    ${k}: ${v}`)
  .join('\n')}
involvedObject:
  kind: ${e.objectKind}
  name: ${e.objectName}
  namespace: ${e.namespace}
reason: ${e.reason}
message: ${e.message}
source:
  component: ${e.source}
firstTimestamp: "${e.firstSeen}"
lastTimestamp: "${e.lastSeen}"
count: ${e.count}
type: ${e.type}
`
}

const getTypeSeverity = (type: string) => {
  switch (type) {
    case 'Warning':
      return 'warn'
    case 'Error':
      return 'danger'
    case 'Normal':
      return 'success'
    default:
      return 'secondary'
  }
}
</script>

<template>
  <Drawer
    :visible="props.visible"
    @update:visible="emit('update:visible', $event)"
    position="right"
    class="w-full sm:max-w-lg border-l border-(--border) bg-(--bg-card) p-0"
    :header="props.event?.reason || 'Event Details'"
    :style="{ width: '36rem' }"
  >
    <template #header>
      <div class="flex items-center gap-3 w-full" v-if="props.event">
        <div class="flex items-center gap-1.5">
          <span
            class="w-2.5 h-2.5 rounded-full"
            :class="
              props.event.type === 'Normal'
                ? 'bg-emerald-500'
                : props.event.type === 'Warning'
                  ? 'bg-amber-500'
                  : 'bg-red-500'
            "
          ></span>
          <span class="text-xs font-bold uppercase tracking-wider text-muted-color">
            {{ props.event.type }}
          </span>
        </div>
        <Tag severity="secondary" class="font-mono" :value="`ns/${props.event.namespace}`" />
        <Tag :severity="getTypeSeverity(props.event.type)" :value="props.event.reason" />
      </div>
    </template>

    <div v-if="props.event" class="h-full flex flex-col">
      <!-- Title/Message Section -->
      <div class="p-6 border-b border-(--border) bg-(--bg-hover)/50">
        <h2
          class="text-sm font-semibold text-primary font-ui mb-2 break-words"
          :title="props.event.message"
        >
          {{ props.event.message }}
        </h2>
        <div class="text-xs text-muted-color flex items-center gap-2">
          <Clock class="w-3.5 h-3.5" />
          <span>Last Seen: {{ props.event.lastSeen }}</span>
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
                <h3 class="text-xs font-bold text-muted-color uppercase tracking-wider">
                  Event Properties
                </h3>
                <div
                  class="border border-(--border) rounded-xl overflow-hidden divide-y divide-(--border) bg-(--bg-hover)/10 text-xs"
                >
                  <div class="grid grid-cols-3 p-3">
                    <span class="text-muted-color font-semibold">Namespace</span>
                    <span class="col-span-2 font-mono text-primary">{{
                      props.event.namespace
                    }}</span>
                  </div>
                  <div class="grid grid-cols-3 p-3">
                    <span class="text-muted-color font-semibold">Involved Object</span>
                    <span class="col-span-2 text-primary">
                      <span class="font-semibold text-violet-400">{{ props.event.objectKind }}</span
                      >/{{ props.event.objectName }}
                    </span>
                  </div>
                  <div class="grid grid-cols-3 p-3">
                    <span class="text-muted-color font-semibold">Reason</span>
                    <span class="col-span-2 text-primary font-mono">{{ props.event.reason }}</span>
                  </div>
                  <div class="grid grid-cols-3 p-3">
                    <span class="text-muted-color font-semibold">Source Component</span>
                    <span class="col-span-2 text-primary">{{ props.event.source }}</span>
                  </div>
                  <div class="grid grid-cols-3 p-3">
                    <span class="text-muted-color font-semibold">First Timestamp</span>
                    <span class="col-span-2 text-primary">{{ props.event.firstSeen }}</span>
                  </div>
                  <div class="grid grid-cols-3 p-3">
                    <span class="text-muted-color font-semibold">Last Timestamp</span>
                    <span class="col-span-2 text-primary">{{ props.event.lastSeen }}</span>
                  </div>
                  <div class="grid grid-cols-3 p-3">
                    <span class="text-muted-color font-semibold">Occurrence Count</span>
                    <span class="col-span-2 text-primary font-mono font-bold">{{
                      props.event.count
                    }}</span>
                  </div>
                  <div class="grid grid-cols-3 p-3">
                    <span class="text-muted-color font-semibold">Type</span>
                    <span class="col-span-2 text-primary">{{ props.event.type }}</span>
                  </div>
                  <div class="grid grid-cols-3 p-3">
                    <span class="text-muted-color font-semibold">UID</span>
                    <span class="col-span-2 font-mono text-[10px] text-primary">{{
                      props.event.uid
                    }}</span>
                  </div>
                </div>
              </div>

              <!-- Labels Section -->
              <div class="space-y-3" v-if="Object.keys(props.event.labels).length > 0">
                <h3
                  class="text-xs font-bold text-muted-color uppercase tracking-wider flex items-center gap-1.5"
                >
                  <TagIcon class="w-3.5 h-3.5" />
                  <span>Labels</span>
                </h3>
                <div class="flex flex-wrap gap-2">
                  <div
                    v-for="(val, key) in props.event.labels"
                    :key="key"
                    class="flex items-center text-xs border border-(--border) bg-(--bg-hover)/20 rounded-lg overflow-hidden"
                  >
                    <span
                      class="px-2 py-1 bg-(--bg-hover)/40 border-r border-(--border) text-muted-color font-medium"
                      >{{ key }}</span
                    >
                    <span class="px-2 py-1 font-mono text-primary">{{ val }}</span>
                  </div>
                </div>
              </div>
            </TabPanel>

            <!-- YAML PANEL -->
            <TabPanel value="yaml" class="h-full flex flex-col gap-2">
              <div
                class="flex-1 min-h-64 border border-(--border) rounded-xl bg-zinc-950 p-4 overflow-y-auto"
              >
                <pre class="font-mono text-[10px] text-zinc-300 leading-relaxed">{{
                  generateYaml(props.event)
                }}</pre>
              </div>
            </TabPanel>
          </TabPanels>
        </Tabs>
      </div>
    </div>
  </Drawer>
</template>
