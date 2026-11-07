export interface FrameBatcherOptions {
  /** Updates within `burstWindowMs` above which the batcher switches to throttled mode. Default 200. */
  burstThreshold?: number
  /** Sliding window used to measure burst rate. Default 250. */
  burstWindowMs?: number
  /** Flush interval for visible kinds while bursting. Default 250. */
  burstFlushMs?: number
  /** Flush interval for kinds not currently visible. Default 2000. */
  backgroundFlushMs?: number
  /** Maximum time (ms) to hold visible updates during continuous scrolling. Default 1000. */
  maxScrollHoldMs?: number
  /** Injectable clock + timers for tests. */
  now?: () => number
  scheduleFrame?: (cb: () => void) => void
  scheduleTimeout?: (cb: () => void, ms: number) => unknown
  cancelTimeout?: (handle: unknown) => void
}

export interface FrameBatcher<T> {
  push(kind: string, updates: T[]): void
  flush(kind?: string): void
  clear(): void
  /** Ref-counted visibility. Returns an unregister fn. Registering flushes that kind immediately. */
  registerVisibleKind(kind: string): () => void
  isBursting(): boolean
  setScrolling(scrolling: boolean): void
  isScrolling(): boolean
}

export function createFrameBatcher<T>(
  apply: (kind: string, updates: T[]) => void,
  options?: FrameBatcherOptions | ((cb: () => void) => void)
): FrameBatcher<T> {
  const opts: FrameBatcherOptions =
    typeof options === 'function' ? { scheduleFrame: options } : (options ?? {})

  const burstThreshold = opts.burstThreshold ?? 200
  const burstWindowMs = opts.burstWindowMs ?? 250
  const burstFlushMs = opts.burstFlushMs ?? 250
  const backgroundFlushMs = opts.backgroundFlushMs ?? 2000
  const maxScrollHoldMs = opts.maxScrollHoldMs ?? 1000

  const now =
    opts.now ??
    (() => (typeof performance !== 'undefined' && performance.now ? performance.now() : Date.now()))

  const scheduleFrame =
    opts.scheduleFrame ??
    ((cb: () => void) => {
      if (typeof requestAnimationFrame === 'function') {
        requestAnimationFrame(cb)
      } else {
        setTimeout(cb, 16)
      }
    })

  const scheduleTimeout =
    opts.scheduleTimeout ?? ((cb: () => void, ms: number) => setTimeout(cb, ms))

  const cancelTimeout =
    opts.cancelTimeout ??
    ((handle: unknown) => clearTimeout(handle as ReturnType<typeof setTimeout>))

  const pending = new Map<string, T[]>()
  const visibleCounts = new Map<string, number>()
  const updateEvents: Array<{ time: number; count: number }> = []

  let frameScheduled = false
  let burstTimerHandle: unknown | null = null
  let backgroundTimerHandle: unknown | null = null
  let scrolling = false
  let scrollHoldTimerHandle: unknown | null = null

  function isKindVisible(kind: string): boolean {
    return (visibleCounts.get(kind) ?? 0) > 0
  }

  function pruneRateWindow(currentTime: number): void {
    const cutoff = currentTime - burstWindowMs
    while (updateEvents.length > 0) {
      const first = updateEvents[0]
      if (first && first.time < cutoff) {
        updateEvents.shift()
      } else {
        break
      }
    }
  }

  function recordUpdates(count: number): void {
    const currentTime = now()
    pruneRateWindow(currentTime)
    updateEvents.push({ time: currentTime, count })
  }

  function isBursting(): boolean {
    const currentTime = now()
    pruneRateWindow(currentTime)
    let total = 0
    for (const ev of updateEvents) {
      total += ev.count
    }
    return total >= burstThreshold
  }

  function hasPendingHidden(): boolean {
    for (const kind of pending.keys()) {
      if (!isKindVisible(kind)) return true
    }
    return false
  }

  function hasPendingVisible(): boolean {
    for (const [kind, updates] of pending.entries()) {
      if (isKindVisible(kind) && updates.length > 0) return true
    }
    return false
  }

  function flushVisible(): void {
    burstTimerHandle = null
    if (scrollHoldTimerHandle !== null) {
      cancelTimeout(scrollHoldTimerHandle)
      scrollHoldTimerHandle = null
    }
    const visibleEntries: Array<[string, T[]]> = []
    for (const [kind, updates] of pending.entries()) {
      if (isKindVisible(kind) && updates.length > 0) {
        visibleEntries.push([kind, updates])
        pending.delete(kind)
      }
    }
    for (const [kind, updates] of visibleEntries) {
      apply(kind, updates)
    }
  }

  function flushHidden(): void {
    backgroundTimerHandle = null
    const hiddenEntries: Array<[string, T[]]> = []
    for (const [kind, updates] of pending.entries()) {
      if (!isKindVisible(kind) && updates.length > 0) {
        hiddenEntries.push([kind, updates])
        pending.delete(kind)
      }
    }
    for (const [kind, updates] of hiddenEntries) {
      apply(kind, updates)
    }
    if (hasPendingHidden() && backgroundTimerHandle === null) {
      backgroundTimerHandle = scheduleTimeout(flushHidden, backgroundFlushMs)
    }
  }

  function flushVisibleOnFrame(): void {
    frameScheduled = false
    if (scrolling) {
      if (scrollHoldTimerHandle === null && hasPendingVisible()) {
        scrollHoldTimerHandle = scheduleTimeout(flushVisible, maxScrollHoldMs)
      }
      return
    }
    if (isBursting()) {
      if (burstTimerHandle === null) {
        burstTimerHandle = scheduleTimeout(flushVisible, burstFlushMs)
      }
      return
    }
    flushVisible()
  }

  function push(kind: string, updates: T[]): void {
    if (!updates || updates.length === 0) return

    recordUpdates(updates.length)

    const existing = pending.get(kind)
    if (existing) {
      existing.push(...updates)
    } else {
      pending.set(kind, [...updates])
    }

    if (isKindVisible(kind)) {
      if (scrolling) {
        if (scrollHoldTimerHandle === null) {
          scrollHoldTimerHandle = scheduleTimeout(flushVisible, maxScrollHoldMs)
        }
      } else if (isBursting()) {
        if (burstTimerHandle === null) {
          burstTimerHandle = scheduleTimeout(flushVisible, burstFlushMs)
        }
      } else {
        if (!frameScheduled) {
          frameScheduled = true
          scheduleFrame(flushVisibleOnFrame)
        }
      }
    } else {
      if (backgroundTimerHandle === null) {
        backgroundTimerHandle = scheduleTimeout(flushHidden, backgroundFlushMs)
      }
    }
  }

  function flush(targetKind?: string): void {
    if (targetKind !== undefined) {
      const updates = pending.get(targetKind)
      if (updates && updates.length > 0) {
        pending.delete(targetKind)
        apply(targetKind, updates)
      }
      return
    }

    if (pending.size === 0) return
    const entries = Array.from(pending.entries())
    pending.clear()
    for (const [kind, updates] of entries) {
      if (updates.length > 0) {
        apply(kind, updates)
      }
    }
  }

  function clear(): void {
    pending.clear()
    updateEvents.length = 0
    frameScheduled = false
    if (burstTimerHandle !== null) {
      cancelTimeout(burstTimerHandle)
      burstTimerHandle = null
    }
    if (backgroundTimerHandle !== null) {
      cancelTimeout(backgroundTimerHandle)
      backgroundTimerHandle = null
    }
    if (scrollHoldTimerHandle !== null) {
      cancelTimeout(scrollHoldTimerHandle)
      scrollHoldTimerHandle = null
    }
  }

  function setScrolling(nextScrolling: boolean): void {
    if (scrolling === nextScrolling) return
    scrolling = nextScrolling
    if (scrolling) {
      if (hasPendingVisible() && scrollHoldTimerHandle === null) {
        scrollHoldTimerHandle = scheduleTimeout(flushVisible, maxScrollHoldMs)
      }
    } else {
      if (scrollHoldTimerHandle !== null) {
        cancelTimeout(scrollHoldTimerHandle)
        scrollHoldTimerHandle = null
      }
      if (hasPendingVisible()) {
        flushVisible()
      }
    }
  }

  function isScrolling(): boolean {
    return scrolling
  }

  function registerVisibleKind(kind: string): () => void {
    const currentCount = visibleCounts.get(kind) ?? 0
    visibleCounts.set(kind, currentCount + 1)

    // Registering flushes that kind immediately
    if (pending.has(kind)) {
      flush(kind)
    }

    if (!hasPendingHidden() && backgroundTimerHandle !== null) {
      cancelTimeout(backgroundTimerHandle)
      backgroundTimerHandle = null
    }

    let unregistered = false
    return () => {
      if (unregistered) return
      unregistered = true
      const count = (visibleCounts.get(kind) ?? 0) - 1
      if (count <= 0) {
        visibleCounts.delete(kind)
      } else {
        visibleCounts.set(kind, count)
      }
      if (hasPendingHidden() && backgroundTimerHandle === null) {
        backgroundTimerHandle = scheduleTimeout(flushHidden, backgroundFlushMs)
      }
    }
  }

  return {
    push,
    flush,
    clear,
    registerVisibleKind,
    isBursting,
    setScrolling,
    isScrolling
  }
}
