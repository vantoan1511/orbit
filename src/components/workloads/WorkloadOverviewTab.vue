<script setup lang="ts">
import KeyValueBadgeList from '@/components/shared/KeyValueBadgeList.vue'
import ReplicasProgressBar from '@/components/shared/ReplicasProgressBar.vue'
import type { DaemonSetReplicas, WorkloadInfo } from '@/types/kubernetes'

defineProps<{
  workload: WorkloadInfo | null
  workloadKind: string
  workloadNamespace: string
  workloadAge: string
  workloadStatus: string
  replicas: DaemonSetReplicas | null
  desiredReplicas?: number
  currentReplicas?: number
  readyReplicas?: number
  availableReplicas?: number
  completions?: string
  duration?: string
  schedule?: string
  suspend?: boolean
  active?: number
  lastSchedule?: string
  strategy?: string
  minReadySeconds?: number
  revisionHistory?: number
  workloadImages: string[]
  workloadLabels: Record<string, string>
  workloadAnnotations: Record<string, string>
}>()
</script>

<template>
  <div class="space-y-6">
    <!-- Replicas Progress Bars -->
    <ReplicasProgressBar
      v-if="replicas"
      :desired="desiredReplicas"
      :current="currentReplicas"
      :ready="readyReplicas"
      :available="availableReplicas"
    />

    <!-- Job Status -->
    <div v-if="completions !== undefined">
      <h3 class="text-[10px] font-bold text-(--text-muted) uppercase tracking-wider mb-3">
        Job Status
      </h3>
      <div class="bg-(--bg-hover)/50 border border-(--border) rounded-xl p-4 text-xs space-y-3">
        <div class="flex justify-between">
          <span class="text-(--text-secondary) font-medium">Completions</span>
          <span class="font-mono font-bold text-(--text-primary)">{{ completions }}</span>
        </div>
        <div v-if="duration" class="flex justify-between">
          <span class="text-(--text-secondary) font-medium">Duration</span>
          <span class="font-mono text-(--text-primary)">{{ duration }}</span>
        </div>
      </div>
    </div>

    <!-- CronJob Schedule -->
    <div v-if="schedule">
      <h3 class="text-[10px] font-bold text-(--text-muted) uppercase tracking-wider mb-3">
        CronJob Schedule
      </h3>
      <div class="bg-(--bg-hover)/50 border border-(--border) rounded-xl p-4 text-xs space-y-3">
        <div class="flex justify-between">
          <span class="text-(--text-secondary) font-medium">Schedule</span>
          <span class="font-mono font-bold text-(--text-primary)">{{ schedule }}</span>
        </div>
        <div class="flex justify-between">
          <span class="text-(--text-secondary) font-medium">Suspend</span>
          <span class="font-mono text-(--text-primary)">{{ suspend ? 'True' : 'False' }}</span>
        </div>
        <div class="flex justify-between">
          <span class="text-(--text-secondary) font-medium">Active Jobs</span>
          <span class="font-mono text-(--text-primary)">{{ active ?? 0 }}</span>
        </div>
        <div v-if="lastSchedule" class="flex justify-between">
          <span class="text-(--text-secondary) font-medium">Last Schedule</span>
          <span class="font-mono text-(--text-primary)">{{ lastSchedule }}</span>
        </div>
      </div>
    </div>

    <!-- Configuration Metadata Grid -->
    <div>
      <h3 class="text-[10px] font-bold text-(--text-muted) uppercase tracking-wider mb-3">
        Configuration
      </h3>
      <div
        class="grid grid-cols-2 gap-4 bg-(--bg-hover)/30 border border-(--border) rounded-xl p-4 text-xs"
      >
        <div>
          <span class="text-(--text-muted) block mb-0.5">Kind</span>
          <span class="font-semibold text-(--text-secondary)">{{ workloadKind }}</span>
        </div>
        <div>
          <span class="text-(--text-muted) block mb-0.5">Namespace</span>
          <span class="font-semibold text-(--text-secondary)">{{ workloadNamespace }}</span>
        </div>
        <div>
          <span class="text-(--text-muted) block mb-0.5">Age</span>
          <span class="font-semibold text-(--text-secondary)">{{ workloadAge }}</span>
        </div>
        <div>
          <span class="text-(--text-muted) block mb-0.5">Status</span>
          <span class="font-semibold text-(--text-secondary)">{{ workloadStatus }}</span>
        </div>
        <div v-if="strategy">
          <span class="text-(--text-muted) block mb-0.5">Strategy</span>
          <span class="font-semibold text-(--text-secondary) truncate block" :title="strategy">
            {{ strategy }}
          </span>
        </div>
        <div v-if="minReadySeconds !== undefined">
          <span class="text-(--text-muted) block mb-0.5">Min Ready Seconds</span>
          <span class="font-mono text-(--text-secondary)">{{ minReadySeconds }}s</span>
        </div>
        <div v-if="revisionHistory !== undefined">
          <span class="text-(--text-muted) block mb-0.5">Revision History Limit</span>
          <span class="font-mono text-(--text-secondary)">{{ revisionHistory }}</span>
        </div>
        <div class="col-span-2" v-if="workloadImages && workloadImages.length">
          <span class="text-(--text-muted) block mb-0.5">Container Images</span>
          <div class="flex flex-wrap gap-1.5 mt-1">
            <span
              v-for="img in workloadImages"
              :key="img"
              class="bg-(--bg-hover) text-(--text-secondary) font-mono text-[10px] px-2 py-0.5 rounded border border-(--border) truncate max-w-full"
              :title="img"
            >
              {{ img }}
            </span>
          </div>
        </div>
      </div>
    </div>

    <!-- Labels & Annotations -->
    <KeyValueBadgeList :items="workloadLabels" title="Labels" variant="tag" />

    <KeyValueBadgeList :items="workloadAnnotations" title="Annotations" variant="list" />
  </div>
</template>
