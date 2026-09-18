import test from 'node:test'
import assert from 'node:assert/strict'
import type { EventInfo } from '@/types/kubernetes'
import { getEventTimestamp, sortEventsDesc } from '../events.ts'

function createMockEvent(partial: Partial<EventInfo>): EventInfo {
  return {
    uid: partial.uid ?? 'test-uid',
    time: partial.time ?? '',
    type: partial.type ?? 'Normal',
    reason: partial.reason ?? 'Scheduled',
    objectName: partial.objectName ?? 'pod-1',
    objectKind: partial.objectKind ?? 'Pod',
    message: partial.message ?? 'Test event message',
    namespace: partial.namespace ?? 'default',
    source: partial.source ?? 'default-scheduler',
    firstSeen: partial.firstSeen ?? '',
    lastSeen: partial.lastSeen ?? '',
    count: partial.count ?? 1,
    labels: partial.labels ?? {},
    annotations: partial.annotations ?? {}
  }
}

test('getEventTimestamp extracts parsed epoch ms from lastSeen or firstSeen', () => {
  const eventWithBoth = createMockEvent({
    firstSeen: 'Sep 18, 2026, 07:00 PM',
    lastSeen: 'Sep 18, 2026, 08:15 PM'
  })
  assert.equal(getEventTimestamp(eventWithBoth), Date.parse('Sep 18, 2026, 08:15 PM'))

  const eventWithFirstOnly = createMockEvent({
    firstSeen: 'Sep 18, 2026, 07:00 PM',
    lastSeen: ''
  })
  assert.equal(getEventTimestamp(eventWithFirstOnly), Date.parse('Sep 18, 2026, 07:00 PM'))

  const eventWithInvalid = createMockEvent({
    firstSeen: 'not-a-date',
    lastSeen: ''
  })
  assert.equal(getEventTimestamp(eventWithInvalid), 0)

  const eventWithEmpty = createMockEvent({
    firstSeen: '',
    lastSeen: ''
  })
  assert.equal(getEventTimestamp(eventWithEmpty), 0)
})

test('sortEventsDesc sorts events with distinct timestamps in descending order', () => {
  const e1 = createMockEvent({ uid: 'e1', lastSeen: 'Sep 18, 2026, 08:00 PM' })
  const e2 = createMockEvent({ uid: 'e2', lastSeen: 'Sep 18, 2026, 08:05 PM' })
  const e3 = createMockEvent({ uid: 'e3', lastSeen: 'Sep 18, 2026, 08:10 PM' })

  const sorted = sortEventsDesc([e1, e2, e3])
  assert.deepEqual(
    sorted.map((e) => e.uid),
    ['e3', 'e2', 'e1']
  )
})

test('sortEventsDesc prioritizes lastSeen over firstSeen for repeated events', () => {
  // e1 occurred first at 07:00 PM, but repeated at 08:20 PM
  const e1 = createMockEvent({
    uid: 'e1',
    firstSeen: 'Sep 18, 2026, 07:00 PM',
    lastSeen: 'Sep 18, 2026, 08:20 PM',
    count: 10
  })
  // e2 occurred once at 08:10 PM
  const e2 = createMockEvent({
    uid: 'e2',
    firstSeen: 'Sep 18, 2026, 08:10 PM',
    lastSeen: 'Sep 18, 2026, 08:10 PM',
    count: 1
  })

  const sorted = sortEventsDesc([e1, e2])
  assert.deepEqual(
    sorted.map((e) => e.uid),
    ['e1', 'e2']
  )
})

test('sortEventsDesc falls back to firstSeen when lastSeen is empty', () => {
  const e1 = createMockEvent({ uid: 'e1', firstSeen: 'Sep 18, 2026, 08:00 PM', lastSeen: '' })
  const e2 = createMockEvent({ uid: 'e2', firstSeen: 'Sep 18, 2026, 08:15 PM', lastSeen: '' })

  const sorted = sortEventsDesc([e1, e2])
  assert.deepEqual(
    sorted.map((e) => e.uid),
    ['e2', 'e1']
  )
})

test('sortEventsDesc preserves reverse arrival order when timestamps are equal', () => {
  const e1 = createMockEvent({ uid: 'e1', lastSeen: 'Sep 18, 2026, 08:00 PM' })
  const e2 = createMockEvent({ uid: 'e2', lastSeen: 'Sep 18, 2026, 08:00 PM' })

  // e2 was later in the original array, so in descending (newest first) it should come first
  const sorted = sortEventsDesc([e1, e2])
  assert.deepEqual(
    sorted.map((e) => e.uid),
    ['e2', 'e1']
  )
})

test('sortEventsDesc preserves reverse arrival order when timestamps are empty or invalid', () => {
  const e1 = createMockEvent({ uid: 'e1', lastSeen: '' })
  const e2 = createMockEvent({ uid: 'e2', lastSeen: '' })
  const e3 = createMockEvent({ uid: 'e3', lastSeen: '' })

  const sorted = sortEventsDesc([e1, e2, e3])
  assert.deepEqual(
    sorted.map((e) => e.uid),
    ['e3', 'e2', 'e1']
  )
})

test('sortEventsDesc handles empty, single-item, and null/undefined gracefully', () => {
  assert.deepEqual(sortEventsDesc([]), [])

  const single = createMockEvent({ uid: 'single' })
  assert.deepEqual(sortEventsDesc([single]), [single])

  assert.deepEqual(sortEventsDesc(null as unknown as EventInfo[]), [])
  assert.deepEqual(sortEventsDesc(undefined as unknown as EventInfo[]), [])
})

test('sortEventsDesc does not mutate the source array', () => {
  const e1 = createMockEvent({ uid: 'e1', lastSeen: 'Sep 18, 2026, 08:00 PM' })
  const e2 = createMockEvent({ uid: 'e2', lastSeen: 'Sep 18, 2026, 08:10 PM' })
  const original = [e1, e2]

  sortEventsDesc(original)
  assert.equal(original[0].uid, 'e1')
  assert.equal(original[1].uid, 'e2')
})
