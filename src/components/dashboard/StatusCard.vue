<script setup lang="ts">
import type { Component } from 'vue'

defineProps<{
  label: string
  value: string | number
  valueExtra?: string
  subtext: string
  icon: Component
  iconBgClass: string
  iconClass: string
  valueClass?: string
}>()
</script>

<template>
  <div class="bg-(--bg-card) p-5 flex items-center gap-5">
    <div
      class="w-12 h-12 rounded-xl flex items-center justify-center shrink-0"
      :class="iconBgClass"
    >
      <component :is="icon" class="w-6 h-6" :class="iconClass" />
    </div>
    <div class="flex-1 min-w-0">
      <div class="text-xs font-semibold text-(--text-muted) uppercase tracking-wider">
        {{ label }}
      </div>
      <div class="text-2xl font-bold mt-1" :class="valueClass ?? iconClass">
        {{ value
        }}<span v-if="valueExtra" class="text-base text-(--text-muted) font-normal">
          {{ valueExtra }}
        </span>
      </div>
      <div class="text-xs text-(--text-muted) mt-0.5 truncate">{{ subtext }}</div>
    </div>
  </div>
</template>
