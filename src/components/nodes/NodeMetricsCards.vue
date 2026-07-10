<script setup lang="ts">
import { Server, Cpu, Database, Box, Activity } from '@lucide/vue'
</script>

<template>
  <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-5 gap-6">
    <!-- Card 1: Nodes Count -->
    <div
      class="bg-[var(--bg-card)] border border-[var(--border)] rounded-xl p-5 flex items-center gap-5 shadow-sm transition-all duration-200 hover:border-[var(--border-strong)]"
    >
      <div
        class="w-12 h-12 rounded-xl bg-violet-500/10 flex items-center justify-center text-violet-400 flex-shrink-0"
      >
        <Server class="w-6 h-6" />
      </div>
      <div class="flex-1 min-w-0">
        <div class="text-[10px] font-bold text-[var(--text-muted)] uppercase tracking-wider">
          Total Nodes
        </div>
        <div class="text-2xl font-bold text-[var(--text-primary)] mt-1">
          12 <span class="text-sm text-[var(--text-muted)] font-normal">/ 12</span>
        </div>
        <div class="text-xs text-emerald-500 mt-0.5 font-medium">12 Online (100%)</div>
      </div>
    </div>

    <!-- Card 2: CPU Allocation -->
    <div
      class="bg-[var(--bg-card)] border border-[var(--border)] rounded-xl p-5 flex flex-col justify-between shadow-sm transition-all duration-200 hover:border-[var(--border-strong)]"
    >
      <div class="flex items-center gap-5">
        <div
          class="w-12 h-12 rounded-xl bg-blue-500/10 flex items-center justify-center text-blue-400 flex-shrink-0"
        >
          <Cpu class="w-6 h-6" />
        </div>
        <div class="flex-1 min-w-0">
          <div class="text-[10px] font-bold text-[var(--text-muted)] uppercase tracking-wider">
            CPU Allocation
          </div>
          <div class="text-2xl font-bold text-[var(--text-primary)] mt-1">58%</div>
        </div>
      </div>
      <div class="mt-4">
        <div class="flex justify-between text-[10px] text-[var(--text-muted)] mb-1 font-mono">
          <span>27.8 / 48 cores</span>
        </div>
        <div class="w-full h-1.5 rounded-full bg-[var(--bg-hover)] overflow-hidden">
          <div class="h-full bg-blue-500 rounded-full" style="width: 58%"></div>
        </div>
      </div>
    </div>

    <!-- Card 3: Memory Allocation -->
    <div
      class="bg-[var(--bg-card)] border border-[var(--border)] rounded-xl p-5 flex flex-col justify-between shadow-sm transition-all duration-200 hover:border-[var(--border-strong)]"
    >
      <div class="flex items-center gap-5">
        <div
          class="w-12 h-12 rounded-xl bg-indigo-500/10 flex items-center justify-center text-indigo-400 flex-shrink-0"
        >
          <Database class="w-6 h-6" />
        </div>
        <div class="flex-1 min-w-0">
          <div class="text-[10px] font-bold text-[var(--text-muted)] uppercase tracking-wider">
            Memory Allocation
          </div>
          <div class="text-2xl font-bold text-[var(--text-primary)] mt-1">72%</div>
        </div>
      </div>
      <div class="mt-4">
        <div class="flex justify-between text-[10px] text-[var(--text-muted)] mb-1 font-mono">
          <span>138.2 / 192 GiB</span>
        </div>
        <div class="w-full h-1.5 rounded-full bg-[var(--bg-hover)] overflow-hidden">
          <div class="h-full bg-indigo-500 rounded-full" style="width: 72%"></div>
        </div>
      </div>
    </div>

    <!-- Card 4: Pods Allocation -->
    <div
      class="bg-[var(--bg-card)] border border-[var(--border)] rounded-xl p-5 flex flex-col justify-between shadow-sm transition-all duration-200 hover:border-[var(--border-strong)]"
    >
      <div class="flex items-center gap-5">
        <div
          class="w-12 h-12 rounded-xl bg-sky-500/10 flex items-center justify-center text-sky-400 flex-shrink-0"
        >
          <Box class="w-6 h-6" />
        </div>
        <div class="flex-1 min-w-0">
          <div class="text-[10px] font-bold text-[var(--text-muted)] uppercase tracking-wider">
            Pods Allocation
          </div>
          <div class="text-2xl font-bold text-[var(--text-primary)] mt-1">
            84 <span class="text-sm text-[var(--text-muted)] font-normal">/ 120</span>
          </div>
        </div>
      </div>
      <div class="mt-4">
        <div class="flex justify-between text-[10px] text-[var(--text-muted)] mb-1 font-mono">
          <span>70% Capacity</span>
        </div>
        <div class="w-full h-1.5 rounded-full bg-[var(--bg-hover)] overflow-hidden">
          <div class="h-full bg-sky-500 rounded-full" style="width: 70%"></div>
        </div>
      </div>
    </div>

    <!-- Card 5: Pressure Status -->
    <div
      class="bg-[var(--bg-card)] border border-[var(--border)] rounded-xl p-5 flex items-center gap-5 shadow-sm transition-all duration-200 hover:border-[var(--border-strong)]"
    >
      <div
        class="w-12 h-12 rounded-xl bg-emerald-500/10 flex items-center justify-center text-emerald-400 flex-shrink-0"
      >
        <Activity class="w-6 h-6" />
      </div>
      <div class="flex-1 min-w-0">
        <div class="text-[10px] font-bold text-[var(--text-muted)] uppercase tracking-wider">
          Node Pressures
        </div>
        <div class="text-lg font-bold text-emerald-500 mt-1">Normal</div>
        <div class="text-[10px] text-[var(--text-muted)] mt-1 flex gap-2">
          <span class="flex items-center gap-1">
            <span class="w-1.5 h-1.5 rounded-full bg-emerald-500"></span>Disk
          </span>
          <span class="flex items-center gap-1">
            <span class="w-1.5 h-1.5 rounded-full bg-emerald-500"></span>Mem
          </span>
          <span class="flex items-center gap-1">
            <span class="w-1.5 h-1.5 rounded-full bg-emerald-500"></span>PID
          </span>
        </div>
      </div>
    </div>
  </div>
</template>
