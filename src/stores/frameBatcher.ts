export interface FrameBatcher<T> {
  push(kind: string, updates: T[]): void
  flush(): void
  clear(): void
}

export function createFrameBatcher<T>(
  apply: (kind: string, updates: T[]) => void,
  schedule?: (cb: () => void) => void
): FrameBatcher<T> {
  const pending = new Map<string, T[]>()
  let isScheduled = false

  const defaultSchedule = (cb: () => void) => {
    if (typeof requestAnimationFrame === 'function') {
      requestAnimationFrame(cb)
    } else {
      setTimeout(cb, 16)
    }
  }

  const runSchedule = schedule ?? defaultSchedule

  function drain(): void {
    isScheduled = false
    if (pending.size === 0) return

    const entries = Array.from(pending.entries())
    pending.clear()

    for (const [kind, updates] of entries) {
      if (updates.length > 0) {
        apply(kind, updates)
      }
    }
  }

  function push(kind: string, updates: T[]): void {
    if (!updates || updates.length === 0) return

    const existing = pending.get(kind)
    if (existing) {
      existing.push(...updates)
    } else {
      pending.set(kind, [...updates])
    }

    if (!isScheduled) {
      isScheduled = true
      runSchedule(drain)
    }
  }

  function flush(): void {
    drain()
  }

  function clear(): void {
    pending.clear()
    isScheduled = false
  }

  return {
    push,
    flush,
    clear
  }
}
