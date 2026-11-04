import assert from 'node:assert/strict'
import test from 'node:test'
import { createFrameBatcher } from '../frameBatcher.ts'

test('frameBatcher: concatenates updates and calls apply once per kind per scheduled frame', () => {
  const calls: Array<{ kind: string; updates: number[] }> = []
  let scheduledCallback: (() => void) | null = null

  const manualScheduler = (cb: () => void) => {
    scheduledCallback = cb
  }

  const batcher = createFrameBatcher<number>((kind, updates) => {
    calls.push({ kind, updates })
  }, manualScheduler)

  // 3 pushes for 'Deployment' before tick
  batcher.push('Deployment', [1])
  batcher.push('Deployment', [2, 3])
  batcher.push('Deployment', [4])

  assert.equal(calls.length, 0, 'apply must not be called synchronously before scheduled frame')
  assert.ok(scheduledCallback !== null, 'scheduler should have been called')

  // Run scheduled tick
  scheduledCallback()
  scheduledCallback = null

  assert.equal(calls.length, 1)
  assert.equal(calls[0]?.kind, 'Deployment')
  assert.deepEqual(calls[0]?.updates, [1, 2, 3, 4])
})

test('frameBatcher: handles multiple kinds in one frame with separate apply calls', () => {
  const calls: Array<{ kind: string; updates: string[] }> = []
  let scheduledCallback: (() => void) | null = null

  const batcher = createFrameBatcher<string>(
    (kind, updates) => {
      calls.push({ kind, updates })
    },
    (cb) => {
      scheduledCallback = cb
    }
  )

  batcher.push('Deployment', ['dep-1'])
  batcher.push('Pod', ['pod-1', 'pod-2'])
  batcher.push('Deployment', ['dep-2'])

  scheduledCallback?.()

  assert.equal(calls.length, 2)
  const depCall = calls.find((c) => c.kind === 'Deployment')
  const podCall = calls.find((c) => c.kind === 'Pod')

  assert.deepEqual(depCall?.updates, ['dep-1', 'dep-2'])
  assert.deepEqual(podCall?.updates, ['pod-1', 'pod-2'])
})

test('frameBatcher: clear() drops pending updates without invoking apply', () => {
  const calls: Array<{ kind: string; updates: number[] }> = []
  let scheduledCallback: (() => void) | null = null

  const batcher = createFrameBatcher<number>(
    (kind, updates) => {
      calls.push({ kind, updates })
    },
    (cb) => {
      scheduledCallback = cb
    }
  )

  batcher.push('Deployment', [1, 2])
  batcher.clear()

  // Even if scheduler callback fires after clear, no updates are applied
  scheduledCallback?.()

  assert.equal(calls.length, 0)
})

test('frameBatcher: flush() drains pending updates synchronously', () => {
  const calls: Array<{ kind: string; updates: number[] }> = []

  const batcher = createFrameBatcher<number>(
    (kind, updates) => {
      calls.push({ kind, updates })
    },
    () => {
      // Manual schedule that does not auto-fire
    }
  )

  batcher.push('Deployment', [10, 20])
  batcher.push('Service', [100])

  // Synchronous flush
  batcher.flush()

  assert.equal(calls.length, 2)
  assert.deepEqual(calls[0], { kind: 'Deployment', updates: [10, 20] })
  assert.deepEqual(calls[1], { kind: 'Service', updates: [100] })

  // Subsequent flush is a no-op
  batcher.flush()
  assert.equal(calls.length, 2)
})
