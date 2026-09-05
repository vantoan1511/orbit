<script setup lang="ts">
import { useTicker } from '@/composables/useTicker'
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'

const props = defineProps<{
  age?: string
}>()

const { currentTimestamp, subscribe, unsubscribe } = useTicker()

let initialAgeSeconds = 0
let receivedAt = Date.now()
const lastAgeProp = ref(props.age)

const parseAgeToSeconds = (ageStr?: string): number => {
  if (!ageStr) return 0
  const match = ageStr.trim().match(/^(\d+)([smhd])$/)
  if (!match) return 0
  const val = parseInt(match[1] || '0', 10)
  const unit = match[2] || 's'
  switch (unit) {
    case 's':
      return val
    case 'm':
      return val * 60
    case 'h':
      return val * 3600
    case 'd':
      return val * 86400
    default:
      return 0
  }
}

const formatSeconds = (seconds: number): string => {
  if (seconds < 0) return '0s'
  if (seconds < 60) return `${seconds}s`
  if (seconds < 3600) return `${Math.floor(seconds / 60)}m`
  if (seconds < 86400) return `${Math.floor(seconds / 3600)}h`
  return `${Math.floor(seconds / 86400)}d`
}

watch(
  () => props.age,
  (newAge) => {
    if (newAge && newAge !== 'Unknown' && newAge !== '-') {
      // If incoming age matches our computed value, avoid resetting timestamp
      if (newAge !== lastAgeProp.value) {
        lastAgeProp.value = newAge
        initialAgeSeconds = parseAgeToSeconds(newAge)
        receivedAt = Date.now()
      }
    } else {
      lastAgeProp.value = newAge
      initialAgeSeconds = 0
    }
  },
  { immediate: true }
)

const displayAge = computed(() => {
  if (
    !props.age ||
    props.age === '-' ||
    props.age === 'Unknown' ||
    (initialAgeSeconds === 0 && !props.age.endsWith('s'))
  ) {
    return props.age || '-'
  }
  const elapsed = Math.floor((currentTimestamp.value - receivedAt) / 1000)
  return formatSeconds(initialAgeSeconds + elapsed)
})

onMounted(() => {
  subscribe()
})

onUnmounted(() => {
  unsubscribe()
})
</script>

<template>
  <span>{{ displayAge }}</span>
</template>
