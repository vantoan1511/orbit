import assert from 'node:assert/strict'
import fs from 'node:fs'
import path from 'node:path'
import test from 'node:test'

test('ResourceActionMenu uses centralized ui-overlay-bg token and delegates hover/focus states to PrimeVue', () => {
  const filePath = path.resolve('src/components/shared/ResourceActionMenu.vue')
  const content = fs.readFileSync(filePath, 'utf-8')

  // Menu and ContextMenu root surfaces must use --ui-overlay-bg
  assert.ok(
    content.includes('bg-(--ui-overlay-bg)'),
    'ResourceActionMenu must use bg-(--ui-overlay-bg) for menu root containers'
  )
  assert.ok(!content.includes('bg-(--bg-card)'), 'ResourceActionMenu must not use bg-(--bg-card)')

  // Inner button must delegate hover/focus backgrounds and colors to PrimeVue item wrapper
  assert.ok(
    !content.includes('hover:bg-(--bg-hover)'),
    'ResourceActionMenu must not have conflicting hover:bg-(--bg-hover) on inner MenuItemButton'
  )
  assert.ok(
    !content.includes('hover:text-primary'),
    'ResourceActionMenu must not have conflicting hover:text-primary on inner MenuItemButton'
  )
  assert.ok(
    content.includes('text-inherit bg-transparent'),
    'ResourceActionMenu inner MenuItemButton must use text-inherit bg-transparent'
  )
})
