import test from 'node:test'
import assert from 'node:assert/strict'
import { isKeyComboMatch } from '../../../utils/shortcutHelpers.ts'

test('AppLayout keyboard matching triggers closeTab for Ctrl+W or customized hotkey', () => {
  const eventCtrlW = { key: 'w', ctrlKey: true }
  assert.equal(isKeyComboMatch(eventCtrlW, 'Ctrl+W', false), true)
  assert.equal(isKeyComboMatch(eventCtrlW, 'Ctrl+Q', false), false)

  const eventCustom = { key: 'q', ctrlKey: true }
  assert.equal(isKeyComboMatch(eventCustom, 'Ctrl+Q', false), true)
})

test('AppLayout keyboard matching triggers nextTab and prevTab hotkeys', () => {
  const eventNext = { key: 'Tab', ctrlKey: true }
  assert.equal(isKeyComboMatch(eventNext, 'Ctrl+Tab', false), true)

  const eventPrev = { key: 'Tab', ctrlKey: true, shiftKey: true }
  assert.equal(isKeyComboMatch(eventPrev, 'Ctrl+Shift+Tab', false), true)
})

test('AppLayout keyboard matching triggers toggleSidebar and toggleTerminal', () => {
  const eventSidebar = { key: 'b', ctrlKey: true }
  assert.equal(isKeyComboMatch(eventSidebar, 'Ctrl+B', false), true)

  const eventTerminal = { key: '`', ctrlKey: true }
  assert.equal(isKeyComboMatch(eventTerminal, 'Ctrl+`', false), true)
})

import fs from 'node:fs'
import path from 'node:path'

test('SearchEverywhereDialog keyboard matching triggers quickOpen', () => {
  const eventQuickOpen = { key: 'p', ctrlKey: true }
  assert.equal(isKeyComboMatch(eventQuickOpen, 'Ctrl+P', false), true)

  const eventCustomQuickOpen = { key: 'k', ctrlKey: true }
  assert.equal(isKeyComboMatch(eventCustomQuickOpen, 'Ctrl+K', false), true)
  assert.equal(isKeyComboMatch(eventQuickOpen, 'Ctrl+K', false), false)
})

test('SearchEverywhereDialog does not hardcode Ctrl+Shift+P alias and strictly uses shortcutStore', () => {
  const filePath = path.resolve(process.cwd(), 'src/components/search/SearchEverywhereDialog.vue')
  const content = fs.readFileSync(filePath, 'utf-8')

  assert.ok(
    content.includes("shortcutStore.matchesAction(e, 'quickOpen')"),
    'SearchEverywhereDialog must match quickOpen action'
  )
  assert.ok(
    !content.includes('e.shiftKey && (e.key ==='),
    'SearchEverywhereDialog must not hardcode Shift+P shortcut alias'
  )

  const eventCtrlShiftP = { key: 'p', ctrlKey: true, shiftKey: true }
  assert.equal(isKeyComboMatch(eventCtrlShiftP, 'Ctrl+P', false), false)
})
