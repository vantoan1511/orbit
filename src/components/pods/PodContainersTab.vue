<script setup lang="ts">
import type { PodContainer } from '@/types/kubernetes'
import { Terminal, Info } from '@lucide/vue'
import Button from 'primevue/button'

defineProps<{
  containers: PodContainer[]
}>()

const emit = defineEmits<{
  (e: 'view-logs', containerName: string): void
}>()

const isRunning = (status: string) => {
  return status.toLowerCase() === 'running'
}
</script>

<template>
  <div class="space-y-4">
    <div class="text-[10px] font-bold text-(--text-muted) uppercase tracking-wider mb-1">
      Containers ({{ containers.length }})
    </div>

    <template v-if="containers.length > 0">
      <div
        v-for="c in containers"
        :key="c.name"
        class="p-4 bg-(--bg-card) border border-(--border) rounded-xl space-y-3 hover:bg-(--bg-hover)/30 transition-colors"
      >
        <div class="flex items-center justify-between">
          <div class="flex items-center gap-2.5 min-w-0">
            <span
              class="w-2.5 h-2.5 rounded-full shrink-0 animate-pulse"
              :class="
                isRunning(c.status)
                  ? 'bg-emerald-500 ring-4 ring-emerald-500/20'
                  : 'bg-rose-500 ring-4 ring-rose-500/20'
              "
            ></span>
            <span
              class="text-xs font-bold text-(--text-primary) font-mono truncate"
              :title="c.name"
            >
              {{ c.name }}
            </span>
            <span
              class="text-[9px] px-1.5 py-0.2 rounded font-mono border"
              :class="
                c.ready === 'true'
                  ? 'bg-emerald-500/10 text-emerald-400 border-emerald-500/20'
                  : 'bg-amber-500/10 text-amber-400 border-amber-500/20'
              "
            >
              Ready: {{ c.ready }}
            </span>
          </div>

          <Button
            severity="secondary"
            size="small"
            variant="outlined"
            class="text-xs shrink-0 flex items-center gap-1.5"
            title="View Container Logs"
            @click="emit('view-logs', c.name)"
          >
            <Terminal class="w-3.5 h-3.5" />
            <span>Logs</span>
          </Button>
        </div>

        <div
          class="grid grid-cols-2 sm:grid-cols-4 gap-2 pt-2 border-t border-(--border)/50 text-xs"
        >
          <div>
            <div class="text-[10px] text-(--text-muted)">Image</div>
            <div class="font-mono text-(--text-primary) truncate" :title="c.image">
              {{ c.image }}
            </div>
          </div>
          <div>
            <div class="text-[10px] text-(--text-muted)">Ports</div>
            <div class="font-mono text-(--text-primary)">{{ c.ports || '-' }}</div>
          </div>
          <div>
            <div class="text-[10px] text-(--text-muted)">Restarts</div>
            <div class="font-mono text-(--text-primary)">{{ c.restarts }}</div>
          </div>
          <div>
            <div class="text-[10px] text-(--text-muted)">Status</div>
            <div class="font-semibold text-(--text-primary)">{{ c.status }}</div>
          </div>
        </div>
      </div>
    </template>

    <div
      v-else
      class="text-center py-10 text-xs text-(--text-muted) flex flex-col items-center gap-2"
    >
      <Info class="w-8 h-8 text-(--text-muted)/50" />
      <span>No container specs found for this pod.</span>
    </div>
  </div>
</template>
