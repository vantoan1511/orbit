<script setup lang="ts">
import { 
  CheckCircle2, 
  Server, 
  LayoutGrid 
} from "@lucide/vue";
</script>

<template>
  <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
    <!-- Card 1: Cluster Status -->
    <div class="bg-[var(--bg-card)] border border-[var(--border)] rounded-xl p-5 flex items-center gap-5 shadow-sm transition-all duration-200 hover:border-[var(--border-strong)]">
      <div class="w-12 h-12 rounded-xl bg-emerald-500/10 flex items-center justify-center text-emerald-500 flex-shrink-0">
        <CheckCircle2 class="w-6 h-6" />
      </div>
      <div class="flex-1 min-w-0">
        <div class="text-xs font-semibold text-[var(--text-muted)] uppercase tracking-wider">Cluster Status</div>
        <div class="text-2xl font-bold text-emerald-500 mt-1">Healthy</div>
        <div class="text-xs text-[var(--text-muted)] mt-0.5 truncate">All systems normal</div>
      </div>
    </div>

    <!-- Card 2: Kubernetes Version -->
    <div class="bg-[var(--bg-card)] border border-[var(--border)] rounded-xl p-5 flex items-center gap-5 shadow-sm transition-all duration-200 hover:border-[var(--border-strong)]">
      <div class="w-12 h-12 rounded-xl bg-sky-500/10 flex items-center justify-center text-sky-500 flex-shrink-0">
        <!-- SVG Kubernetes Icon -->
        <svg class="w-6 h-6" viewBox="0 0 24 24" fill="currentColor">
          <path d="M12 2.69l5.66 3.27v6.54L12 15.77 6.34 12.5V5.96L12 2.69m0-1.8L5 4.96v7.88l7 4.04 7-4.04V4.96l-7-4.07zM12 17.5v5.69" />
          <path d="M5.34 13.5L2 11.58V5.37l7.34-4.24v6.23L5.34 9.48v4.02m0 2l-3.34-1.92V7.37L9.34 3.13v2.23L5.34 7.48v8.02z" class="opacity-70" />
        </svg>
      </div>
      <div class="flex-1 min-w-0">
        <div class="text-xs font-semibold text-[var(--text-muted)] uppercase tracking-wider">Kubernetes Version</div>
        <div class="text-2xl font-bold text-[var(--text-primary)] mt-1">v1.33.2</div>
        <div class="text-xs text-[var(--text-muted)] mt-0.5 truncate">Up to date</div>
      </div>
    </div>

    <!-- Card 3: Nodes -->
    <div class="bg-[var(--bg-card)] border border-[var(--border)] rounded-xl p-5 flex items-center gap-5 shadow-sm transition-all duration-200 hover:border-[var(--border-strong)]">
      <div class="w-12 h-12 rounded-xl bg-violet-500/10 flex items-center justify-center text-violet-400 flex-shrink-0">
        <Server class="w-6 h-6" />
      </div>
      <div class="flex-1 min-w-0">
        <div class="text-xs font-semibold text-[var(--text-muted)] uppercase tracking-wider">Nodes</div>
        <div class="text-2xl font-bold text-[var(--text-primary)] mt-1">12 <span class="text-base text-[var(--text-muted)] font-normal">/ 12</span></div>
        <div class="text-xs text-[var(--text-muted)] mt-0.5 truncate">12 Ready</div>
      </div>
    </div>

    <!-- Card 4: Namespaces -->
    <div class="bg-[var(--bg-card)] border border-[var(--border)] rounded-xl p-5 flex items-center gap-5 shadow-sm transition-all duration-200 hover:border-[var(--border-strong)]">
      <div class="w-12 h-12 rounded-xl bg-amber-500/10 flex items-center justify-center text-amber-400 flex-shrink-0">
        <LayoutGrid class="w-6 h-6" />
      </div>
      <div class="flex-1 min-w-0">
        <div class="text-xs font-semibold text-[var(--text-muted)] uppercase tracking-wider">Namespaces</div>
        <div class="text-2xl font-bold text-[var(--text-primary)] mt-1">38</div>
        <div class="text-xs text-[var(--text-muted)] mt-0.5 truncate">Active</div>
      </div>
    </div>
  </div>
</template>
