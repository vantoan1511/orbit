<script setup lang="ts">
import KeyValueBadgeList from '@/components/shared/KeyValueBadgeList.vue'
import ReactiveAge from '@/components/shared/ReactiveAge.vue'
import type { NodeInfo } from '@/types/kubernetes'
import { computed } from 'vue'

const props = defineProps<{
  node: NodeInfo
}>()

const podsPct = computed(() => {
  if (!props.node.podsLimit || props.node.podsLimit === 0) return 0
  return Math.round((props.node.podsCount / props.node.podsLimit) * 100)
})

const getConditionSeverity = (type: string, status: string) => {
  if (type === 'Ready') {
    return status === 'True' ? 'success' : 'danger'
  }
  // For Pressure conditions (MemoryPressure, DiskPressure, PIDPressure), True is bad
  return status === 'True' ? 'danger' : 'success'
}

const getTaintEffectSeverity = (effect: string) => {
  if (effect === 'NoExecute') return 'danger'
  if (effect === 'NoSchedule') return 'warn'
  return 'info'
}
</script>

<template>
  <div class="flex flex-col gap-8">
    <!-- Resource Metrics Grid -->
    <div class="grid grid-cols-3 gap-3">
      <!-- CPU -->
      <div class="p-4 rounded-xl bg-(--bg-hover)/40">
        <div class="text-[10px] font-semibold uppercase tracking-wider text-muted-color mb-1.5">
          CPU Requests
        </div>
        <div class="text-base font-bold font-mono text-primary flex items-baseline justify-between">
          <span>{{ node.cpuPct }}%</span>
          <span class="text-xs text-muted-color font-normal">
            {{ node.cpuUsed }} / {{ node.cpuTotal }} Cores
          </span>
        </div>
        <div class="w-full h-1.5 rounded-full bg-(--bg-hover) overflow-hidden mt-2.5">
          <div
            class="h-full bg-blue-500 rounded-full"
            :style="{ width: Math.min(node.cpuPct, 100) + '%' }"
          ></div>
        </div>
      </div>

      <!-- Memory -->
      <div class="p-4 rounded-xl bg-(--bg-hover)/40">
        <div class="text-[10px] font-semibold uppercase tracking-wider text-muted-color mb-1.5">
          Memory Requests
        </div>
        <div class="text-base font-bold font-mono text-primary flex items-baseline justify-between">
          <span>{{ node.memPct }}%</span>
          <span class="text-xs text-muted-color font-normal">
            {{ node.memUsed }} / {{ node.memTotal }} GiB
          </span>
        </div>
        <div class="w-full h-1.5 rounded-full bg-(--bg-hover) overflow-hidden mt-2.5">
          <div
            class="h-full bg-indigo-500 rounded-full"
            :style="{ width: Math.min(node.memPct, 100) + '%' }"
          ></div>
        </div>
      </div>

      <!-- Pods -->
      <div class="p-4 rounded-xl bg-(--bg-hover)/40">
        <div class="text-[10px] font-semibold uppercase tracking-wider text-muted-color mb-1.5">
          Allocated Pods
        </div>
        <div class="text-base font-bold font-mono text-primary flex items-baseline justify-between">
          <span>{{ podsPct }}%</span>
          <span class="text-xs text-muted-color font-normal">
            {{ node.podsCount }} / {{ node.podsLimit }}
          </span>
        </div>
        <div class="w-full h-1.5 rounded-full bg-(--bg-hover) overflow-hidden mt-2.5">
          <div
            class="h-full bg-sky-500 rounded-full"
            :style="{ width: Math.min(podsPct, 100) + '%' }"
          ></div>
        </div>
      </div>
    </div>

    <!-- Node Attributes & Addresses -->
    <div>
      <div class="text-xs font-semibold text-muted-color uppercase tracking-wider mb-3">
        Node Attributes
      </div>
      <div class="grid grid-cols-2 gap-4 p-4 rounded-xl bg-(--bg-hover)/40 text-xs">
        <div>
          <span class="text-muted-color">Role: </span>
          <span class="font-semibold text-primary uppercase font-mono">{{ node.role }}</span>
        </div>
        <div>
          <span class="text-muted-color">Status: </span>
          <span
            class="font-semibold"
            :class="node.status === 'Ready' ? 'text-emerald-400' : 'text-rose-400'"
          >
            {{ node.status }}
          </span>
        </div>
        <div>
          <span class="text-muted-color">Kubelet Version: </span>
          <span class="font-semibold text-primary font-mono">{{ node.version }}</span>
        </div>
        <div>
          <span class="text-muted-color">Unschedulable / Cordoned: </span>
          <span
            class="font-semibold font-mono"
            :class="node.isCordoned ? 'text-amber-400' : 'text-primary'"
          >
            {{ node.isCordoned ? 'Yes (SchedulingDisabled)' : 'No' }}
          </span>
        </div>
        <div>
          <span class="text-muted-color">Uptime: </span>
          <span class="font-semibold text-primary font-mono">
            <ReactiveAge :age="node.uptime" />
          </span>
        </div>
        <div>
          <span class="text-muted-color">Images Count: </span>
          <span class="font-semibold text-primary font-mono">{{ node.imagesCount ?? '-' }}</span>
        </div>

        <!-- Node Addresses -->
        <div
          v-if="node.addresses && node.addresses.length > 0"
          class="col-span-2 pt-3 border-t border-(--border)/30 mt-1"
        >
          <div class="text-[10px] font-semibold text-muted-color uppercase tracking-wider mb-2">
            Addresses
          </div>
          <div class="flex flex-wrap gap-2.5">
            <div
              v-for="addr in node.addresses"
              :key="addr.type + addr.address"
              class="flex items-center gap-1.5 px-2.5 py-1 rounded bg-(--bg-hover)/80"
            >
              <span class="text-muted-color text-[11px]">{{ addr.type }}:</span>
              <span class="font-mono font-medium text-primary text-[11px]">{{ addr.address }}</span>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- System Information -->
    <div v-if="node.nodeInfo">
      <div class="text-xs font-semibold text-muted-color uppercase tracking-wider mb-3">
        System Information
      </div>
      <div class="grid grid-cols-2 gap-4 p-4 rounded-xl bg-(--bg-hover)/40 text-xs">
        <div>
          <span class="text-muted-color">OS Image: </span>
          <span class="font-medium text-primary font-mono">{{ node.nodeInfo.osImage || '-' }}</span>
        </div>
        <div>
          <span class="text-muted-color">OS / Architecture: </span>
          <span class="font-medium text-primary font-mono">
            {{ node.nodeInfo.operatingSystem }} / {{ node.nodeInfo.architecture }}
          </span>
        </div>
        <div>
          <span class="text-muted-color">Kernel Version: </span>
          <span class="font-medium text-primary font-mono">{{
            node.nodeInfo.kernelVersion || '-'
          }}</span>
        </div>
        <div>
          <span class="text-muted-color">Container Runtime: </span>
          <span class="font-medium text-primary font-mono">{{
            node.nodeInfo.containerRuntimeVersion || '-'
          }}</span>
        </div>
        <div>
          <span class="text-muted-color">Kubelet Version: </span>
          <span class="font-medium text-primary font-mono">{{
            node.nodeInfo.kubeletVersion || '-'
          }}</span>
        </div>
        <div>
          <span class="text-muted-color">Kube-Proxy Version: </span>
          <span class="font-medium text-primary font-mono">{{
            node.nodeInfo.kubeProxyVersion || '-'
          }}</span>
        </div>
        <div v-if="node.nodeInfo.machineID">
          <span class="text-muted-color">Machine ID: </span>
          <span
            class="font-medium text-primary font-mono truncate max-w-48 inline-block align-bottom"
            :title="node.nodeInfo.machineID"
          >
            {{ node.nodeInfo.machineID }}
          </span>
        </div>
        <div v-if="node.nodeInfo.systemUUID">
          <span class="text-muted-color">System UUID: </span>
          <span
            class="font-medium text-primary font-mono truncate max-w-48 inline-block align-bottom"
            :title="node.nodeInfo.systemUUID"
          >
            {{ node.nodeInfo.systemUUID }}
          </span>
        </div>
      </div>
    </div>

    <!-- Capacity & Allocatable -->
    <div v-if="node.capacity || node.allocatable">
      <div class="text-xs font-semibold text-muted-color uppercase tracking-wider mb-3">
        Capacity & Allocatable
      </div>
      <div class="border border-(--border) rounded-lg overflow-hidden bg-(--bg-card) text-xs">
        <table class="w-full text-left border-collapse">
          <thead>
            <tr
              class="bg-(--bg-hover)/60 border-b border-(--border) text-[10px] text-muted-color uppercase tracking-wider font-semibold"
            >
              <th class="p-2.5 pl-4">Resource</th>
              <th class="p-2.5">Capacity</th>
              <th class="p-2.5 pr-4">Allocatable</th>
            </tr>
          </thead>
          <tbody class="divide-y divide-(--border) font-mono">
            <tr class="hover:bg-(--bg-hover) transition-colors">
              <td class="p-2.5 pl-4 text-muted-color font-sans font-medium">CPU</td>
              <td class="p-2.5 text-primary">{{ node.capacity?.cpu || '-' }}</td>
              <td class="p-2.5 pr-4 text-primary">{{ node.allocatable?.cpu || '-' }}</td>
            </tr>
            <tr class="hover:bg-(--bg-hover) transition-colors">
              <td class="p-2.5 pl-4 text-muted-color font-sans font-medium">Memory</td>
              <td class="p-2.5 text-primary">{{ node.capacity?.memory || '-' }}</td>
              <td class="p-2.5 pr-4 text-primary">{{ node.allocatable?.memory || '-' }}</td>
            </tr>
            <tr class="hover:bg-(--bg-hover) transition-colors">
              <td class="p-2.5 pl-4 text-muted-color font-sans font-medium">Pods</td>
              <td class="p-2.5 text-primary">{{ node.capacity?.pods || '-' }}</td>
              <td class="p-2.5 pr-4 text-primary">{{ node.allocatable?.pods || '-' }}</td>
            </tr>
            <tr
              v-if="node.capacity?.ephemeralStorage || node.allocatable?.ephemeralStorage"
              class="hover:bg-(--bg-hover) transition-colors"
            >
              <td class="p-2.5 pl-4 text-muted-color font-sans font-medium">Ephemeral Storage</td>
              <td class="p-2.5 text-primary">{{ node.capacity?.ephemeralStorage || '-' }}</td>
              <td class="p-2.5 pr-4 text-primary">
                {{ node.allocatable?.ephemeralStorage || '-' }}
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>

    <!-- Node Conditions -->
    <div v-if="node.conditions && node.conditions.length > 0">
      <div class="text-xs font-semibold text-muted-color uppercase tracking-wider mb-3">
        Conditions ({{ node.conditions.length }})
      </div>
      <div class="border border-(--border) rounded-lg overflow-hidden bg-(--bg-card) text-xs">
        <table class="w-full text-left border-collapse">
          <thead>
            <tr
              class="bg-(--bg-hover)/60 border-b border-(--border) text-[10px] text-muted-color uppercase tracking-wider font-semibold"
            >
              <th class="p-2.5 pl-4">Type</th>
              <th class="p-2.5">Status</th>
              <th class="p-2.5">Reason</th>
              <th class="p-2.5 pr-4">Message</th>
            </tr>
          </thead>
          <tbody class="divide-y divide-(--border)">
            <tr
              v-for="cond in node.conditions"
              :key="cond.type"
              class="hover:bg-(--bg-hover) transition-colors"
            >
              <td class="p-2.5 pl-4 font-semibold text-primary font-mono">{{ cond.type }}</td>
              <td class="p-2.5">
                <Tag
                  rounded
                  class="font-mono whitespace-nowrap"
                  :severity="getConditionSeverity(cond.type, cond.status)"
                  :value="cond.status"
                />
              </td>
              <td class="p-2.5 text-muted-color font-mono text-[11px]">{{ cond.reason || '-' }}</td>
              <td
                class="p-2.5 pr-4 text-muted-color text-[11px] max-w-64 truncate"
                :title="cond.message || ''"
              >
                {{ cond.message || '-' }}
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>

    <!-- Node Taints -->
    <div v-if="node.taints && node.taints.length > 0">
      <div class="text-xs font-semibold text-muted-color uppercase tracking-wider mb-3">
        Taints ({{ node.taints.length }})
      </div>
      <div class="bg-(--bg-hover)/40 rounded-xl p-3 flex flex-col gap-2">
        <div
          v-for="(taint, idx) in node.taints"
          :key="idx"
          class="flex items-center justify-between p-2.5 rounded-lg bg-(--bg-hover)/60 text-xs font-mono"
        >
          <div class="flex items-center gap-2 min-w-0">
            <span class="text-primary font-semibold">{{ taint.key }}</span>
            <span v-if="taint.value" class="text-muted-color">= {{ taint.value }}</span>
          </div>
          <Tag
            rounded
            class="whitespace-nowrap"
            :severity="getTaintEffectSeverity(taint.effect)"
            :value="taint.effect"
          />
        </div>
      </div>
    </div>

    <!-- Labels & Annotations -->
    <div class="space-y-4">
      <KeyValueBadgeList
        title="Labels"
        :items="node.labelsMap"
        variant="tag"
        empty-message="No labels attached to this node."
      />
      <KeyValueBadgeList
        title="Annotations"
        :items="node.annotations"
        variant="list"
        empty-message="No annotations attached to this node."
      />
    </div>
  </div>
</template>
