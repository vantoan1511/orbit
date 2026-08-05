<script setup lang="ts">
import type { Component } from 'vue'

export interface EventListItem {
  uid?: string
  icon: Component
  iconClass: string
  label: string
  message: string
  time: string
}

defineProps<{
  title: string
  to: string
  items: EventListItem[]
}>()
</script>

<template>
  <Card>
    <template #title>
      <div class="flex items-center justify-between">
        <div class="text-sm font-semibold text-primary uppercase tracking-wider">
          {{ title }}
        </div>
        <router-link
          :to="to"
          class="text-xs text-muted-color hover:underline flex items-center gap-1 font-medium"
        >
          <span>View all</span>
        </router-link>
      </div>
    </template>
    <template #content>
      <div class="divide-y divide-dashed divide-surface-500">
        <div
          v-for="(item, index) in items"
          :key="item.uid || index"
          class="py-3 flex items-start gap-3.5 first:pt-0 last:pb-0"
        >
          <div class="p-2 rounded-lg mt-0.5 shrink-0" :class="item.iconClass">
            <component :is="item.icon" class="w-4 h-4" />
          </div>
          <div class="flex-1 min-w-0">
            <div class="flex items-center justify-between gap-2">
              <span class="text-sm font-semibold text-primary">{{ item.label }}</span>
              <span class="text-xs text-muted-color font-mono whitespace-nowrap">{{
                item.time
              }}</span>
            </div>
            <p class="text-sm text-muted-color mt-1 truncate">{{ item.message }}</p>
          </div>
        </div>
      </div>
    </template>
  </Card>
</template>
