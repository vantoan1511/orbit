<script setup lang="ts">
import { ref, watch, onMounted, onUnmounted } from 'vue'

const props = defineProps<{
  age?: string
}>()

const displayAge = ref(props.age || '-')
let interval: ReturnType<typeof setInterval>
let initialAgeSeconds = 0
let receivedAt = Date.now()

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

const updateDisplay = () => {
  if (
    !props.age ||
    props.age === '-' ||
    props.age === 'Unknown' ||
    (initialAgeSeconds === 0 && !props.age.endsWith('s'))
  ) {
    displayAge.value = props.age || '-'
    return
  }
  const elapsed = Math.floor((Date.now() - receivedAt) / 1000)
  displayAge.value = formatSeconds(initialAgeSeconds + elapsed)
}

watch(
  () => props.age,
  (newAge) => {
    if (newAge && newAge !== 'Unknown' && newAge !== '-') {
      // If the incoming age matches our current fake-counted display,
      // don't reset. This prevents losing precision when the backend
      // sends an update that matches our rounded value.
      if (newAge === displayAge.value) {
        return
      }

      initialAgeSeconds = parseAgeToSeconds(newAge)
      receivedAt = Date.now()
    } else {
      initialAgeSeconds = 0
    }
    updateDisplay()
  },
  { immediate: true }
)

onMounted(() => {
  interval = setInterval(updateDisplay, 1000)
})

onUnmounted(() => {
  clearInterval(interval)
})
</script>

<template>
  <span>{{ displayAge }}</span>
</template>
