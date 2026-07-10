<script setup lang="ts">
import { computed } from 'vue'
import { FileText, ShieldAlert, Activity, Boxes, FileWarning, Lock } from '@lucide/vue'

const props = defineProps<{
  activeTab: 'configmaps' | 'secrets'
}>()

// Calculate mock states based on activeTab
const metrics = computed(() => {
  if (props.activeTab === 'configmaps') {
    return [
      {
        title: 'ConfigMaps',
        value: '128',
        subtext: 'Across 12 namespaces',
        icon: FileText,
        iconClass: 'bg-(--bg-hover) text-(--text-muted)'
      },
      {
        title: 'Total Size',
        value: '4.2 MiB',
        subtext: '+12% vs last 7d',
        subtextClass: 'text-emerald-400 font-medium',
        icon: Activity,
        iconClass: 'bg-emerald-500/10 text-emerald-400'
      },
      {
        title: 'Mounted in Pods',
        value: '62',
        subtext: '48% of total',
        subtextClass: 'text-blue-400 font-medium',
        icon: Boxes,
        iconClass: 'bg-blue-500/10 text-blue-400'
      },
      {
        title: 'Orphaned',
        value: '8',
        subtext: 'Not referenced',
        subtextClass: 'text-amber-400 font-medium',
        icon: FileWarning,
        iconClass: 'bg-amber-500/10 text-amber-400'
      }
    ]
  } else {
    return [
      {
        title: 'Secrets',
        value: '64',
        subtext: 'Across 10 namespaces',
        icon: Lock,
        iconClass: 'bg-(--bg-hover) text-(--text-muted)'
      },
      {
        title: 'Total Size',
        value: '1.1 MiB',
        subtext: '-4% vs last 7d',
        subtextClass: 'text-emerald-400 font-medium',
        icon: Activity,
        iconClass: 'bg-emerald-500/10 text-emerald-400'
      },
      {
        title: 'Mounted in Pods',
        value: '30',
        subtext: '46.8% of total',
        subtextClass: 'text-blue-400 font-medium',
        icon: Boxes,
        iconClass: 'bg-blue-500/10 text-blue-400'
      },
      {
        title: 'Orphaned',
        value: '5',
        subtext: 'Not referenced',
        subtextClass: 'text-rose-400 font-medium',
        icon: ShieldAlert,
        iconClass: 'bg-rose-500/10 text-rose-400'
      }
    ]
  }
})
</script>

<template>
  <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-6">
    <div
      v-for="metric in metrics"
      :key="metric.title"
      class="bg-(--bg-card) border border-(--border) rounded-xl p-5 flex items-center gap-4 shadow-sm transition-all duration-200 hover:border-(--border-strong)"
    >
      <div class="w-10 h-10 rounded-lg flex items-center justify-center" :class="metric.iconClass">
        <component :is="metric.icon" class="w-5 h-5" />
      </div>
      <div>
        <div class="text-[11px] font-semibold text-(--text-muted) uppercase tracking-wider">
          {{ metric.title }}
        </div>
        <div class="text-2xl font-bold text-(--text-primary) font-mono mt-0.5">
          {{ metric.value }}
        </div>
        <div class="text-[10px]" :class="metric.subtextClass || 'text-(--text-muted)'">
          {{ metric.subtext }}
        </div>
      </div>
    </div>
  </div>
</template>
