import assert from 'node:assert/strict'
import test from 'node:test'
import { useTicker, resetTickerForTesting, getSubscriberCountForTesting } from '../useTicker.ts'

test('subscribe -> unsubscribe -> subscribe manages activeSubscribers and interval consistently', () => {
  resetTickerForTesting()

  const { subscribe, unsubscribe } = useTicker()
  assert.equal(getSubscriberCountForTesting(), 0)

  subscribe()
  assert.equal(getSubscriberCountForTesting(), 1)

  unsubscribe()
  assert.equal(getSubscriberCountForTesting(), 0)

  // Subscribing again increments properly
  subscribe()
  assert.equal(getSubscriberCountForTesting(), 1)

  // Double unsubscribe does not make it negative
  unsubscribe()
  unsubscribe()
  assert.equal(getSubscriberCountForTesting(), 0)

  resetTickerForTesting()
})

test('pauses interval when document becomes hidden, resumes on visible', async () => {
  // Setup mock document
  const listeners = new Map<string, Array<() => void>>()
  let visibilityState = 'visible'

  const originalDocument = globalThis.document

  globalThis.document = {
    get visibilityState() {
      return visibilityState
    },
    addEventListener(event: string, handler: () => void) {
      const arr = listeners.get(event) ?? []
      arr.push(handler)
      listeners.set(event, arr)
    },
    removeEventListener(event: string, handler: () => void) {
      const arr = listeners.get(event) ?? []
      listeners.set(
        event,
        arr.filter((h) => h !== handler)
      )
    },
    dispatchEvent(event: string) {
      const arr = listeners.get(event) ?? []
      for (const fn of arr) fn()
    }
  } as unknown as Document

  resetTickerForTesting?.()

  try {
    const { currentTimestamp, subscribe, unsubscribe } = useTicker()

    subscribe()
    const initialTime = currentTimestamp.value

    // Advance real or simulated time
    await new Promise((r) => setTimeout(r, 1100))
    const advancedTime = currentTimestamp.value
    assert.ok(advancedTime > initialTime, 'currentTimestamp should advance while visible')

    // Document becomes hidden
    visibilityState = 'hidden'
    ;(globalThis.document as unknown as { dispatchEvent: (e: string) => void }).dispatchEvent(
      'visibilitychange'
    )

    const hiddenTime = currentTimestamp.value
    await new Promise((r) => setTimeout(r, 1200))
    assert.equal(
      currentTimestamp.value,
      hiddenTime,
      'currentTimestamp must not advance while hidden'
    )

    // Document becomes visible
    visibilityState = 'visible'
    ;(globalThis.document as unknown as { dispatchEvent: (e: string) => void }).dispatchEvent(
      'visibilitychange'
    )

    assert.ok(
      currentTimestamp.value > hiddenTime,
      'currentTimestamp updates immediately upon becoming visible'
    )

    unsubscribe()
  } finally {
    resetTickerForTesting?.()
    if (originalDocument) {
      globalThis.document = originalDocument
    } else {
      delete (globalThis as { document?: Document }).document
    }
  }
})
