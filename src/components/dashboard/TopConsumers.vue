<script setup lang="ts">
import { useKubernetesStore } from '@/stores/kubernetesStore'
import { ArrowRight } from '@lucide/vue'
import { computed, ref } from 'vue'

const kubernetesStore = useKubernetesStore()
const activeTab = ref('cpu')

const consumers = computed(() => {
  const pods = [...kubernetesStore.pods]

  if (activeTab.value === 'cpu') {
    return pods
      .sort((a, b) => (b.cpuPct || 0) - (a.cpuPct || 0))
      .slice(0, 5)
      .map((p) => ({
        pod: p.name,
        namespace: p.namespace,
        value: p.cpu || '0m',
        pct: Math.min(Math.max(Math.round(p.cpuPct || 0), 0), 100),
        color: 'bg-blue-500'
      }))
  } else if (activeTab.value === 'memory') {
    return pods
      .sort((a, b) => (b.memoryPct || 0) - (a.memoryPct || 0))
      .slice(0, 5)
      .map((p) => ({
        pod: p.name,
        namespace: p.namespace,
        value: p.memory || '0Mi',
        pct: Math.min(Math.max(Math.round(p.memoryPct || 0), 0), 100),
        color: 'bg-violet-500'
      }))
  } else {
    return pods
      .sort((a, b) => (b.restarts || 0) - (a.restarts || 0))
      .slice(0, 5)
      .map((p) => ({
        pod: p.name,
        namespace: p.namespace,
        value: String(p.restarts || 0),
        pct: 0,
        color: 'bg-amber-500'
      }))
  }
})
</script>

<template>
  <div
    class="bg-(--bg-card) border border-(--border) rounded-xl p-5 shadow-sm transition-all duration-200 flex flex-col justify-between h-full"
  >
    <div>
      <!-- Header with Tabs -->
      <div class="flex items-center justify-between mb-5 flex-wrap gap-2">
        <div class="text-xs font-semibold text-(--text-primary) uppercase tracking-wider">
          Top Consumers
        </div>

        <!-- Tab Selector -->
        <div class="flex rounded-lg bg-(--bg-hover) p-0.5 border border-(--border)">
          <button
            @click="activeTab = 'cpu'"
            class="px-2.5 py-1 text-[10px] font-bold rounded-md transition-all duration-200"
            :class="
              activeTab === 'cpu'
                ? 'bg-(--bg-card) text-(--text-primary) shadow-sm'
                : 'text-(--text-secondary) hover:text-(--text-primary)'
            "
          >
            CPU
          </button>
          <button
            @click="activeTab = 'memory'"
            class="px-2.5 py-1 text-[10px] font-bold rounded-md transition-all duration-200"
            :class="
              activeTab === 'memory'
                ? 'bg-(--bg-card) text-(--text-primary) shadow-sm'
                : 'text-(--text-secondary) hover:text-(--text-primary)'
            "
          >
            Memory
          </button>
          <button
            @click="activeTab = 'restarts'"
            class="px-2.5 py-1 text-[10px] font-bold rounded-md transition-all duration-200"
            :class="
              activeTab === 'restarts'
                ? 'bg-(--bg-card) text-(--text-primary) shadow-sm'
                : 'text-(--text-secondary) hover:text-(--text-primary)'
            "
          >
            Restarts
          </button>
        </div>
      </div>

      <!-- Table -->
      <div class="overflow-x-auto">
        <table class="w-full text-left text-xs border-collapse">
          <thead>
            <tr class="text-(--text-muted) font-semibold border-b border-(--border) pb-2">
              <th class="pb-2 font-medium">Pod</th>
              <th class="pb-2 font-medium">Namespace</th>
              <th class="pb-2 font-medium" :class="activeTab === 'restarts' ? 'text-right' : ''">
                {{
                  activeTab === 'cpu'
                    ? 'CPU (cores)'
                    : activeTab === 'memory'
                      ? 'Memory'
                      : 'Restarts'
                }}
              </th>
              <th v-if="activeTab !== 'restarts'" class="pb-2 font-medium text-right">Usage (%)</th>
            </tr>
          </thead>
          <tbody class="divide-y divide-(--border)/50">
            <tr
              v-for="item in consumers"
              :key="item.pod"
              class="text-(--text-secondary) hover:text-(--text-primary)"
            >
              <td class="py-2.5 font-medium truncate max-w-25" :title="item.pod">
                {{ item.pod }}
              </td>
              <td class="py-2.5 text-(--text-muted)">{{ item.namespace }}</td>
              <td class="py-2.5 font-mono" :class="activeTab === 'restarts' ? 'text-right' : ''">
                {{ item.value }}
              </td>
              <td v-if="activeTab !== 'restarts'" class="py-2.5 text-right font-mono">
                <div class="flex items-center justify-end gap-2">
                  <span class="w-8 text-right">{{ item.pct }}%</span>
                  <div
                    class="w-16 h-1.5 rounded-full bg-(--bg-hover) overflow-hidden hidden sm:block"
                  >
                    <div
                      :style="{ width: item.pct + '%' }"
                      :class="item.color"
                      class="h-full"
                    ></div>
                  </div>
                </div>
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>

    <!-- View All Link -->
    <router-link
      to="/pods"
      class="text-xs text-(--accent) hover:underline flex items-center justify-between font-semibold mt-4 border-t border-(--border) pt-4 w-full"
    >
      <span>View all resource usage</span>
      <ArrowRight class="w-3.5 h-3.5" />
    </router-link>
  </div>
</template>
