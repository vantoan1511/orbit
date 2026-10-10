import { computed, onMounted, onUnmounted, ref, type ComputedRef, type Ref } from 'vue'
import {
  app,
  events,
  probe as nativeProbe,
  reconnect as nativeReconnect
} from '../services/nativeService.ts'
import { OrbitEvents } from '../types/events.ts'

export type EngineLinkState = 'connected' | 'reconnecting' | 'failed'

export const ENGINE_BACKOFF_SCHEDULE = [1000, 2000, 4000, 8000, 15000] as const
export const ENGINE_PROBE_INTERVAL_MS = 15000
export const MAX_RECONNECT_DURATION_MS = 60000

export interface EngineConnectionDependencies {
  isTauriEnvironment?: () => boolean
  isNeutralinoEnvironment?: () => boolean
  probe?: (timeoutMs?: number) => Promise<boolean>
  reconnect?: () => void
  restartProcess?: () => Promise<unknown>
  now?: () => number
}

export interface EngineConnectionManager {
  state: Ref<EngineLinkState>
  isReconnecting: ComputedRef<boolean>
  isFailed: ComputedRef<boolean>
  handleServerOffline: () => void
  handleEngineConnected: () => void
  retryNow: () => Promise<void>
  restartApp: () => Promise<void>
  tick: () => Promise<void>
  startWatchdog: () => void
  stopWatchdog: () => void
}

function defaultIsTauri(): boolean {
  return (
    typeof globalThis.window !== 'undefined' &&
    Boolean(
      (globalThis.window as unknown as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__ ||
      (globalThis.window as unknown as { __TAURI__?: unknown }).__TAURI__ ||
      (globalThis.window as unknown as { NL_PORT?: unknown }).NL_PORT
    )
  )
}

export function createEngineConnectionManager(
  deps: EngineConnectionDependencies = {}
): EngineConnectionManager {
  const isEnv = deps.isTauriEnvironment ?? deps.isNeutralinoEnvironment ?? defaultIsTauri
  const probeFn = deps.probe ?? nativeProbe
  const reconnectFn = deps.reconnect ?? nativeReconnect
  const restartFn = deps.restartProcess ?? (() => app.restartProcess())
  const nowFn = deps.now ?? (() => Date.now())

  const state = ref<EngineLinkState>('connected')
  let backoffIndex = 0
  let reconnectStartedAt: number | null = null
  let nextAttemptAt: number | null = null
  let isAttempting = false
  let watchdogInterval: ReturnType<typeof setInterval> | null = null

  const isReconnecting = computed(() => state.value === 'reconnecting')
  const isFailed = computed(() => state.value === 'failed')

  function triggerReconnect(): void {
    if (!isEnv()) return
    const now = nowFn()
    if (state.value !== 'reconnecting') {
      state.value = 'reconnecting'
      reconnectStartedAt = now
      backoffIndex = 0
    }
    const delay =
      ENGINE_BACKOFF_SCHEDULE[Math.min(backoffIndex, ENGINE_BACKOFF_SCHEDULE.length - 1)] ?? 15000
    nextAttemptAt = now + delay
  }

  function handleServerOffline(): void {
    if (!isEnv()) return
    triggerReconnect()
  }

  function handleEngineConnected(): void {
    state.value = 'connected'
    backoffIndex = 0
    reconnectStartedAt = null
    nextAttemptAt = null
    isAttempting = false
  }

  async function performReconnectAttempt(): Promise<void> {
    if (isAttempting) return
    isAttempting = true

    try {
      reconnectFn()
      backoffIndex++
      const now = nowFn()
      const delay =
        ENGINE_BACKOFF_SCHEDULE[Math.min(backoffIndex, ENGINE_BACKOFF_SCHEDULE.length - 1)] ?? 15000
      nextAttemptAt = now + delay
    } finally {
      isAttempting = false
    }
  }

  async function tick(): Promise<void> {
    if (!isEnv()) return
    const now = nowFn()

    if (state.value === 'reconnecting') {
      if (reconnectStartedAt !== null && now - reconnectStartedAt >= MAX_RECONNECT_DURATION_MS) {
        state.value = 'failed'
        return
      }

      if (nextAttemptAt !== null && now >= nextAttemptAt) {
        await performReconnectAttempt()
      }
    } else if (state.value === 'connected') {
      const healthy = await probeFn(2000)
      if (!healthy) {
        triggerReconnect()
      }
    }
  }

  async function retryNow(): Promise<void> {
    if (!isEnv()) return
    state.value = 'reconnecting'
    reconnectStartedAt = nowFn()
    backoffIndex = 0
    nextAttemptAt = null
    await performReconnectAttempt()
  }

  async function restartApp(): Promise<void> {
    await restartFn()
  }

  function startWatchdog(): void {
    if (watchdogInterval !== null || !isEnv()) return
    watchdogInterval = setInterval(() => {
      void tick()
    }, 2000)
  }

  function stopWatchdog(): void {
    if (watchdogInterval !== null) {
      clearInterval(watchdogInterval)
      watchdogInterval = null
    }
  }

  return {
    state,
    isReconnecting,
    isFailed,
    handleServerOffline,
    handleEngineConnected,
    retryNow,
    restartApp,
    tick,
    startWatchdog,
    stopWatchdog
  }
}

let sharedManager: EngineConnectionManager | null = null

export function useEngineConnection(deps: EngineConnectionDependencies = {}) {
  if (!sharedManager) {
    sharedManager = createEngineConnectionManager(deps)
  }
  const manager = sharedManager

  function onOffline() {
    manager.handleServerOffline()
  }

  function onConnected() {
    manager.handleEngineConnected()
  }

  function onVisibilityOrFocus() {
    if (typeof document !== 'undefined' && document.visibilityState === 'visible') {
      void manager.tick()
    }
  }

  onMounted(() => {
    void events.on(OrbitEvents.ServerOffline, onOffline)
    void events.on(OrbitEvents.EngineConnected, onConnected)

    if (typeof window !== 'undefined') {
      window.addEventListener('focus', onVisibilityOrFocus)
      document.addEventListener('visibilitychange', onVisibilityOrFocus)
    }

    manager.startWatchdog()
  })

  onUnmounted(() => {
    void events.off(OrbitEvents.ServerOffline, onOffline)
    void events.off(OrbitEvents.EngineConnected, onConnected)

    if (typeof window !== 'undefined') {
      window.removeEventListener('focus', onVisibilityOrFocus)
      document.removeEventListener('visibilitychange', onVisibilityOrFocus)
    }

    manager.stopWatchdog()
  })

  return {
    state: manager.state,
    isReconnecting: manager.isReconnecting,
    isFailed: manager.isFailed,
    retryNow: manager.retryNow,
    restartApp: manager.restartApp
  }
}
