import assert from 'node:assert/strict'
import fs from 'node:fs'
import path from 'node:path'
import test from 'node:test'

test('StatusBadge and ReplicasProgressBar consume CSS variable tokens', () => {
  const statusBadgePath = path.resolve('src/components/shared/StatusBadge.vue')
  const statusBadgeContent = fs.readFileSync(statusBadgePath, 'utf-8')

  assert.ok(
    statusBadgeContent.includes('data-component="status-badge"'),
    'StatusBadge must have data-component="status-badge"'
  )
  assert.ok(
    !statusBadgeContent.includes('bg-gray-400'),
    'StatusBadge must not have hardcoded bg-gray-400'
  )
  assert.ok(
    !statusBadgeContent.includes('text-gray-400'),
    'StatusBadge must not have hardcoded text-gray-400'
  )
  assert.ok(
    statusBadgeContent.includes('bg-(--text-muted)'),
    'StatusBadge must use bg-(--text-muted) for fallback dot'
  )

  const replicasPath = path.resolve('src/components/shared/ReplicasProgressBar.vue')
  const replicasContent = fs.readFileSync(replicasPath, 'utf-8')

  assert.ok(
    replicasContent.includes('data-component="replicas-progress-bar"'),
    'ReplicasProgressBar must have data-component="replicas-progress-bar"'
  )
  assert.ok(
    replicasContent.includes('bg-(--success)'),
    'ReplicasProgressBar must use bg-(--success) for ready replicas'
  )
  assert.ok(
    !replicasContent.includes('bg-emerald-500'),
    'ReplicasProgressBar must not have hardcoded bg-emerald-500'
  )

  const highlightedPath = path.resolve('src/components/shared/HighlightedText.vue')
  const highlightedContent = fs.readFileSync(highlightedPath, 'utf-8')

  assert.ok(
    highlightedContent.includes('data-component="highlighted-text"'),
    'HighlightedText must have data-component="highlighted-text"'
  )
  assert.ok(
    !highlightedContent.includes('bg-amber-500/30'),
    'HighlightedText must not have hardcoded bg-amber-500/30'
  )
})
