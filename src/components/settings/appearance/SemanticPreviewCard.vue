<script setup lang="ts">
import { Info } from '@lucide/vue'
</script>

<template>
  <div class="p-4 rounded-lg bg-(--bg-hover)/40 border border-(--border) flex flex-col gap-3">
    <div class="flex items-center justify-between gap-2">
      <span class="text-xs font-semibold text-primary uppercase tracking-wider">
        Live Status Preview
      </span>
      <span
        class="inline-flex items-center gap-1.5 px-2 py-0.5 rounded-full text-[11px] font-medium bg-(--success-soft) text-(--success)"
      >
        <span class="w-1.5 h-1.5 rounded-full bg-(--success) animate-pulse" />
        Cluster Healthy
      </span>
    </div>

    <!-- Status Dots and Labels Row -->
    <div class="grid grid-cols-2 sm:grid-cols-4 gap-2 text-xs">
      <div class="flex items-center gap-1.5">
        <span class="w-2 h-2 rounded-full bg-(--success)" />
        <span class="text-(--success) font-medium">Running</span>
      </div>
      <div class="flex items-center gap-1.5">
        <span class="w-2 h-2 rounded-full bg-(--warning)" />
        <span class="text-(--warning) font-medium">Pending</span>
      </div>
      <div class="flex items-center gap-1.5">
        <span class="w-2 h-2 rounded-full bg-(--danger)" />
        <span class="text-(--danger) font-medium">Failed</span>
      </div>
      <div class="flex items-center gap-1.5">
        <span class="w-2 h-2 rounded-full bg-(--info)" />
        <span class="text-(--info) font-medium">Active</span>
      </div>
    </div>

    <!-- Sample Alert Banner Pill -->
    <div class="flex items-center gap-2 p-2 rounded bg-(--info-soft) text-xs text-(--info)">
      <Info class="w-3.5 h-3.5 shrink-0 text-(--info)" />
      <span class="text-[11px] font-medium leading-none">
        Sample notification banner reacting in real time to semantic theme colors.
      </span>
    </div>
  </div>
</template>
