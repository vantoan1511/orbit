<script setup lang="ts">
import BaseResourceDrawer from '@/components/shared/BaseResourceDrawer.vue'
import KeyValueBadgeList from '@/components/shared/KeyValueBadgeList.vue'
import { KUBERNETES_EVENT_TYPE, KUBERNETES_RESOURCE_KIND } from '@/constants/kubernetes'
import type { EventInfo } from '@/types/kubernetes'
import { getEventTypeSeverity } from '@/utils/severity'
import { Clock } from '@lucide/vue'
import { ref } from 'vue'

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
kind: ${KUBERNETES_RESOURCE_KIND.Event}
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
message: "${e.message}"
source:
  component: ${e.source}
firstTimestamp: "${e.firstSeen}"
lastTimestamp: "${e.lastSeen}"
count: ${e.count}
type: ${e.type}
`
}

const getEventBadgeClass = (type: string) => {
  if (type === KUBERNETES_EVENT_TYPE.Normal) return 'bg-emerald-500'
  if (type === KUBERNETES_EVENT_TYPE.Warning) return 'bg-amber-500'
  return 'bg-rose-500'
}
</script>

<template>
  <BaseResourceDrawer
    v-model:active-tab="activeTab"
    :visible="props.visible"
    :has-resource="!!props.event"
    :title="props.event?.reason ?? ''"
    :kind="props.event?.type ?? ''"
    :kind-severity="props.event ? getEventTypeSeverity(props.event.type) : 'info'"
    :status-badge-class="props.event ? getEventBadgeClass(props.event.type) : 'bg-emerald-500'"
    @update:visible="emit('update:visible', $event)"
  >
    <template #metadata>
      <div
        v-if="props.event"
        class="flex items-center gap-2 text-xs text-muted-color font-mono mt-0.5"
      >
        <span>ns: {{ props.event.namespace }}</span>
        <span class="text-muted-color/60">•</span>
        <span class="flex items-center gap-1">
          <Clock class="w-3 h-3" />
          <span>{{ props.event.lastSeen }}</span>
        </span>
      </div>
    </template>

    <!-- Overview Panel -->
    <template #overview>
      <div v-if="props.event" class="space-y-6">
        <!-- Event Message -->
        <div v-if="props.event.message" class="bg-(--bg-hover)/40 rounded-xl p-4">
          <div class="text-[10px] font-bold text-muted-color uppercase tracking-wider mb-1.5">
            Message
          </div>
          <div class="text-xs text-primary font-mono leading-relaxed break-words">
            {{ props.event.message }}
          </div>
        </div>

        <!-- General Info Grid -->
        <div>
          <h3 class="text-xs font-bold text-muted-color uppercase tracking-wider mb-3">
            Event Properties
          </h3>
          <div class="bg-(--bg-hover)/40 rounded-xl p-4 flex flex-col gap-3 text-xs font-ui">
            <div class="grid grid-cols-3">
              <span class="text-muted-color font-semibold">Namespace</span>
              <span class="col-span-2 font-mono text-primary">{{ props.event.namespace }}</span>
            </div>
            <div class="grid grid-cols-3">
              <span class="text-muted-color font-semibold">Involved Object</span>
              <span class="col-span-2 text-primary">
                <span class="font-semibold text-violet-400">{{ props.event.objectKind }}</span
                >/{{ props.event.objectName }}
              </span>
            </div>
            <div class="grid grid-cols-3">
              <span class="text-muted-color font-semibold">Reason</span>
              <span class="col-span-2 text-primary font-mono">{{ props.event.reason }}</span>
            </div>
            <div class="grid grid-cols-3">
              <span class="text-muted-color font-semibold">Source Component</span>
              <span class="col-span-2 text-primary">{{ props.event.source }}</span>
            </div>
            <div class="grid grid-cols-3">
              <span class="text-muted-color font-semibold">First Timestamp</span>
              <span class="col-span-2 text-primary">{{ props.event.firstSeen }}</span>
            </div>
            <div class="grid grid-cols-3">
              <span class="text-muted-color font-semibold">Last Timestamp</span>
              <span class="col-span-2 text-primary">{{ props.event.lastSeen }}</span>
            </div>
            <div class="grid grid-cols-3">
              <span class="text-muted-color font-semibold">Occurrence Count</span>
              <span class="col-span-2 text-primary font-mono font-bold">{{
                props.event.count
              }}</span>
            </div>
            <div class="grid grid-cols-3">
              <span class="text-muted-color font-semibold">Type</span>
              <span class="col-span-2 text-primary">{{ props.event.type }}</span>
            </div>
            <div class="grid grid-cols-3 items-start gap-4">
              <span class="text-muted-color font-semibold shrink-0">UID</span>
              <span class="col-span-2 font-mono text-[10px] text-primary truncate">{{
                props.event.uid
              }}</span>
            </div>
          </div>
        </div>

        <!-- Labels Section -->
        <KeyValueBadgeList title="Labels" :items="props.event.labels" variant="tag" />
      </div>
    </template>

    <!-- YAML Panel -->
    <template #yaml>
      <div
        v-if="props.event"
        class="flex-1 min-h-64 border border-(--border) rounded-xl bg-zinc-950 p-4 overflow-y-auto"
      >
        <pre class="font-mono text-[10px] text-zinc-300 leading-relaxed">{{
          generateYaml(props.event)
        }}</pre>
      </div>
    </template>
  </BaseResourceDrawer>
</template>
