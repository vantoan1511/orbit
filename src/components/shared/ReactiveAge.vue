<script setup lang="ts">
import { useTicker } from '@/composables/useTicker'
import {
  formatAgeSeconds,
  parseAgeToSeconds,
  resolveAgeTickCadence,
  type AgeTickCadence
} from '@/utils/reactiveAge'
import { computed, onActivated, onDeactivated, onMounted, onUnmounted, ref, watch } from 'vue'

const props = defineProps<{
  age?: string
}>()

const {
  currentTimestamp,
  currentMinuteTimestamp,
  subscribe,
  unsubscribe,
  subscribeMinute,
  unsubscribeMinute
} = useTicker()

let initialAgeSeconds = 0
let receivedAt = Date.now()
const lastAgeProp = ref(props.age)
let activeCadence: AgeTickCadence = 'none'
let isActive = false

function applySubscription(nextCadence: AgeTickCadence) {
  if (!isActive) return
  if (activeCadence === nextCadence) return

  if (activeCadence === 'second') {
    unsubscribe()
  } else if (activeCadence === 'minute') {
    unsubscribeMinute()
  }

  activeCadence = nextCadence

  if (activeCadence === 'second') {
    subscribe()
  } else if (activeCadence === 'minute') {
    subscribeMinute()
  }
}

function handleSubscribe() {
  isActive = true
  applySubscription(resolveAgeTickCadence(props.age))
}

function handleUnsubscribe() {
  isActive = false
  if (activeCadence === 'second') {
    unsubscribe()
  } else if (activeCadence === 'minute') {
    unsubscribeMinute()
  }
  activeCadence = 'none'
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
    applySubscription(resolveAgeTickCadence(newAge))
  },
  { immediate: true }
)

const displayAge = computed(() => {
  const cadence = resolveAgeTickCadence(props.age)
  if (cadence === 'none') {
    return props.age || '-'
  }
  const now = cadence === 'second' ? currentTimestamp.value : currentMinuteTimestamp.value
  const elapsed = Math.floor((now - receivedAt) / 1000)
  return formatAgeSeconds(initialAgeSeconds + elapsed)
})

onMounted(() => {
  handleSubscribe()
})

onActivated(() => {
  handleSubscribe()
})

onDeactivated(() => {
  handleUnsubscribe()
})

onUnmounted(() => {
  handleUnsubscribe()
})
</script>

<template>
  <span>{{ displayAge }}</span>
</template>
