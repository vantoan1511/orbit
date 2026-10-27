import assert from 'node:assert/strict'
import fs from 'node:fs'
import path from 'node:path'
import test from 'node:test'

test('ResourceActionMenu delegates root surfaces to PrimeVue preset and delegates hover/focus states', () => {
  const filePath = path.resolve('src/components/shared/ResourceActionMenu.vue')
  const content = fs.readFileSync(filePath, 'utf-8')

  // Menu and ContextMenu root surfaces delegate styling to PrimeVue preset; must not have ad-hoc surface classes
  assert.ok(
    !content.includes('bg-(--ui-overlay-bg)'),
    'ResourceActionMenu must not have ad-hoc bg-(--ui-overlay-bg) in template'
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
