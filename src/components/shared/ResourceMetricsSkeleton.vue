<script setup lang="ts">
import Card from 'primevue/card'
import Skeleton from 'primevue/skeleton'

withDefaults(
  defineProps<{
    cards?: number
    gridClass?: string
  }>(),
  {
    cards: 5,
    gridClass: 'grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-5 gap-6'
  }
)
</script>

<template>
  <div data-component="resource-metrics-skeleton" :class="gridClass">
    <Card v-for="i in cards" :key="i">
      <template #content>
        <div class="flex items-center gap-4">
          <Skeleton
            width="2.5rem"
            height="2.5rem"
            class="bg-(--bg-hover)/40 rounded-[var(--ui-radius-control)] shrink-0"
          />
          <div class="flex-1 min-w-0 flex flex-col gap-2">
            <Skeleton
              width="60%"
              height="0.75rem"
              class="bg-(--bg-hover)/40 rounded-[var(--ui-radius-control)]"
            />
            <Skeleton
              width="40%"
              height="1.5rem"
              class="bg-(--bg-hover)/40 rounded-[var(--ui-radius-control)]"
            />
          </div>
        </div>
        <div class="mt-4">
          <Skeleton
            width="80%"
            height="0.75rem"
            class="bg-(--bg-hover)/40 rounded-[var(--ui-radius-control)]"
          />
        </div>
      </template>
    </Card>
  </div>
</template>
