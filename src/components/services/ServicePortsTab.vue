<script setup lang="ts">
import type { ServiceInfo } from '@/types/kubernetes'

defineProps<{
  service: ServiceInfo
}>()
</script>

<template>
  <div class="space-y-4">
    <div class="flex items-center justify-between">
      <h3 class="text-[10px] font-bold text-muted-color uppercase tracking-wider">
        Port Mappings ({{ service.portsList.length }})
      </h3>
    </div>

    <div
      v-if="service.portsList.length > 0"
      class="border border-(--border) rounded-lg overflow-hidden bg-(--bg-card) text-xs"
    >
      <table class="w-full text-left border-collapse">
        <thead>
          <tr
            class="bg-(--bg-hover)/60 border-b border-(--border) text-muted-color text-[11px] font-semibold tracking-wider uppercase"
          >
            <th class="py-2.5 px-3.5">Port</th>
            <th class="py-2.5 px-3.5">Target Port</th>
            <th class="py-2.5 px-3.5">Protocol</th>
            <th class="py-2.5 px-3.5">Node Port</th>
          </tr>
        </thead>
        <tbody class="divide-y divide-(--border)">
          <tr
            v-for="(port, idx) in service.portsList"
            :key="idx"
            class="text-muted-color hover:bg-(--bg-hover) transition-colors"
          >
            <td class="py-2.5 px-3.5 font-mono font-medium text-primary">{{ port.port }}</td>
            <td class="py-2.5 px-3.5 font-mono text-primary">{{ port.targetPort }}</td>
            <td class="py-2.5 px-3.5">
              <span
                class="px-1.5 py-0.5 rounded bg-(--bg-hover) font-semibold text-[10px] uppercase text-primary"
              >
                {{ port.protocol }}
              </span>
            </td>
            <td class="py-2.5 px-3.5 font-mono">{{ port.nodePort || '-' }}</td>
          </tr>
        </tbody>
      </table>
    </div>

    <div v-else class="bg-(--bg-hover)/20 rounded-xl p-4 text-xs text-muted-color italic">
      No ports configured for this service.
    </div>
  </div>
</template>
