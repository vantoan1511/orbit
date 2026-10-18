import assert from 'node:assert/strict'
import fs from 'node:fs'
import path from 'node:path'
import test from 'node:test'

test('StatusSummary uses semantic tokens for cluster health and has data-component', () => {
  const summaryPath = path.resolve('src/components/dashboard/StatusSummary.vue')
  const content = fs.readFileSync(summaryPath, 'utf-8')

  assert.ok(
    content.includes('data-component="dashboard-status-summary"'),
    'StatusSummary must have data-component="dashboard-status-summary"'
  )
  assert.ok(
    content.includes('text-(--success)'),
    'StatusSummary must use text-(--success)'
  )
  assert.ok(
    !content.includes('text-emerald-500'),
    'StatusSummary must not contain text-emerald-500'
  )
  assert.ok(
    !content.includes('bg-emerald-500/10'),
    'StatusSummary must not contain bg-emerald-500/10'
  )
  assert.ok(
    content.includes('text-(--warning)'),
    'StatusSummary must use text-(--warning)'
  )
})

test('ResourceUsage uses semantic tokens, data-component, and eliminates hardcoded violet', () => {
  const usagePath = path.resolve('src/components/dashboard/ResourceUsage.vue')
  const content = fs.readFileSync(usagePath, 'utf-8')

  assert.ok(
    content.includes('data-component="dashboard-resource-usage"'),
    'ResourceUsage must have data-component="dashboard-resource-usage"'
  )
  assert.ok(
    !content.includes('bg-violet-500'),
    'ResourceUsage must not have hardcoded bg-violet-500'
  )
  assert.ok(
    content.includes('bg-(--color-node)'),
    'ResourceUsage must use bg-(--color-node) for memory indicator'
  )
})
