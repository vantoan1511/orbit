import assert from 'node:assert/strict'
import test from 'node:test'
import { effectScope } from 'vue'
import { useDeferredMount } from '../useDeferredMount.ts'

test('useDeferredMount starts with isHydrated=false, stays false after first frame, and flips to true after second frame', () => {
  const queue: Array<{ id: number; cb: () => void }> = []
  let nextId = 1
  const scheduleFrame = (cb: () => void) => {
    const id = nextId++
    queue.push({ id, cb })
    return id
  }
  const cancelFrame = (handle: unknown) => {
    const idx = queue.findIndex((item) => item.id === handle)
    if (idx !== -1) queue.splice(idx, 1)
  }

  const scope = effectScope()
  let isHydratedRef: { value: boolean } | undefined

  scope.run(() => {
    const { isHydrated } = useDeferredMount({ scheduleFrame, cancelFrame })
    isHydratedRef = isHydrated
  })

  assert.ok(isHydratedRef, 'isHydrated must be defined')
  assert.equal(isHydratedRef.value, false, 'isHydrated must initially be false')
  assert.equal(queue.length, 1, 'first frame must be queued')

  // Flush frame 1: should still be false, and second frame must be queued
  const frame1 = queue.shift()!
  frame1.cb()
  assert.equal(
    isHydratedRef.value,
    false,
    'isHydrated must remain false after first frame (yield across paint)'
  )
  assert.equal(queue.length, 1, 'second frame must be queued')

  // Flush frame 2: now flips to true
  const frame2 = queue.shift()!
  frame2.cb()
  assert.equal(isHydratedRef.value, true, 'isHydrated must become true after second frame')

  scope.stop()
})

test('useDeferredMount cancels pending first frame when scope is disposed before frame fires', () => {
  let cancelledHandle: unknown = null
  const scheduleFrame = () => 101
  const cancelFrame = (handle: unknown) => {
    cancelledHandle = handle
  }

  const scope = effectScope()
  let isHydratedRef: { value: boolean } | undefined

  scope.run(() => {
    const { isHydrated } = useDeferredMount({ scheduleFrame, cancelFrame })
    isHydratedRef = isHydrated
  })

  assert.ok(isHydratedRef)
  assert.equal(isHydratedRef.value, false)

  // Dispose scope before frame executes
  scope.stop()

  assert.equal(cancelledHandle, 101, 'cancelFrame must be called with handle on scope dispose')
  assert.equal(isHydratedRef.value, false, 'isHydrated must remain false')
})

test('useDeferredMount dispose between frames cancels the pending second frame', () => {
  const queue: Array<{ id: number; cb: () => void }> = []
  let nextId = 1
  let cancelledHandle: unknown = null

  const scheduleFrame = (cb: () => void) => {
    const id = nextId++
    queue.push({ id, cb })
    return id
  }
  const cancelFrame = (handle: unknown) => {
    cancelledHandle = handle
    const idx = queue.findIndex((item) => item.id === handle)
    if (idx !== -1) queue.splice(idx, 1)
  }

  const scope = effectScope()
  let isHydratedRef: { value: boolean } | undefined

  scope.run(() => {
    const { isHydrated } = useDeferredMount({ scheduleFrame, cancelFrame })
    isHydratedRef = isHydrated
  })

  assert.equal(queue.length, 1)
  const frame1 = queue.shift()!
  assert.equal(frame1.id, 1)
  frame1.cb() // executes frame 1

  assert.equal(queue.length, 1, 'second frame should be queued')
  assert.equal(queue[0].id, 2, 'second frame handle should be 2')
  assert.equal(isHydratedRef?.value, false)

  // Dispose between frames
  scope.stop()

  assert.equal(cancelledHandle, 2, 'cancelFrame must be called with frame 2 handle')
  assert.equal(queue.length, 0, 'frame 2 should have been cancelled')
  assert.equal(isHydratedRef?.value, false, 'isHydrated must remain false after disposal')
})
