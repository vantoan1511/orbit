import test from 'node:test'
import assert from 'node:assert/strict'
import {
  DEFAULT_SHORTCUT_DEFINITIONS,
  DEFAULT_SHORTCUT_BINDINGS,
  DEFAULT_SHORTCUT_BINDINGS_MAC
} from '../shortcuts.ts'
import type { ShortcutActionId } from '../../types/shortcuts.ts'

const EXPECTED_ACTIONS: ShortcutActionId[] = [
  'quickOpen',
  'closeTab',
  'nextTab',
  'prevTab',
  'toggleSidebar',
  'toggleTerminal',
  'openSettings'
]

test('DEFAULT_SHORTCUT_DEFINITIONS defines all 7 core workspace actions', () => {
  assert.equal(DEFAULT_SHORTCUT_DEFINITIONS.length, 7)
  const ids = DEFAULT_SHORTCUT_DEFINITIONS.map((def) => def.id)
  for (const expectedId of EXPECTED_ACTIONS) {
    assert.equal(ids.includes(expectedId), true, `Missing definition for ${expectedId}`)
  }
})

test('Every shortcut definition has valid metadata and non-empty key strings', () => {
  for (const def of DEFAULT_SHORTCUT_DEFINITIONS) {
    assert.ok(def.id.length > 0)
    assert.ok(def.name.length > 0)
    assert.ok(def.description.length > 0)
    assert.ok(def.category.length > 0)
    assert.ok(def.defaultKey.length > 0)
    assert.ok(def.defaultKeyMac.length > 0)
  }
})

test('DEFAULT_SHORTCUT_BINDINGS contains valid mappings for all actions', () => {
  for (const id of EXPECTED_ACTIONS) {
    assert.ok(DEFAULT_SHORTCUT_BINDINGS[id], `Missing binding for ${id}`)
    assert.ok(DEFAULT_SHORTCUT_BINDINGS_MAC[id], `Missing mac binding for ${id}`)
  }
})
