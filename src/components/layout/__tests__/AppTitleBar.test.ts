import assert from 'node:assert/strict'
import fs from 'node:fs'
import path from 'node:path'
import test from 'node:test'

test('AppTitleBar component defines data-tauri-drag-region and integrates WindowControls', () => {
  const filePath = path.resolve(process.cwd(), 'src/components/layout/AppTitleBar.vue')
  const content = fs.readFileSync(filePath, 'utf-8')

  assert.ok(
    content.includes('data-tauri-drag-region'),
    'AppTitleBar must include data-tauri-drag-region attribute for native window dragging'
  )
  assert.ok(
    content.includes('WindowControls'),
    'AppTitleBar must integrate WindowControls component'
  )
  assert.ok(
    content.includes('toggleMaximize'),
    'AppTitleBar must support double click to toggle maximize'
  )
})

test('AppLayout integrates AppTitleBar at root of window layout', () => {
  const layoutPath = path.resolve(process.cwd(), 'src/components/layout/AppLayout.vue')
  const content = fs.readFileSync(layoutPath, 'utf-8')

  assert.ok(content.includes('AppTitleBar'), 'AppLayout must mount AppTitleBar')
})
