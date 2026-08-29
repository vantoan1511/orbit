<script setup lang="ts">
import type { PodInfo } from '@/types/kubernetes'
import ReactiveAge from '@/components/shared/ReactiveAge.vue'
import { Info, Terminal } from '@lucide/vue'
import Button from 'primevue/button'

defineProps<{
  pods: PodInfo[]
  getStatusBadgeClass: (status: string) => string
}>()

const emit = defineEmits<{
  (e: 'view-pod-logs', podName: string): void
}>()
</script>

<template>
  <div class="space-y-4">
    <div class="text-[10px] font-bold text-muted-color uppercase tracking-wider mb-1">
      Active Pods ({{ pods.length }})
    </div>
    <div class="space-y-2.5">
      <template v-if="pods.length > 0">
        <div
          v-for="pod in pods"
          :key="pod.name"
          class="flex items-center justify-between p-3.5 bg-(--bg-hover)/40 rounded-xl hover:bg-(--bg-hover)/60 transition-colors"
        >
          <div class="flex items-center gap-3 min-w-0">
            <span
              class="w-2.5 h-2.5 rounded-full shrink-0 animate-pulse"
              :class="getStatusBadgeClass(pod.status)"
            ></span>
            <div class="min-w-0">
              <span
                class="text-xs font-semibold text-primary font-mono truncate block"
                :title="pod.name"
              >
                {{ pod.name }}
              </span>
              <div class="flex items-center gap-3 text-[10px] text-muted-color font-mono mt-0.5">
                <span>IP: {{ pod.ip || 'N/A' }}</span>
                <span>Node: {{ pod.node || 'N/A' }}</span>
                <span v-if="pod.restarts !== undefined">Restarts: {{ pod.restarts }}</span>
                <span>Age: <ReactiveAge :age="pod.age" /></span>
              </div>
            </div>
          </div>

          <Button
            severity="secondary"
            size="small"
            variant="text"
            class="text-xs shrink-0"
            title="View Logs"
            @click="emit('view-pod-logs', pod.name)"
          >
            <Terminal class="w-3.5 h-3.5" />
          </Button>
        </div>
      </template>
      <div
        v-else
        class="text-center py-10 text-xs text-muted-color flex flex-col items-center gap-2"
      >
        <Info class="w-8 h-8 text-muted-color/50" />
        <span>No active pods found for this workload.</span>
      </div>
    </div>
  </div>
</template>
