import type { EventInfo } from '@/types/kubernetes'

/**
 * Extracts an epoch millisecond timestamp from an EventInfo object.
 * Prefers `lastSeen` (most recent occurrence) and falls back to `firstSeen`.
 * Returns 0 if neither can be parsed into a valid timestamp.
 */
export function getEventTimestamp(event: EventInfo): number {
  const raw = event.lastSeen || event.firstSeen
  if (raw) {
    const parsed = Date.parse(raw)
    if (!Number.isNaN(parsed)) {
      return parsed
    }
  }
  return 0
}

/**
 * Sorts a list of Kubernetes events in descending chronological order (newest first).
 * - Primary sort: epoch timestamp descending (from `lastSeen` or `firstSeen`).
 * - Tie-breaker: reverse original arrival order (higher index in Kubernetes list = arrived later).
 * Does not mutate the source array.
 */
export function sortEventsDesc(events: EventInfo[]): EventInfo[] {
  if (!events || events.length <= 1) {
    return events ? [...events] : []
  }

  return events
    .map((event, index) => ({ event, index, timestamp: getEventTimestamp(event) }))
    .sort((a, b) => {
      if (a.timestamp !== b.timestamp) {
        return b.timestamp - a.timestamp
      }
      return b.index - a.index
    })
    .map((item) => item.event)
}
