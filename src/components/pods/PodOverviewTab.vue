<script setup lang="ts">
import ActivePortForwardsList from '@/components/shared/ActivePortForwardsList.vue'
import KeyValueBadgeList from '@/components/shared/KeyValueBadgeList.vue'
import ReactiveAge from '@/components/shared/ReactiveAge.vue'
import type { PodInfo } from '@/types/kubernetes'

defineProps<{
  pod: PodInfo
}>()
</script>

<template>
  <div class="space-y-6">
    <!-- Metrics Section -->
    <div class="grid grid-cols-2 gap-4">
      <div class="p-4 rounded-xl bg-(--bg-hover)/40">
        <div class="text-[10px] font-bold uppercase tracking-wider text-muted-color mb-1">
          CPU Usage
        </div>
        <div class="text-lg font-bold font-mono text-primary">
          {{ pod.cpu || '-' }}
          <span v-if="pod.cpuPct !== undefined" class="text-xs text-muted-color font-normal ml-1">
            ({{ pod.cpuPct }}%)
          </span>
        </div>
      </div>
      <div class="p-4 rounded-xl bg-(--bg-hover)/40">
        <div class="text-[10px] font-bold uppercase tracking-wider text-muted-color mb-1">
          Memory Usage
        </div>
        <div class="text-lg font-bold font-mono text-primary">
          {{ pod.memory || '-' }}
          <span
            v-if="pod.memoryPct !== undefined"
            class="text-xs text-muted-color font-normal ml-1"
          >
            ({{ pod.memoryPct }}%)
          </span>
        </div>
      </div>
    </div>

    <!-- Configuration Metadata Grid -->
    <div>
      <div class="text-[10px] font-bold text-muted-color uppercase tracking-wider mb-2.5">
        Pod Configuration
      </div>
      <div class="grid grid-cols-2 gap-3 p-4 rounded-xl bg-(--bg-hover)/40 text-xs">
        <div>
          <span class="text-muted-color">Namespace: </span>
          <span class="font-semibold text-primary font-mono">{{ pod.namespace }}</span>
        </div>
        <div>
          <span class="text-muted-color">Node: </span>
          <span
            class="font-semibold text-primary font-mono truncate inline-block max-w-50 align-bottom"
            :title="pod.node || '-'"
          >
            {{ pod.node || '-' }}
          </span>
        </div>
        <div>
          <span class="text-muted-color">Status: </span>
          <span class="font-semibold text-primary">{{ pod.status }}</span>
        </div>
        <div>
          <span class="text-muted-color">IP Address: </span>
          <span class="font-semibold text-primary font-mono">{{ pod.ip || '-' }}</span>
        </div>
        <div>
          <span class="text-muted-color">Node IP: </span>
          <span class="font-semibold text-primary font-mono">{{ pod.nodeIP || '-' }}</span>
        </div>
        <div>
          <span class="text-muted-color">Controlled By: </span>
          <span
            class="font-semibold text-primary font-mono truncate inline-block max-w-50 align-bottom"
            :title="pod.controlledBy || '-'"
          >
            {{ pod.controlledBy || '-' }}
          </span>
        </div>
        <div>
          <span class="text-muted-color">QoS Class: </span>
          <span class="font-semibold text-primary font-mono">{{ pod.qosClass || '-' }}</span>
        </div>
        <div>
          <span class="text-muted-color">Restarts: </span>
          <span class="font-semibold text-primary font-mono">{{ pod.restarts ?? 0 }}</span>
        </div>
        <div>
          <span class="text-muted-color">Age: </span>
          <span class="font-semibold text-primary font-mono"><ReactiveAge :age="pod.age" /></span>
        </div>
      </div>
    </div>

    <!-- Active Port Forwards -->
    <ActivePortForwardsList kind="Pod" :namespace="pod.namespace" :name="pod.name" />

    <!-- Container Images -->
    <div v-if="pod.images && pod.images.length > 0">
      <div class="text-[10px] font-bold text-muted-color uppercase tracking-wider mb-2.5">
        Container Images ({{ pod.images.length }})
      </div>
      <div class="flex flex-wrap gap-2">
        <Tag
          v-for="img in pod.images"
          :key="img"
          severity="secondary"
          class="font-mono truncate max-w-full"
          :title="img"
          :value="img"
        />
      </div>
    </div>

    <!-- Labels & Annotations -->
    <div class="space-y-4">
      <KeyValueBadgeList
        title="Labels"
        :items="pod.labels"
        variant="tag"
        empty-message="No labels attached to this pod."
      />
      <KeyValueBadgeList
        title="Annotations"
        :items="pod.annotations"
        variant="list"
        empty-message="No annotations attached to this pod."
      />
    </div>
  </div>
</template>
