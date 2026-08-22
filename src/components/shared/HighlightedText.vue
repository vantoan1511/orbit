<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(
  defineProps<{
    text: string
    query?: string
    isRegex?: boolean
  }>(),
  {
    query: '',
    isRegex: false
  }
)

interface TextSegment {
  text: string
  isMatch: boolean
}

const segments = computed<TextSegment[]>(() => {
  if (!props.text) return []
  if (!props.query) return [{ text: props.text, isMatch: false }]

  try {
    const pattern = props.isRegex
      ? props.query
      : props.query.replace(/[-\/\\^$*+?.()|[\]{}]/g, '\\$&')

    if (!pattern) return [{ text: props.text, isMatch: false }]

    const regex = new RegExp(pattern, 'gi')
    const result: TextSegment[] = []
    let lastIndex = 0
    let match: RegExpExecArray | null

    while ((match = regex.exec(props.text)) !== null) {
      if (match[0].length === 0) {
        regex.lastIndex = match.index + 1
        continue
      }

      if (match.index > lastIndex) {
        result.push({
          text: props.text.slice(lastIndex, match.index),
          isMatch: false
        })
      }

      result.push({
        text: match[0],
        isMatch: true
      })

      lastIndex = match.index + match[0].length
    }

    if (lastIndex < props.text.length) {
      result.push({
        text: props.text.slice(lastIndex),
        isMatch: false
      })
    }

    return result.length > 0 ? result : [{ text: props.text, isMatch: false }]
  } catch {
    return [{ text: props.text, isMatch: false }]
  }
})
</script>

<template>
  <span>
    <template v-for="(segment, idx) in segments" :key="idx">
      <mark
        v-if="segment.isMatch"
        class="bg-amber-500/30 text-inherit font-bold rounded-sm px-0.5"
        >{{ segment.text }}</mark
      >
      <template v-else>{{ segment.text }}</template>
    </template>
  </span>
</template>
