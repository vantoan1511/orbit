import { getCurrentScope, onScopeDispose, ref, type Ref } from 'vue'

export interface UseDeferredMountOptions {
  /** Custom frame scheduler (defaults to requestAnimationFrame with setTimeout(16) fallback) */
  scheduleFrame?: (cb: () => void) => unknown
  /** Custom frame canceller (defaults to cancelAnimationFrame / clearTimeout) */
  cancelFrame?: (handle: unknown) => void
}

export interface UseDeferredMountReturn {
  /** False on initial synchronous mount frame; flips to true after second animation frame */
  isHydrated: Ref<boolean>
}

/**
 * Returns a reactive `isHydrated` ref that is synchronously `false` when called,
 * deferring the true state across two animation frames.
 *
 * Browsers execute `requestAnimationFrame` callbacks before the next paint. Scheduling
 * through two consecutive animation frames guarantees that the browser paints at least
 * one frame containing the skeleton loader and tab chrome before heavy data table
 * rows and DOM elements mount.
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

  let pending: unknown = null
  let disposed = false

  pending = schedule(() => {
    if (disposed) return
    pending = schedule(() => {
      if (!disposed) {
        isHydrated.value = true
      }
    })
  })

  if (getCurrentScope()) {
    onScopeDispose(() => {
      disposed = true
      cancel(pending)
    })
  }

  return {
    isHydrated
  }
}
