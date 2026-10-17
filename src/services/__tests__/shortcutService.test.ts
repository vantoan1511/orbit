import test from 'node:test'
import assert from 'node:assert/strict'
import { shortcutService, SHORTCUTS_STORAGE_KEY, sanitizeBindings } from '../shortcutService.ts'
import { storage } from '../nativeService.ts'
import {
  DEFAULT_SHORTCUT_BINDINGS,
  DEFAULT_SHORTCUT_BINDINGS_MAC
} from '../../constants/shortcuts.ts'
import type { ShortcutBindings } from '../../types/shortcuts.ts'

test('sanitizeBindings preserves valid actions and fills missing actions with defaults', () => {
  const partial = {
    quickOpen: 'Ctrl+K'
  }
  const sanitized = sanitizeBindings(partial, false)
  assert.equal(sanitized.quickOpen, 'Ctrl+K')
  assert.equal(sanitized.closeTab, DEFAULT_SHORTCUT_BINDINGS.closeTab)
  assert.equal(sanitized.openSettings, DEFAULT_SHORTCUT_BINDINGS.openSettings)
})

test('sanitizeBindings handles empty, null, or corrupted objects gracefully', () => {
  const fallbackWin = sanitizeBindings(null, false)
  assert.deepEqual(fallbackWin, DEFAULT_SHORTCUT_BINDINGS)

  const fallbackMac = sanitizeBindings(undefined, true)
  assert.deepEqual(fallbackMac, DEFAULT_SHORTCUT_BINDINGS_MAC)

  const corrupted = sanitizeBindings('not-an-object', false)
  assert.deepEqual(corrupted, DEFAULT_SHORTCUT_BINDINGS)
})

test('shortcutService.loadBindings returns defaults when storage is empty', async () => {
  await storage.removeData(SHORTCUTS_STORAGE_KEY)
  const bindings = await shortcutService.loadBindings(false)
  assert.deepEqual(bindings, DEFAULT_SHORTCUT_BINDINGS)
})

test('shortcutService.saveBindings persists to storage and loadBindings retrieves it', async () => {
  const custom: ShortcutBindings = {
    ...DEFAULT_SHORTCUT_BINDINGS,
    quickOpen: 'Ctrl+Alt+O',
    openSettings: 'Ctrl+Alt+S'
  }

  await shortcutService.saveBindings(custom)
  const loaded = await shortcutService.loadBindings(false)
  assert.equal(loaded.quickOpen, 'Ctrl+Alt+O')
  assert.equal(loaded.openSettings, 'Ctrl+Alt+S')
  assert.equal(loaded.closeTab, DEFAULT_SHORTCUT_BINDINGS.closeTab)
})

test('shortcutService.resetBindings restores factory default bindings', async () => {
  await shortcutService.saveBindings({
    ...DEFAULT_SHORTCUT_BINDINGS,
    quickOpen: 'Ctrl+Shift+Z'
  })

  const reset = await shortcutService.resetBindings(false)
  assert.deepEqual(reset, DEFAULT_SHORTCUT_BINDINGS)

  const fromStorage = await shortcutService.loadBindings(false)
  assert.deepEqual(fromStorage, DEFAULT_SHORTCUT_BINDINGS)
})

test('shortcutService.saveBindings dispatches updateAppSettings to configuration system', async () => {
  const { coreEngine } = await import('../nativeService.ts')
  const originalDispatch = coreEngine.dispatch
  let dispatchedEvent = ''
  let dispatchedPayload: Record<string, unknown> | null = null

  coreEngine.dispatch = async (event: string, data?: unknown) => {
    dispatchedEvent = event
    dispatchedPayload = data as Record<string, unknown>
  }

  try {
    const custom: ShortcutBindings = {
      ...DEFAULT_SHORTCUT_BINDINGS,
      quickOpen: 'Ctrl+Shift+O'
    }
    await shortcutService.saveBindings(custom, false)
    assert.equal(dispatchedEvent, 'updateAppSettings')
    assert.ok(dispatchedPayload)
    assert.ok('keyboardShortcuts' in dispatchedPayload)
    const shortcuts = dispatchedPayload.keyboardShortcuts as Record<string, string>
    assert.equal(shortcuts.quickOpen, 'Ctrl+Shift+O')
  } finally {
    coreEngine.dispatch = originalDispatch
  }
})

test('shortcutService.loadBindings migrates legacy storage and dispatches updateAppSettings', async () => {
  const { coreEngine } = await import('../nativeService.ts')
  const originalDispatch = coreEngine.dispatch
  let dispatchedEvent = ''
  let dispatchedPayload: Record<string, unknown> | null = null

  coreEngine.dispatch = async (event: string, data?: unknown) => {
    dispatchedEvent = event
    dispatchedPayload = data as Record<string, unknown>
  }

  // Pre-seed legacy storage with an old shortcut
  await storage.setData(SHORTCUTS_STORAGE_KEY, JSON.stringify({ quickOpen: 'Ctrl+Alt+P' }))

  try {
    const loaded = await shortcutService.loadBindings(false)
    assert.equal(loaded.quickOpen, 'Ctrl+Alt+P')

    // Dispatched migration to core configuration system
    assert.equal(dispatchedEvent, 'updateAppSettings')
    assert.ok(dispatchedPayload)
    assert.ok('keyboardShortcuts' in dispatchedPayload)
  } finally {
    coreEngine.dispatch = originalDispatch
    await storage.removeData(SHORTCUTS_STORAGE_KEY)
  }
})
