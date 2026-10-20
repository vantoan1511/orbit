import { ref, type Ref } from 'vue'

let activeSubscribers = 0
let tickerInterval: ReturnType<typeof setInterval> | null = null
const currentTimestamp: Ref<number> = ref(Date.now())
let isVisibilityListenerAttached = false

function isDocumentHidden(): boolean {
  if (typeof document === 'undefined') return false
  return document.visibilityState === 'hidden'
}

function startInterval() {
  if (tickerInterval || isDocumentHidden() || activeSubscribers <= 0) return
  currentTimestamp.value = Date.now()
  tickerInterval = setInterval(() => {
    currentTimestamp.value = Date.now()
  }, 1000)
}

function stopInterval() {
  if (tickerInterval) {
    clearInterval(tickerInterval)
    tickerInterval = null
  }
}

function handleVisibilityChange() {
  if (isDocumentHidden()) {
    stopInterval()
  } else if (activeSubscribers > 0) {
    currentTimestamp.value = Date.now()
    startInterval()
  }
}

function ensureVisibilityListener() {
  if (isVisibilityListenerAttached || typeof document === 'undefined') return
  document.addEventListener('visibilitychange', handleVisibilityChange)
  isVisibilityListenerAttached = true
}

export function useTicker() {
  ensureVisibilityListener()

  const subscribe = () => {
    activeSubscribers++
    if (activeSubscribers === 1) {
      ensureVisibilityListener()
      startInterval()
    }
  }

  const unsubscribe = () => {
    activeSubscribers = Math.max(0, activeSubscribers - 1)
    if (activeSubscribers === 0) {
      stopInterval()
    }
  }

  return {
    currentTimestamp,
    subscribe,
    unsubscribe
  }
}

/**
 * Resets internal state for test isolation.
 */
export function resetTickerForTesting() {
  stopInterval()
  activeSubscribers = 0
  if (isVisibilityListenerAttached && typeof document !== 'undefined') {
    document.removeEventListener('visibilitychange', handleVisibilityChange)
  }
  isVisibilityListenerAttached = false
  currentTimestamp.value = Date.now()
}
