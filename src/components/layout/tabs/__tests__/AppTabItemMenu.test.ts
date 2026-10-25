import assert from 'node:assert/strict'
import fs from 'node:fs'
import path from 'node:path'
import test from 'node:test'

test('AppTabItem ContextMenu uses centralized ui-overlay-bg token and delegates hover/focus states to PrimeVue', () => {
  const filePath = path.resolve('src/components/layout/tabs/AppTabItem.vue')
  const content = fs.readFileSync(filePath, 'utf-8')

  // ContextMenu root surface must use --ui-overlay-bg
  assert.ok(
    content.includes('bg-(--ui-overlay-bg)'),
    'AppTabItem ContextMenu must use bg-(--ui-overlay-bg)'
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
