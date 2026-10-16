<script setup lang="ts">
import { useAppearance } from '@/composables/useAppearance'
import { UI_STYLE_DEFINITIONS } from '@/utils/appearanceConstants'
import { Check, Sparkles } from '@lucide/vue'
import Tag from 'primevue/tag'

const { uiStyle, setUiStyle } = useAppearance()
</script>

<template>
  <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3">
    <div
      v-for="style in UI_STYLE_DEFINITIONS"
      :key="style.id"
      role="button"
      tabindex="0"
      class="group relative flex flex-col p-3.5 rounded-lg border transition-all cursor-pointer select-none outline-none focus-visible:ring-2 focus-visible:ring-(--accent)"
      :class="[
        uiStyle === style.id
          ? 'border-(--accent) bg-(--bg-card) ring-1 ring-(--accent)'
          : 'border-(--border) bg-(--bg-hover)/30 hover:bg-(--bg-hover)/60 hover:border-(--border-strong)'
      ]"
      @click="setUiStyle(style.id)"
      @keydown.enter.space.prevent="setUiStyle(style.id)"
    >
      <!-- Header row with Badges and Selection Check -->
      <div class="flex items-center justify-between mb-2">
        <div class="flex items-center gap-1.5">
          <Sparkles class="w-4 h-4 text-muted-color group-hover:text-primary transition-colors" />
          <Tag
            v-if="style.badge"
            severity="secondary"
            :value="style.badge"
            class="text-[10px] uppercase font-semibold"
          />
        </div>
        <div
          v-if="uiStyle === style.id"
          class="w-5 h-5 rounded-full bg-(--accent) text-white flex items-center justify-center shrink-0"
        >
          <Check class="w-3 h-3 stroke-[3]" />
        </div>
      </div>

      <!-- Title & Description -->
      <span class="text-xs font-semibold text-primary">{{ style.name }}</span>
      <p class="text-[11px] text-muted-color mt-1 leading-snug">
        {{ style.description }}
      </p>
    </div>
  </div>
</template>
