import assert from 'node:assert/strict'
import test from 'node:test'
import { createFrameBatcher } from '../frameBatcher.ts'

function createFakeEnvironment() {
  let currentTime = 0
  const frameQueue: Array<() => void> = []
  let nextTimerId = 1
  const timers = new Map<number, { dueTime: number; cb: () => void }>()

  const now = () => currentTime
  const scheduleFrame = (cb: () => void) => {
    frameQueue.push(cb)
  }
  const scheduleTimeout = (cb: () => void, ms: number) => {
    const id = nextTimerId++
    timers.set(id, { dueTime: currentTime + ms, cb })
    return id
  }
  const cancelTimeout = (id: unknown) => {
    timers.delete(id as number)
  }

  const runFrames = () => {
    const queue = [...frameQueue]
    frameQueue.length = 0
    for (const cb of queue) {
      cb()
    }
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
    now,
    scheduleFrame,
    scheduleTimeout,
    cancelTimeout,
    runFrames,
    advanceTime,
    getPendingTimersCount: () => timers.size,
    getPendingFramesCount: () => frameQueue.length
  }
}

test('frameBatcher: quiet visible kind flushes on next frame', () => {
  const env = createFakeEnvironment()
  const calls: Array<{ kind: string; updates: number[] }> = []

  const batcher = createFrameBatcher<number>(
    (kind, updates) => {
      calls.push({ kind, updates })
    },
    {
      now: env.now,
      scheduleFrame: env.scheduleFrame,
      scheduleTimeout: env.scheduleTimeout,
      cancelTimeout: env.cancelTimeout
    }
  )

  batcher.registerVisibleKind('Pod')

  batcher.push('Pod', [1])
  batcher.push('Pod', [2])
  batcher.push('Pod', [3])

  assert.equal(calls.length, 0, 'apply must not be called synchronously before scheduled frame')
  assert.equal(env.getPendingFramesCount(), 1, 'one frame should be scheduled')

  env.runFrames()

  assert.equal(calls.length, 1)
  assert.equal(calls[0]?.kind, 'Pod')
  assert.deepEqual(calls[0]?.updates, [1, 2, 3])
})

test('frameBatcher: burst coalesces into burstFlushMs windows', () => {
  const env = createFakeEnvironment()
  const calls: Array<{ kind: string; updates: number[] }> = []

  const batcher = createFrameBatcher<number>(
    (kind, updates) => {
      calls.push({ kind, updates })
    },
    {
      burstThreshold: 200,
      burstWindowMs: 250,
      burstFlushMs: 250,
      now: env.now,
      scheduleFrame: env.scheduleFrame,
      scheduleTimeout: env.scheduleTimeout,
      cancelTimeout: env.cancelTimeout
    }
  )

  batcher.registerVisibleKind('Pod')

  // Push 1000 single updates over simulated 250 ms
  for (let i = 0; i < 1000; i++) {
    batcher.push('Pod', [i])
    env.advanceTime(0.25)
  }

  env.runFrames()
  env.advanceTime(250)

  // Must coalesce into <= 2 apply calls for the window
  assert.ok(calls.length >= 1 && calls.length <= 2, `expected <= 2 calls, got ${calls.length}`)

  // Total updates across calls must equal 1000 and preserve order
  const allUpdates = calls.flatMap((c) => c.updates)
  assert.equal(allUpdates.length, 1000)
  for (let i = 0; i < 1000; i++) {
    assert.equal(allUpdates[i], i)
  }
})

test('frameBatcher: exits burst mode when rate drops', () => {
  const env = createFakeEnvironment()
  const calls: Array<{ kind: string; updates: number[] }> = []

  const batcher = createFrameBatcher<number>(
    (kind, updates) => {
      calls.push({ kind, updates })
    },
    {
      burstThreshold: 200,
      burstWindowMs: 250,
      burstFlushMs: 250,
      now: env.now,
      scheduleFrame: env.scheduleFrame,
      scheduleTimeout: env.scheduleTimeout,
      cancelTimeout: env.cancelTimeout
    }
  )

  batcher.registerVisibleKind('Pod')

  // Generate a burst
  for (let i = 0; i < 250; i++) {
    batcher.push('Pod', [i])
  }
  assert.equal(batcher.isBursting(), true)

  env.advanceTime(300)
  env.runFrames()
  calls.length = 0

  // Advance 1 second so sliding rate window expires
  env.advanceTime(1000)

  // Push 1 update
  batcher.push('Pod', [9999])
  assert.equal(batcher.isBursting(), false, 'should no longer be bursting')

  // Should flush on next frame
  assert.equal(calls.length, 0)
  env.runFrames()
  assert.equal(calls.length, 1)
  assert.deepEqual(calls[0]?.updates, [9999])
})

test('frameBatcher: hidden kind flushes only on background interval', () => {
  const env = createFakeEnvironment()
  const calls: Array<{ kind: string; updates: string[] }> = []

  const batcher = createFrameBatcher<string>(
    (kind, updates) => {
      calls.push({ kind, updates })
    },
    {
      backgroundFlushMs: 2000,
      now: env.now,
      scheduleFrame: env.scheduleFrame,
      scheduleTimeout: env.scheduleTimeout,
      cancelTimeout: env.cancelTimeout
    }
  )

  // 'Secret' is NOT registered as visible
  batcher.push('Secret', ['secret-1'])

  // Frame queue does not flush hidden kind
  env.runFrames()
  assert.equal(calls.length, 0)

  // At 1999 ms, still 0 calls
  env.advanceTime(1999)
  assert.equal(calls.length, 0)

  // At 2000 ms, background timer fires
  env.advanceTime(1)
  assert.equal(calls.length, 1)
  assert.equal(calls[0]?.kind, 'Secret')
  assert.deepEqual(calls[0]?.updates, ['secret-1'])
})

test('frameBatcher: registerVisibleKind flushes pending immediately', () => {
  const env = createFakeEnvironment()
  const calls: Array<{ kind: string; updates: string[] }> = []

  const batcher = createFrameBatcher<string>(
    (kind, updates) => {
      calls.push({ kind, updates })
    },
    {
      backgroundFlushMs: 2000,
      now: env.now,
      scheduleFrame: env.scheduleFrame,
      scheduleTimeout: env.scheduleTimeout,
      cancelTimeout: env.cancelTimeout
    }
  )

  batcher.push('Secret', ['s1', 's2'])
  assert.equal(calls.length, 0)

  // Advance some time, still before background interval
  env.advanceTime(500)
  assert.equal(calls.length, 0)

  // Registering visible kind flushes pending immediately
  const unregister = batcher.registerVisibleKind('Secret')
  assert.equal(calls.length, 1)
  assert.equal(calls[0]?.kind, 'Secret')
  assert.deepEqual(calls[0]?.updates, ['s1', 's2'])

  unregister()
})

test('frameBatcher: ref-counted unregister maintains visibility until last unregister', () => {
  const env = createFakeEnvironment()
  const calls: Array<{ kind: string; updates: number[] }> = []

  const batcher = createFrameBatcher<number>(
    (kind, updates) => {
      calls.push({ kind, updates })
    },
    {
      backgroundFlushMs: 2000,
      now: env.now,
      scheduleFrame: env.scheduleFrame,
      scheduleTimeout: env.scheduleTimeout,
      cancelTimeout: env.cancelTimeout
    }
  )

  const unreg1 = batcher.registerVisibleKind('Pod')
  const unreg2 = batcher.registerVisibleKind('Pod')

  // Unregister once -> still visible
  unreg1()

  batcher.push('Pod', [10])
  env.runFrames()
  assert.equal(calls.length, 1, 'should flush on frame since ref-count is still 1')
  calls.length = 0

  // Unregister second time -> now hidden
  unreg2()

  batcher.push('Pod', [20])
  env.runFrames()
  assert.equal(calls.length, 0, 'should NOT flush on frame since it is now hidden')

  env.advanceTime(2000)
  assert.equal(calls.length, 1, 'should flush on background interval')
  assert.deepEqual(calls[0]?.updates, [20])
})

test('frameBatcher: unregister schedules background flush if pending updates exist', () => {
  const env = createFakeEnvironment()
  const calls: Array<{ kind: string; updates: number[] }> = []

  const batcher = createFrameBatcher<number>(
    (kind, updates) => {
      calls.push({ kind, updates })
    },
    {
      burstThreshold: 200,
      burstWindowMs: 250,
      burstFlushMs: 250,
      backgroundFlushMs: 2000,
      now: env.now,
      scheduleFrame: env.scheduleFrame,
      scheduleTimeout: env.scheduleTimeout,
      cancelTimeout: env.cancelTimeout
    }
  )

  const unregister = batcher.registerVisibleKind('Pod')
  // Push 250 updates so it's bursting and pending
  for (let i = 0; i < 250; i++) {
    batcher.push('Pod', [i])
  }

  // Now unregister before burst timer fires
  unregister()

  // Advance time past background flush interval
  env.advanceTime(2000)
  assert.equal(calls.length, 1)
  assert.equal(calls[0]?.updates.length, 250)
})

test('frameBatcher: clear() cancels pending timers and drops updates without invoking apply', () => {
  const env = createFakeEnvironment()
  const calls: Array<{ kind: string; updates: number[] }> = []

  const batcher = createFrameBatcher<number>(
    (kind, updates) => {
      calls.push({ kind, updates })
    },
    {
      backgroundFlushMs: 2000,
      now: env.now,
      scheduleFrame: env.scheduleFrame,
      scheduleTimeout: env.scheduleTimeout,
      cancelTimeout: env.cancelTimeout
    }
  )

  batcher.push('Secret', [1, 2])
  assert.equal(env.getPendingTimersCount(), 1)

  batcher.clear()
  assert.equal(env.getPendingTimersCount(), 0, 'timers should be cancelled')

  // Even if time advances 5 seconds, nothing applies
  env.advanceTime(5000)
  env.runFrames()
  assert.equal(calls.length, 0)
})

test('frameBatcher: flush(kind) drains only the specified kind synchronously', () => {
  const calls: Array<{ kind: string; updates: number[] }> = []

  const batcher = createFrameBatcher<number>((kind, updates) => {
    calls.push({ kind, updates })
  })

  batcher.push('Deployment', [1, 2])
  batcher.push('Pod', [3, 4])

  batcher.flush('Deployment')

  assert.equal(calls.length, 1)
  assert.deepEqual(calls[0], { kind: 'Deployment', updates: [1, 2] })

  // Pod is still pending
  batcher.flush('Pod')
  assert.equal(calls.length, 2)
  assert.deepEqual(calls[1], { kind: 'Pod', updates: [3, 4] })
})

test('frameBatcher: flush() drains all pending updates synchronously', () => {
  const calls: Array<{ kind: string; updates: number[] }> = []

  const batcher = createFrameBatcher<number>((kind, updates) => {
    calls.push({ kind, updates })
  })

  batcher.push('Deployment', [10, 20])
  batcher.push('Service', [100])

  batcher.flush()

  assert.equal(calls.length, 2)
  assert.deepEqual(calls[0], { kind: 'Deployment', updates: [10, 20] })
  assert.deepEqual(calls[1], { kind: 'Service', updates: [100] })
})
