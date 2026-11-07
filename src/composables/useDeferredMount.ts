import { getCurrentScope, onScopeDispose, ref, type Ref } from 'vue'

export interface UseDeferredMountOptions {
  /** Custom frame scheduler (defaults to requestAnimationFrame with setTimeout(16) fallback) */
  scheduleFrame?: (cb: () => void) => unknown
  /** Custom frame canceller (defaults to cancelAnimationFrame / clearTimeout) */
  cancelFrame?: (handle: unknown) => void
}

export interface UseDeferredMountReturn {
  /** False on initial synchronous mount frame; flips to true on next animation frame */
  isHydrated: Ref<boolean>
}

/**
 * Returns a reactive `isHydrated` ref that is synchronously `false` when called,
 * deferring the true state to the next animation frame.
 * This allows heavy data tables and child components to yield the initial paint frame to
 * lightweight skeleton loaders and instant tab bar updates.
 */
export function useDeferredMount(options?: UseDeferredMountOptions): UseDeferredMountReturn {
  const isHydrated = ref(false)

  const schedule =
    options?.scheduleFrame ??
    ((cb: () => void) => {
      if (typeof window !== 'undefined' && typeof window.requestAnimationFrame === 'function') {
        return window.requestAnimationFrame(cb)
      }
      return setTimeout(cb, 16)
    })

  const cancel =
    options?.cancelFrame ??
    ((handle: unknown) => {
      if (typeof window !== 'undefined' && typeof window.cancelAnimationFrame === 'function') {
        window.cancelAnimationFrame(handle as number)
      } else {
        clearTimeout(handle as ReturnType<typeof setTimeout>)
      }
    })

  const handle = schedule(() => {
    isHydrated.value = true
  })

  if (getCurrentScope()) {
    onScopeDispose(() => {
      cancel(handle)
    })
  }

  return {
    isHydrated
  }
}
