import test from 'node:test'
import assert from 'node:assert/strict'
import {
  parseKeyCombo,
  formatKeyCombo,
  eventToKeyCombo,
  isKeyComboMatch,
  findShortcutConflict,
  normalizeComboString
} from '../shortcutHelpers.ts'
import type { KeyboardEventLike } from '../keyboard.ts'
import type { ShortcutBindings } from '../../types/shortcuts.ts'

test('parseKeyCombo correctly parses modifier flags and normalized key', () => {
  const win = parseKeyCombo('Ctrl+Shift+P')
  assert.equal(win.ctrlKey, true)
  assert.equal(win.shiftKey, true)
  assert.equal(win.altKey, false)
  assert.equal(win.metaKey, false)
  assert.equal(win.key, 'P')

  const mac = parseKeyCombo('⌘P')
  assert.equal(mac.metaKey, true)
  assert.equal(mac.key, 'P')

  const backtick = parseKeyCombo('Ctrl+`')
  assert.equal(backtick.ctrlKey, true)
  assert.equal(backtick.key, '`')

  const comma = parseKeyCombo('Ctrl+,')
  assert.equal(comma.ctrlKey, true)
  assert.equal(comma.key, ',')
})

test('formatKeyCombo formats symbols for Mac and text modifiers for Windows', () => {
  const combo = {
    key: 'P',
    ctrlKey: false,
    metaKey: true,
    shiftKey: true,
    altKey: false
  }

  assert.equal(formatKeyCombo(combo, true), '⌘⇧P')
  assert.equal(formatKeyCombo(combo, false), 'Ctrl+Shift+P')

  const ctrlTab = {
    key: 'Tab',
    ctrlKey: true,
    metaKey: false,
    shiftKey: false,
    altKey: false
  }
  assert.equal(formatKeyCombo(ctrlTab, false), 'Ctrl+Tab')
  assert.equal(formatKeyCombo(ctrlTab, true), '⌃Tab')
})

test('eventToKeyCombo filters solitary modifier presses and returns KeyCombo', () => {
  assert.equal(eventToKeyCombo({ key: 'Control' }), null)
  assert.equal(eventToKeyCombo({ key: 'Shift' }), null)
  assert.equal(eventToKeyCombo({ key: 'Alt' }), null)
  assert.equal(eventToKeyCombo({ key: 'Meta' }), null)

  const normalEvent: KeyboardEventLike = {
    key: 'p',
    ctrlKey: true
  }
  const combo = eventToKeyCombo(normalEvent)
  assert.ok(combo)
  assert.equal(combo.key, 'P')
  assert.equal(combo.ctrlKey, true)
})

test('isKeyComboMatch matches exact hotkeys case-insensitively and respects OS modifiers', () => {
  const eventP: KeyboardEventLike = {
    key: 'p',
    ctrlKey: true
  }
  assert.equal(isKeyComboMatch(eventP, 'Ctrl+P', false), true)
  assert.equal(isKeyComboMatch(eventP, 'Ctrl+W', false), false)
  assert.equal(isKeyComboMatch(eventP, 'Ctrl+P', true), false) // On Mac, 'Ctrl+P' expects metaKey unless ⌃

  const macEventP: KeyboardEventLike = {
    key: 'p',
    metaKey: true
  }
  assert.equal(isKeyComboMatch(macEventP, '⌘P', true), true)
  assert.equal(isKeyComboMatch(macEventP, 'Ctrl+P', true), true)
})

test('findShortcutConflict detects when another action is assigned the same key', () => {
  const bindings: ShortcutBindings = {
    quickOpen: 'Ctrl+P',
    closeTab: 'Ctrl+W',
    nextTab: 'Ctrl+Tab',
    prevTab: 'Ctrl+Shift+Tab',
    toggleSidebar: 'Ctrl+B',
    toggleTerminal: 'Ctrl+`',
    openSettings: 'Ctrl+,'
  }

  // Assigning 'Ctrl+W' to 'quickOpen' conflicts with 'closeTab'
  const conflict = findShortcutConflict('quickOpen', 'Ctrl+W', bindings, false)
  assert.equal(conflict, 'closeTab')

  // Assigning 'Ctrl+K' does not conflict
  const noConflict = findShortcutConflict('quickOpen', 'Ctrl+K', bindings, false)
  assert.equal(noConflict, null)

  // Re-assigning the same key to the same action is not a conflict
  const sameAction = findShortcutConflict('quickOpen', 'Ctrl+P', bindings, false)
  assert.equal(sameAction, null)
})

test('normalizeComboString cleans whitespace and normalizes order', () => {
  assert.equal(normalizeComboString('ctrl + shift + p', false), 'Ctrl+Shift+P')
  assert.equal(normalizeComboString('shift+ctrl+p', false), 'Ctrl+Shift+P')
})
