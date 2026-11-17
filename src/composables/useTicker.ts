import { ref, type Ref } from 'vue'

let activeSubscribers = 0
let activeMinuteSubscribers = 0
let tickerInterval: ReturnType<typeof setInterval> | null = null
let minuteTickerInterval: ReturnType<typeof setInterval> | null = null
const currentTimestamp: Ref<number> = ref(Date.now())
const currentMinuteTimestamp: Ref<number> = ref(Date.now())
let isVisibilityListenerAttached = false
let isTickerPaused = false

function isDocumentHidden(): boolean {
  if (typeof document === 'undefined') return false
  return document.visibilityState === 'hidden'
}

function startInterval() {
  if (tickerInterval || isDocumentHidden() || activeSubscribers <= 0) return
  currentTimestamp.value = Date.now()
  tickerInterval = setInterval(() => {
    if (isTickerPaused) return
    currentTimestamp.value = Date.now()
  }, 1000)
}

function stopInterval() {
  if (tickerInterval) {
    clearInterval(tickerInterval)
    tickerInterval = null
  }
}

function startMinuteInterval() {
  if (minuteTickerInterval || isDocumentHidden() || activeMinuteSubscribers <= 0) return
  currentMinuteTimestamp.value = Date.now()
  minuteTickerInterval = setInterval(() => {
    if (isTickerPaused) return
    currentMinuteTimestamp.value = Date.now()
  }, 60_000)
}

function stopMinuteInterval() {
  if (minuteTickerInterval) {
    clearInterval(minuteTickerInterval)
    minuteTickerInterval = null
  }
}

function handleVisibilityChange() {
  if (isDocumentHidden()) {
    stopInterval()
    stopMinuteInterval()
    return
  }
  if (activeSubscribers > 0) {
    currentTimestamp.value = Date.now()
    startInterval()
  }
  if (activeMinuteSubscribers > 0) {
    currentMinuteTimestamp.value = Date.now()
    startMinuteInterval()
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

  const subscribeMinute = () => {
    activeMinuteSubscribers++
    if (activeMinuteSubscribers === 1) {
      ensureVisibilityListener()
      startMinuteInterval()
    }
  }

  const unsubscribeMinute = () => {
    activeMinuteSubscribers = Math.max(0, activeMinuteSubscribers - 1)
    if (activeMinuteSubscribers === 0) {
      stopMinuteInterval()
    }
  }

  return {
    currentTimestamp,
    currentMinuteTimestamp,
    subscribe,
    unsubscribe,
    subscribeMinute,
    unsubscribeMinute
  }
}

/**
 * Resets internal state for test isolation.
 */
export function resetTickerForTesting() {
  stopInterval()
  stopMinuteInterval()
  activeSubscribers = 0
  activeMinuteSubscribers = 0
  isTickerPaused = false
  if (isVisibilityListenerAttached && typeof document !== 'undefined') {
    document.removeEventListener('visibilitychange', handleVisibilityChange)
  }
  isVisibilityListenerAttached = false
  const now = Date.now()
  currentTimestamp.value = now
  currentMinuteTimestamp.value = now
}

export function getSubscriberCountForTesting(): number {
  return activeSubscribers
}

export function getMinuteSubscriberCountForTesting(): number {
  return activeMinuteSubscribers
}

export function setTickerPaused(paused: boolean) {
  if (isTickerPaused === paused) return
  isTickerPaused = paused
  if (!isTickerPaused && !isDocumentHidden()) {
    const now = Date.now()
    if (activeSubscribers > 0) {
      currentTimestamp.value = now
    }
    if (activeMinuteSubscribers > 0) {
      currentMinuteTimestamp.value = now
    }
  }
}

export function isTickerPausedForTesting(): boolean {
  return isTickerPaused
}
