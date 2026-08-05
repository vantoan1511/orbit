<script setup lang="ts">
import type { EventInfo } from '@/types/kubernetes'
import { Info } from '@lucide/vue'

defineProps<{
  events: EventInfo[]
}>()
</script>

<template>
  <div class="space-y-4">
    <div class="text-[10px] font-bold text-muted-color uppercase tracking-wider mb-1">
      Recent Events ({{ events.length }})
    </div>
    <div v-if="events.length > 0" class="relative pl-4 border-l border-(--border) space-y-4 ml-2">
      <div v-for="(ev, idx) in events" :key="idx" class="relative">
        <span
          class="absolute -left-5.25 top-1 w-2.5 h-2.5 rounded-full ring-4 ring-(--bg-card)"
          :class="ev.type === 'Warning' ? 'bg-rose-500' : 'bg-emerald-500'"
        ></span>
        <div class="flex items-center gap-2">
          <span class="text-xs font-semibold text-primary">{{ ev.reason }}</span>
          <span
            class="text-[9px] px-1.5 py-0.2 rounded font-mono border"
            :class="
              ev.type === 'Warning'
                ? 'bg-rose-500/10 text-rose-400 border-rose-500/20'
                : 'bg-emerald-500/10 text-emerald-400 border-emerald-500/20'
            "
          >
            {{ ev.type }}
          </span>
          <span v-if="ev.count > 1" class="text-[9px] text-muted-color font-mono">
            (x{{ ev.count }})
          </span>
        </div>
        <div class="text-[10px] text-muted-color mt-0.5">
          {{ ev.message }}
        </div>
        <div class="text-[9px] font-mono text-muted-color mt-1 flex items-center gap-2">
          <span>Object: {{ ev.objectKind }}/{{ ev.objectName }}</span>
          <span>•</span>
          <span>Source: {{ ev.source }}</span>
          <span>•</span>
          <span>{{ ev.lastSeen || ev.time || 'recent' }}</span>
        </div>
      </div>
    </div>
    <div v-else class="text-center py-10 text-xs text-muted-color flex flex-col items-center gap-2">
      <Info class="w-8 h-8 text-muted-color/50" />
      <span>No recent events recorded for this workload.</span>
    </div>
  </div>
</template>
