<script setup lang="ts">
import { ref } from 'vue'
import { RefreshCw, ChevronDown, Clock, Cloud } from '@lucide/vue'

const isRefreshing = ref(false)

const handleRefresh = () => {
  isRefreshing.value = true
  setTimeout(() => {
    isRefreshing.value = false
  }, 1000)
}
</script>

<template>
  <header
    class="bg-[var(--bg-app)] border-b border-[var(--border)] px-8 py-4 flex flex-col gap-3 select-none"
  >
    <!-- Top Row -->
    <div class="flex items-center justify-between">
      <!-- Left side: Cluster info & status -->
      <div class="flex items-center gap-4">
        <h1 class="text-2xl font-bold text-[var(--text-primary)] font-ui tracking-tight">
          production-us-east-1
        </h1>
        <div
          class="flex items-center gap-1.5 px-2 py-0.5 rounded-full bg-emerald-500/10 text-emerald-500 text-xs font-semibold border border-emerald-500/20"
        >
          <span class="w-1.5 h-1.5 rounded-full bg-emerald-500"></span>
          <span>Healthy</span>
        </div>
      </div>

      <!-- Right side: Last updated, Refresh, Actions -->
      <div class="flex items-center gap-3">
        <span class="text-xs text-[var(--text-muted)]"> Last updated: 1m ago </span>
        <button
          @click="handleRefresh"
          class="p-2 rounded-lg hover:bg-[var(--bg-hover)] text-[var(--text-secondary)] border border-[var(--border)] transition-all duration-200"
          :class="{ 'animate-spin': isRefreshing }"
          title="Refresh"
        >
          <RefreshCw class="w-4 h-4" />
        </button>
        <button
          class="px-4 py-2 rounded-lg bg-[var(--bg-card)] hover:bg-[var(--bg-hover)] text-[var(--text-primary)] border border-[var(--border)] text-sm font-medium flex items-center gap-2 transition-all duration-200"
        >
          <span>Actions</span>
          <ChevronDown class="w-4 h-4 text-[var(--text-muted)]" />
        </button>
      </div>
    </div>

    <!-- Bottom Row (Sub-metadata) -->
    <div class="flex items-center gap-6 text-xs text-[var(--text-secondary)] font-medium">
      <!-- Kubernetes Version -->
      <div class="flex items-center gap-2">
        <!-- SVG Kubernetes Icon -->
        <svg class="w-4 h-4 text-sky-500" viewBox="0 0 24 24" fill="currentColor">
          <path
            d="M12 2.69l5.66 3.27v6.54L12 15.77 6.34 12.5V5.96L12 2.69m0-1.8L5 4.96v7.88l7 4.04 7-4.04V4.96l-7-4.07zM12 17.5v5.69"
          />
          <path
            d="M5.34 13.5L2 11.58V5.37l7.34-4.24v6.23L5.34 9.48v4.02m0 2l-3.34-1.92V7.37L9.34 3.13v2.23L5.34 7.48v8.02z"
            class="opacity-70"
          />
        </svg>
        <span>Kubernetes v1.33.2</span>
      </div>

      <!-- Cloud Provider -->
      <div class="flex items-center gap-2 border-l border-[var(--border)] pl-6">
        <Cloud class="w-4 h-4 text-orange-400" />
        <span class="font-semibold text-[var(--text-muted)] uppercase text-[10px] tracking-wider"
          >aws</span
        >
        <span class="text-[var(--text-muted)]">/</span>
        <span>EKS</span>
      </div>

      <!-- Uptime -->
      <div class="flex items-center gap-2 border-l border-[var(--border)] pl-6">
        <Clock class="w-4 h-4 text-indigo-400" />
        <span>Uptime: 18d 4h 32m</span>
      </div>
    </div>
  </header>
</template>
