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

test('AppTitleBar component integrates centered Search Everywhere bar', () => {
  const filePath = path.resolve(process.cwd(), 'src/components/layout/AppTitleBar.vue')
  const content = fs.readFileSync(filePath, 'utf-8')

  assert.ok(content.includes('useSearchStore'), 'AppTitleBar must import useSearchStore')
  assert.ok(
    content.includes('searchStore.open()'),
    'AppTitleBar must trigger searchStore.open() on click'
  )
  assert.ok(
    content.includes('absolute left-1/2'),
    'Search bar must be centered using absolute left-1/2'
  )
  assert.ok(
    content.includes('-translate-x-1/2'),
    'Search bar must be translated -translate-x-1/2 for true center'
  )
  assert.ok(
    content.includes('@click.stop'),
    'Click on search bar must stop propagation to prevent window dragging'
  )
  assert.ok(
    content.includes('@dblclick.stop'),
    'Double click on search bar must stop propagation to prevent maximize toggle'
  )
  assert.ok(content.includes('Search everywhere...'), 'Search bar must render placeholder label')
})

test('AppHeader component does not retain legacy search everywhere button', () => {
  const headerPath = path.resolve(process.cwd(), 'src/components/layout/AppHeader.vue')
  const content = fs.readFileSync(headerPath, 'utf-8')

  assert.ok(!content.includes('useSearchStore'), 'AppHeader must not import useSearchStore')
  assert.ok(
    !content.includes('Search everywhere...'),
    'AppHeader must not retain legacy search button'
  )
})
