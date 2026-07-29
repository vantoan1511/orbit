<script setup lang="ts">
import Skeleton from 'primevue/skeleton'

withDefaults(
  defineProps<{
    rows?: number
    columns?: number
  }>(),
  {
    rows: 5,
    columns: 6
  }
)

const getColumnWidth = (index: number, total: number) => {
  if (index === 1) return '30%'
  if (index === 2) return '15%'
  if (index === total) return '10%'
  return '18%'
}
</script>

<template>
  <div class="flex flex-col w-full divide-y divide-(--border)/40">
    <div
      v-for="r in rows"
      :key="r"
      class="flex items-center gap-4 py-3.5 px-4 w-full animate-pulse"
    >
      <div
        v-for="c in columns"
        :key="c"
        class="flex items-center"
        :style="{ width: getColumnWidth(c, columns) }"
      >
        <Skeleton
          :width="c === 2 ? '70%' : '85%'"
          height="1.25rem"
          class="bg-(--bg-hover)/40 rounded-md"
        />
      </div>
    </div>
  </div>
</template>
