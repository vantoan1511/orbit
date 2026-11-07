import assert from 'node:assert/strict'
import test from 'node:test'
import { effectScope } from 'vue'
import { useDeferredMount } from '../useDeferredMount.ts'

test('useDeferredMount starts with isHydrated=false and flips to true on scheduled frame', () => {
  let scheduledCallback: (() => void) | null = null
  const scheduleFrame = (cb: () => void) => {
    scheduledCallback = cb
    return 42
  }
  const cancelFrame = () => {
    scheduledCallback = null
  }

  const scope = effectScope()
  let isHydratedRef: { value: boolean } | undefined

  scope.run(() => {
    const { isHydrated } = useDeferredMount({ scheduleFrame, cancelFrame })
    isHydratedRef = isHydrated
  })

  assert.ok(isHydratedRef, 'isHydrated must be defined')
  assert.equal(isHydratedRef.value, false, 'isHydrated must initially be false')
  assert.ok(scheduledCallback, 'scheduleFrame must be invoked with callback')

  // Trigger scheduled frame
  scheduledCallback!()
  assert.equal(isHydratedRef.value, true, 'isHydrated must become true after frame callback')

  scope.stop()
})

test('useDeferredMount cancels pending frame when scope is disposed before frame fires', () => {
  let scheduledHandle: unknown = null
  let cancelledHandle: unknown = null
  let scheduledCallback: (() => void) | null = null

  const scheduleFrame = (cb: () => void) => {
    scheduledCallback = cb
    scheduledHandle = 'frame-handle-99'
    return scheduledHandle
  }
  const cancelFrame = (handle: unknown) => {
    cancelledHandle = handle
    scheduledCallback = null
  }

  const scope = effectScope()
  let isHydratedRef: { value: boolean } | undefined

  scope.run(() => {
    const { isHydrated } = useDeferredMount({ scheduleFrame, cancelFrame })
    isHydratedRef = isHydrated
  })

  assert.ok(isHydratedRef)
  assert.equal(isHydratedRef.value, false)
  assert.equal(scheduledHandle, 'frame-handle-99')

  // Dispose scope before frame executes
  scope.stop()

  assert.equal(
    cancelledHandle,
    'frame-handle-99',
    'cancelFrame must be called with handle on scope dispose'
  )
  assert.equal(isHydratedRef.value, false, 'isHydrated must remain false')
  assert.equal(scheduledCallback, null)
})
