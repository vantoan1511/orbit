import { ref, readonly, onScopeDispose, getCurrentScope, type Ref } from 'vue'
import { getActivePinia } from 'pinia'
import { useKubernetesStore } from '../stores/kubernetesStore.ts'
import { setTickerPaused } from './useTicker.ts'

export interface TableScrollGuardOptions {
  idleMs?: number
  onScrollChange?: (scrolling: boolean) => void
  now?: () => number
  scheduleTimeout?: (cb: () => void, ms: number) => unknown
  cancelTimeout?: (handle: unknown) => void
}

export interface TableScrollGuard {
  readonly isScrolling: Readonly<Ref<boolean>>
  onScroll: () => void
  stop: () => void
}

export function useTableScrollGuard(options?: TableScrollGuardOptions): TableScrollGuard {
  const idleMs = options?.idleMs ?? 150
  const now = options?.now ?? (() => Date.now())
  const scheduleTimeout =
    options?.scheduleTimeout ?? ((cb: () => void, ms: number) => setTimeout(cb, ms))
  const cancelTimeout =
    options?.cancelTimeout ??
    ((handle: unknown) => clearTimeout(handle as ReturnType<typeof setTimeout>))

  const isScrolling = ref(false)
  let idleTimerHandle: unknown | null = null
  let lastScrollAt = 0

  function notify(scrolling: boolean): void {
    if (options?.onScrollChange) {
      options.onScrollChange(scrolling)
    } else {
      setTickerPaused(scrolling)
      if (getActivePinia()) {
        const store = useKubernetesStore()
        if (typeof store.setScrolling === 'function') {
          store.setScrolling(scrolling)
        }
      }
    }
  }

  function stop(): void {
    if (idleTimerHandle !== null) {
      cancelTimeout(idleTimerHandle)
      idleTimerHandle = null
    }
    if (isScrolling.value) {
      isScrolling.value = false
      notify(false)
    }
  }

  function scheduleIdleCheck(delayMs: number): void {
    idleTimerHandle = scheduleTimeout(() => {
      idleTimerHandle = null
      const elapsed = now() - lastScrollAt
      if (elapsed < idleMs) {
        scheduleIdleCheck(idleMs - elapsed)
        return
      }
      isScrolling.value = false
      notify(false)
    }, delayMs)
  }

  function onScroll(): void {
    lastScrollAt = now()

    if (!isScrolling.value) {
      isScrolling.value = true
      notify(true)
    }

    if (idleTimerHandle === null) {
      scheduleIdleCheck(idleMs)
    }
  }

  if (getCurrentScope()) {
    onScopeDispose(() => {
      stop()
    })
  }

  return {
    isScrolling: readonly(isScrolling),
    onScroll,
    stop
  }
}
