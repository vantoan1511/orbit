import test from 'node:test'
import assert from 'node:assert/strict'
import {
  formatWorkloadContextBadge,
  recalculateVirtualScroller
} from '../../utils/logsViewHelpers.ts'

test('formatWorkloadContextBadge formats kind and workload name correctly', () => {
  const result = formatWorkloadContextBadge('Deployment', 'nginx', 'prod')
  assert.equal(result.badgeText, 'Deployment/nginx')
  assert.equal(result.subText, 'prod')
})

test('formatWorkloadContextBadge handles missing namespace gracefully', () => {
  const result = formatWorkloadContextBadge('Pod', 'standalone-pod')
  assert.equal(result.badgeText, 'Pod/standalone-pod')
  assert.equal(result.subText, undefined)
})

test('formatWorkloadContextBadge returns empty string when workload name is missing', () => {
  const result = formatWorkloadContextBadge(undefined, undefined)
  assert.equal(result.badgeText, '')
  assert.equal(result.subText, undefined)
})

test('recalculateVirtualScroller calls vs.init() when virtualScrollerRef is present', () => {
  let initCalled = 0
  let scrollCalled = 0
  const mockVs = {
    init: () => {
      initCalled++
    }
  }

  const success = recalculateVirtualScroller(mockVs, false, () => {
    scrollCalled++
  })

  assert.equal(success, true)
  assert.equal(initCalled, 1)
  assert.equal(scrollCalled, 0)
})

test('recalculateVirtualScroller calls scrollToBottom when isFollowing is true', () => {
  let initCalled = 0
  let scrollCalled = 0
  const mockVs = {
    init: () => {
      initCalled++
    }
  }

  const success = recalculateVirtualScroller(mockVs, true, () => {
    scrollCalled++
  })

  assert.equal(success, true)
  assert.equal(initCalled, 1)
  assert.equal(scrollCalled, 1)
})

test('recalculateVirtualScroller handles null or missing vs instance gracefully', () => {
  let scrollCalled = 0
  const success = recalculateVirtualScroller(null, true, () => {
    scrollCalled++
  })

  assert.equal(success, false)
  assert.equal(scrollCalled, 0)
})

test('CONSOLE_CONTAINER_CLASSES contains both light and dark theme background and text tokens', async () => {
  const { CONSOLE_CONTAINER_CLASSES } = await import('../../utils/logsViewHelpers.ts')
  assert.equal(CONSOLE_CONTAINER_CLASSES.includes('bg-(--bg-card)'), true)
  assert.equal(CONSOLE_CONTAINER_CLASSES.includes('dark:bg-[#090d16]'), true)
  assert.equal(CONSOLE_CONTAINER_CLASSES.includes('text-zinc-800'), true)
  assert.equal(CONSOLE_CONTAINER_CLASSES.includes('dark:text-zinc-200'), true)
})

test('LOG_ROW_CLASSES contains light hover and dark hover classes', async () => {
  const { LOG_ROW_CLASSES } = await import('../../utils/logsViewHelpers.ts')
  assert.equal(LOG_ROW_CLASSES.includes('hover:bg-zinc-100'), true)
  assert.equal(LOG_ROW_CLASSES.includes('dark:hover:bg-white/5'), true)
})

test('LOG_ORIGIN_BADGE_CLASSES contains readable badge text colors for both themes', async () => {
  const { LOG_ORIGIN_BADGE_CLASSES } = await import('../../utils/logsViewHelpers.ts')
  assert.equal(LOG_ORIGIN_BADGE_CLASSES.includes('text-zinc-600'), true)
  assert.equal(LOG_ORIGIN_BADGE_CLASSES.includes('dark:text-zinc-400'), true)
})

test('useLogHighlighting getLogLevelColor returns dual-theme classes for gray rule', async () => {
  const { useLogHighlighting } = await import('../../composables/useLogHighlighting.ts')
  const { getLogLevelColor, customRules } = useLogHighlighting()
  customRules.value = [
    {
      id: 'test-gray',
      pattern: 'TRACE',
      color: 'gray',
      bold: false,
      caseSensitive: false,
      isRegex: false
    }
  ]
  const colorClass = getLogLevelColor('TRACE: some message')
  assert.equal(colorClass.includes('text-zinc-500'), true)
  assert.equal(colorClass.includes('dark:text-zinc-400'), true)
})
