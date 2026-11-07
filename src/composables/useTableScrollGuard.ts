import { ref, readonly, onScopeDispose, getCurrentScope, type Ref } from 'vue'
import { getActivePinia } from 'pinia'
import { useKubernetesStore } from '../stores/kubernetesStore.ts'
import { setTickerPaused } from './useTicker.ts'

export interface TableScrollGuardOptions {
  idleMs?: number
  onScrollChange?: (scrolling: boolean) => void
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
  const scheduleTimeout =
    options?.scheduleTimeout ?? ((cb: () => void, ms: number) => setTimeout(cb, ms))
  const cancelTimeout =
    options?.cancelTimeout ??
    ((handle: unknown) => clearTimeout(handle as ReturnType<typeof setTimeout>))

  const isScrolling = ref(false)
  let idleTimerHandle: unknown | null = null

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

  function onScroll(): void {
    if (idleTimerHandle !== null) {
      cancelTimeout(idleTimerHandle)
      idleTimerHandle = null
    }

    if (!isScrolling.value) {
      isScrolling.value = true
      notify(true)
    }

    idleTimerHandle = scheduleTimeout(() => {
      idleTimerHandle = null
      isScrolling.value = false
      notify(false)
    }, idleMs)
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
