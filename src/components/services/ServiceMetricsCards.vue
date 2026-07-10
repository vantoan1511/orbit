<script setup lang="ts">
import { Network, Circle, Cpu, Cloud, Globe } from '@lucide/vue'

// Metrics stats
const totalServices = 96
const clusterIPCount = 72
const nodePortCount = 18
const loadBalancerCount = 5
const externalNameCount = 1
</script>

<template>
  <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-5 gap-6">
    <!-- Card 1: Total Services -->
    <div
      class="bg-(--bg-card) border border-(--border) rounded-xl p-5 flex items-center gap-4 shadow-sm transition-all duration-200 hover:border-(--border-strong)"
    >
      <div
        class="w-10 h-10 rounded-lg bg-(--bg-hover) flex items-center justify-center text-(--text-muted)"
      >
        <Network class="w-5 h-5" />
      </div>
      <div>
        <div class="text-[11px] font-semibold text-(--text-muted) uppercase tracking-wider">
          Total Services
        </div>
        <div class="text-2xl font-bold text-(--text-primary) font-mono mt-0.5">
          {{ totalServices }}
        </div>
        <div class="text-[10px] text-(--text-muted) mt-0.5">Across 12 namespaces</div>
      </div>
    </div>

    <!-- Card 2: Cluster IP -->
    <div
      class="bg-(--bg-card) border border-(--border) rounded-xl p-5 flex items-center gap-4 shadow-sm transition-all duration-200 hover:border-(--border-strong)"
    >
      <div
        class="w-10 h-10 rounded-lg bg-emerald-500/10 flex items-center justify-center text-emerald-400"
      >
        <Circle class="w-5 h-5" />
      </div>
      <div>
        <div class="text-[11px] font-semibold text-(--text-muted) uppercase tracking-wider">
          Cluster IP
        </div>
        <div class="text-2xl font-bold text-(--text-primary) font-mono mt-0.5">
          {{ clusterIPCount }}
        </div>
        <div class="text-[10px] text-emerald-400 font-medium mt-0.5">75.0%</div>
      </div>
    </div>

    <!-- Card 3: NodePort -->
    <div
      class="bg-(--bg-card) border border-(--border) rounded-xl p-5 flex items-center gap-4 shadow-sm transition-all duration-200 hover:border-(--border-strong)"
    >
      <div
        class="w-10 h-10 rounded-lg bg-blue-500/10 flex items-center justify-center text-blue-400"
      >
        <Cpu class="w-5 h-5" />
      </div>
      <div>
        <div class="text-[11px] font-semibold text-(--text-muted) uppercase tracking-wider">
          NodePort
        </div>
        <div class="text-2xl font-bold text-(--text-primary) font-mono mt-0.5">
          {{ nodePortCount }}
        </div>
        <div class="text-[10px] text-blue-400 font-medium mt-0.5">18.8%</div>
      </div>
    </div>

    <!-- Card 4: LoadBalancer -->
    <div
      class="bg-(--bg-card) border border-(--border) rounded-xl p-5 flex items-center gap-4 shadow-sm transition-all duration-200 hover:border-(--border-strong)"
    >
      <div
        class="w-10 h-10 rounded-lg bg-violet-500/10 flex items-center justify-center text-violet-400"
      >
        <Cloud class="w-5 h-5" />
      </div>
      <div>
        <div class="text-[11px] font-semibold text-(--text-muted) uppercase tracking-wider">
          LoadBalancer
        </div>
        <div class="text-2xl font-bold text-(--text-primary) font-mono mt-0.5">
          {{ loadBalancerCount }}
        </div>
        <div class="text-[10px] text-violet-400 font-medium mt-0.5">5.2%</div>
      </div>
    </div>

    <!-- Card 5: External Name -->
    <div
      class="bg-(--bg-card) border border-(--border) rounded-xl p-5 flex items-center gap-4 shadow-sm transition-all duration-200 hover:border-(--border-strong)"
    >
      <div
        class="w-10 h-10 rounded-lg bg-amber-500/10 flex items-center justify-center text-amber-400"
      >
        <Globe class="w-5 h-5" />
      </div>
      <div>
        <div class="text-[11px] font-semibold text-(--text-muted) uppercase tracking-wider">
          External Name
        </div>
        <div class="text-2xl font-bold text-(--text-primary) font-mono mt-0.5">
          {{ externalNameCount }}
        </div>
        <div class="text-[10px] text-amber-400 font-medium mt-0.5">1.0%</div>
      </div>
    </div>
  </div>
</template>
