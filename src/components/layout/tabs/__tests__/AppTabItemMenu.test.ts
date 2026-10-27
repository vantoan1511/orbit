import assert from 'node:assert/strict'
import fs from 'node:fs'
import path from 'node:path'
import test from 'node:test'

test('AppTabItem ContextMenu delegates root surface to PrimeVue preset and delegates hover/focus states', () => {
  const filePath = path.resolve('src/components/layout/tabs/AppTabItem.vue')
  const content = fs.readFileSync(filePath, 'utf-8')

  // ContextMenu root surface delegates styling to PrimeVue preset; must not have ad-hoc surface classes
  assert.ok(
    !content.includes('bg-(--ui-overlay-bg)'),
    'AppTabItem ContextMenu must not have ad-hoc bg-(--ui-overlay-bg) in template'
  )
  assert.ok(
    !content.includes('bg-(--bg-card)'),
    'AppTabItem ContextMenu must not use bg-(--bg-card)'
  )

  // Inner action element must not have conflicting hover:bg-(--bg-hover)
  assert.ok(
    !content.includes('hover:bg-(--bg-hover)'),
    'AppTabItem ContextMenu must not have conflicting hover:bg-(--bg-hover) on inner action link'
  )
  assert.ok(
    content.includes('text-inherit bg-transparent'),
    'AppTabItem ContextMenu inner action link must use text-inherit bg-transparent'
  )
})
