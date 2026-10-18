import assert from 'node:assert/strict'
import fs from 'node:fs'
import path from 'node:path'
import test from 'node:test'

test('AppTitleBar template includes data-component and uses semantic token classes', () => {
  const filePath = path.resolve('src/components/layout/AppTitleBar.vue')
  const content = fs.readFileSync(filePath, 'utf-8')

  assert.ok(
    content.includes('data-component="app-titlebar"'),
    'AppTitleBar must have data-component="app-titlebar"'
  )
  assert.ok(content.includes('bg-(--warning-soft)'), 'DEV badge must use bg-(--warning-soft)')
  assert.ok(
    !content.includes('bg-amber-500/15'),
    'AppTitleBar must not contain hardcoded bg-amber-500/15'
  )
})

test('WindowControls includes data-component="window-controls"', () => {
  const filePath = path.resolve('src/components/layout/WindowControls.vue')
  const content = fs.readFileSync(filePath, 'utf-8')

  assert.ok(
    content.includes('data-component="window-controls"'),
    'WindowControls must have data-component="window-controls"'
  )
})
