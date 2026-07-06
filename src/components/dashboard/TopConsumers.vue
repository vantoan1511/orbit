<script setup lang="ts">
import { ref } from "vue";
import { ArrowRight } from "@lucide/vue";

const activeTab = ref("cpu");

const consumers = ref([
  { pod: "api-0", namespace: "backend", value: "850m", pct: 21, color: "bg-blue-500" },
  { pod: "worker-2", namespace: "jobs", value: "650m", pct: 16, color: "bg-blue-500" },
  { pod: "redis-0", namespace: "cache", value: "590m", pct: 14, color: "bg-blue-500" },
  { pod: "api-1", namespace: "backend", value: "480m", pct: 12, color: "bg-blue-500" },
  { pod: "reporting-6b7c5d", namespace: "analytics", value: "410m", pct: 10, color: "bg-blue-500" }
]);
</script>

<template>
  <div class="bg-[var(--bg-card)] border border-[var(--border)] rounded-xl p-5 shadow-sm transition-all duration-200 flex flex-col justify-between h-full">
    <div>
      <!-- Header with Tabs -->
      <div class="flex items-center justify-between mb-5 flex-wrap gap-2">
        <div class="text-xs font-semibold text-[var(--text-primary)] uppercase tracking-wider">
          Top Consumers
        </div>
        
        <!-- Tab Selector -->
        <div class="flex rounded-lg bg-[var(--bg-hover)] p-0.5 border border-[var(--border)]">
          <button 
            @click="activeTab = 'cpu'"
            class="px-2.5 py-1 text-[10px] font-bold rounded-md transition-all duration-200"
            :class="activeTab === 'cpu' ? 'bg-[var(--bg-card)] text-[var(--text-primary)] shadow-sm' : 'text-[var(--text-secondary)] hover:text-[var(--text-primary)]'"
          >
            CPU
          </button>
          <button 
            @click="activeTab = 'memory'"
            class="px-2.5 py-1 text-[10px] font-bold rounded-md transition-all duration-200"
            :class="activeTab === 'memory' ? 'bg-[var(--bg-card)] text-[var(--text-primary)] shadow-sm' : 'text-[var(--text-secondary)] hover:text-[var(--text-primary)]'"
          >
            Memory
          </button>
          <button 
            @click="activeTab = 'restarts'"
            class="px-2.5 py-1 text-[10px] font-bold rounded-md transition-all duration-200"
            :class="activeTab === 'restarts' ? 'bg-[var(--bg-card)] text-[var(--text-primary)] shadow-sm' : 'text-[var(--text-secondary)] hover:text-[var(--text-primary)]'"
          >
            Restarts
          </button>
        </div>
      </div>

      <!-- Table -->
      <div class="overflow-x-auto">
        <table class="w-full text-left text-xs border-collapse">
          <thead>
            <tr class="text-[var(--text-muted)] font-semibold border-b border-[var(--border)] pb-2">
              <th class="pb-2 font-medium">Pod</th>
              <th class="pb-2 font-medium">Namespace</th>
              <th class="pb-2 font-medium" :class="activeTab === 'restarts' ? 'text-right' : ''">
                {{ activeTab === 'cpu' ? 'CPU (cores)' : activeTab === 'memory' ? 'Memory' : 'Restarts' }}
              </th>
              <th v-if="activeTab !== 'restarts'" class="pb-2 font-medium text-right">Usage (%)</th>
            </tr>
          </thead>
          <tbody class="divide-y divide-[var(--border)]/50">
            <tr v-for="item in consumers" :key="item.pod" class="text-[var(--text-secondary)] hover:text-[var(--text-primary)]">
              <td class="py-2.5 font-medium truncate max-w-[100px]" :title="item.pod">{{ item.pod }}</td>
              <td class="py-2.5 text-[var(--text-muted)]">{{ item.namespace }}</td>
              <td class="py-2.5 font-mono" :class="activeTab === 'restarts' ? 'text-right' : ''">{{ activeTab === 'cpu' ? item.value : activeTab === 'memory' ? '2.4 GiB' : '2' }}</td>
              <td v-if="activeTab !== 'restarts'" class="py-2.5 text-right font-mono">
                <div class="flex items-center justify-end gap-2">
                  <span class="w-8 text-right">{{ activeTab === 'cpu' ? item.pct : item.pct - 2 }}%</span>
                  <div class="w-16 h-1.5 rounded-full bg-[var(--bg-hover)] overflow-hidden hidden sm:block">
                    <div :style="{ width: (activeTab === 'cpu' ? item.pct : item.pct - 2) * 2 + '%' }" :class="item.color" class="h-full"></div>
                  </div>
                </div>
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>

    <!-- View All Link -->
    <button class="text-xs text-[var(--accent)] hover:underline flex items-center justify-between font-semibold mt-4 border-t border-[var(--border)] pt-4 w-full">
      <span>View all resource usage</span>
      <ArrowRight class="w-3.5 h-3.5" />
    </button>
  </div>
</template>
