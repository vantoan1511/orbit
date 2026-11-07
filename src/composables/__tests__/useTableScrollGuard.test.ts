import assert from 'node:assert/strict'
import test from 'node:test'
import { effectScope } from 'vue'
import { useTableScrollGuard } from '../useTableScrollGuard.ts'

function createFakeTimerEnvironment() {
  let currentTime = 0
  let nextTimerId = 1
  const timers = new Map<number, { dueTime: number; cb: () => void }>()

  const scheduleTimeout = (cb: () => void, ms: number) => {
    const id = nextTimerId++
    timers.set(id, { dueTime: currentTime + ms, cb })
    return id
  }

  const cancelTimeout = (id: unknown) => {
    timers.delete(id as number)
  }

  const advanceTime = (ms: number) => {
    currentTime += ms
    while (true) {
      let earliestId: number | null = null
      let earliestTime = Infinity
      for (const [id, t] of timers.entries()) {
        if (t.dueTime <= currentTime && t.dueTime < earliestTime) {
          earliestTime = t.dueTime
          earliestId = id
        }
      }
      if (earliestId === null) break
      const timer = timers.get(earliestId)!
      timers.delete(earliestId)
      timer.cb()
    }
  }

  return {
    scheduleTimeout,
    cancelTimeout,
    advanceTime,
    getPendingCount: () => timers.size
  }
}

test('transitions isScrolling to true immediately on scroll and resets to false after idleMs', () => {
  const env = createFakeTimerEnvironment()
  const events: boolean[] = []

  const guard = useTableScrollGuard({
    idleMs: 150,
    scheduleTimeout: env.scheduleTimeout,
    cancelTimeout: env.cancelTimeout,
    onScrollChange: (scrolling) => events.push(scrolling)
  })

  assert.equal(guard.isScrolling.value, false)
  assert.equal(events.length, 0)

  // First scroll
  guard.onScroll()
  assert.equal(guard.isScrolling.value, true)
  assert.deepEqual(events, [true])

  // Advance 100ms (< 150ms)
  env.advanceTime(100)
  assert.equal(guard.isScrolling.value, true)
  assert.deepEqual(events, [true])

  // Advance remaining 50ms
  env.advanceTime(50)
  assert.equal(guard.isScrolling.value, false)
  assert.deepEqual(events, [true, false])
})

test('debounces consecutive scroll events and resets cleanly on scope dispose', () => {
  const env = createFakeTimerEnvironment()
  const events: boolean[] = []

  let guard!: ReturnType<typeof useTableScrollGuard>
  const scope = effectScope()

  scope.run(() => {
    guard = useTableScrollGuard({
      idleMs: 150,
      scheduleTimeout: env.scheduleTimeout,
      cancelTimeout: env.cancelTimeout,
      onScrollChange: (scrolling) => events.push(scrolling)
    })
  })

  // Fire scroll multiple times in rapid succession
  guard.onScroll()
  assert.equal(guard.isScrolling.value, true)
  assert.deepEqual(events, [true])

  env.advanceTime(80)
  guard.onScroll() // resets 150ms timer
  assert.equal(guard.isScrolling.value, true)
  assert.deepEqual(events, [true], 'should not re-emit true on consecutive events')

  env.advanceTime(80)
  assert.equal(
    guard.isScrolling.value,
    true,
    'still scrolling 160ms from start because of debounce'
  )

  // Dispose scope while still scrolling
  scope.stop()
  assert.equal(guard.isScrolling.value, false, 'must reset to false on scope dispose')
  assert.deepEqual(events, [true, false], 'must emit false when disposed during scroll')
  assert.equal(env.getPendingCount(), 0, 'pending timers must be cancelled on dispose')
})
