import { ref, type Ref } from 'vue'

let activeSubscribers = 0
let tickerInterval: ReturnType<typeof setInterval> | null = null
const currentTimestamp: Ref<number> = ref(Date.now())

export function useTicker() {
  const subscribe = () => {
    activeSubscribers++
    if (activeSubscribers === 1 && !tickerInterval) {
      currentTimestamp.value = Date.now()
      tickerInterval = setInterval(() => {
        currentTimestamp.value = Date.now()
      }, 1000)
    }
  }

  const unsubscribe = () => {
    activeSubscribers = Math.max(0, activeSubscribers - 1)
    if (activeSubscribers === 0 && tickerInterval) {
      clearInterval(tickerInterval)
      tickerInterval = null
    }
  }

  return {
    currentTimestamp,
    subscribe,
    unsubscribe
  }
}
