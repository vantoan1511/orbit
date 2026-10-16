<script setup lang="ts">
import { useAppearance } from '@/composables/useAppearance'
import type { ThemeMode } from '@/types/appearance'
import { Check, Laptop, Moon, Sun } from '@lucide/vue'

const { themeMode, setThemeMode, systemIsDark } = useAppearance()

interface ThemeOption {
  id: ThemeMode
  name: string
  description: string
  icon: typeof Moon
}

const themeOptions: ThemeOption[] = [
  {
    id: 'dark',
    name: 'Dark',
    description: 'Noir IDE dark palette, optimized for low-light environments.',
    icon: Moon
  },
  {
    id: 'light',
    name: 'Light',
    description: 'Clean high-contrast light palette with crisp boundaries.',
    icon: Sun
  },
  {
    id: 'system',
    name: 'System',
    description: 'Automatically synchronizes with your operating system preference.',
    icon: Laptop
  }
]
</script>

<template>
  <div class="grid grid-cols-1 sm:grid-cols-3 gap-3">
    <div
      v-for="opt in themeOptions"
      :key="opt.id"
      role="button"
      tabindex="0"
      class="group relative flex flex-col text-left p-3.5 rounded-lg border transition-all cursor-pointer select-none outline-none focus-visible:ring-2 focus-visible:ring-(--accent)"
      :class="[
        themeMode === opt.id
          ? 'border-(--accent) bg-(--bg-card) ring-1 ring-(--accent)'
          : 'border-(--border) bg-(--bg-hover)/30 hover:bg-(--bg-hover)/60 hover:border-(--border-strong)'
      ]"
      @click="setThemeMode(opt.id)"
      @keydown.enter.space.prevent="setThemeMode(opt.id)"
    >
      <!-- Header row with Icon and Selection Check -->
      <div class="flex items-center justify-between mb-2">
        <div
          class="w-8 h-8 rounded-md flex items-center justify-center transition-colors"
          :class="[
            themeMode === opt.id
              ? 'bg-(--accent)/15 text-(--accent)'
              : 'bg-(--bg-hover) text-muted-color group-hover:text-primary'
          ]"
        >
          <component :is="opt.icon" class="w-4 h-4" />
        </div>
        <div
          v-if="themeMode === opt.id"
          class="w-5 h-5 rounded-full bg-(--accent) text-white flex items-center justify-center"
        >
          <Check class="w-3 h-3 stroke-[3]" />
        </div>
      </div>

      <!-- Title & Details -->
      <div class="flex items-center gap-1.5">
        <span class="text-xs font-semibold text-primary">{{ opt.name }}</span>
        <span v-if="opt.id === 'system'" class="text-[10px] text-muted-color">
          ({{ systemIsDark ? 'Dark' : 'Light' }})
        </span>
      </div>
      <p class="text-[11px] text-muted-color mt-1 leading-snug">
        {{ opt.description }}
      </p>
    </div>
  </div>
</template>
