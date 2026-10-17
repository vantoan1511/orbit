import test from 'node:test'
import assert from 'node:assert/strict'
import { setActivePinia, createPinia } from 'pinia'
import { useShortcutStore } from '../shortcutStore.ts'
import { DEFAULT_SHORTCUT_BINDINGS } from '../../constants/shortcuts.ts'
import { storage } from '../../services/nativeService.ts'
import { SHORTCUTS_STORAGE_KEY } from '../../services/shortcutService.ts'

test('shortcutStore initializes with platform default bindings', async () => {
  setActivePinia(createPinia())
  await storage.removeData(SHORTCUTS_STORAGE_KEY)

  const store = useShortcutStore()
  await store.init(false)

  assert.deepEqual(store.bindings, DEFAULT_SHORTCUT_BINDINGS)
  assert.equal(store.isListening, false)
  assert.equal(store.listeningActionId, null)
  assert.equal(store.pendingConflict, null)
})

test('shortcutStore.rebind updates binding and saves when no conflict occurs', async () => {
  setActivePinia(createPinia())
  const store = useShortcutStore()
  await store.init(false)

  const result = await store.rebind('quickOpen', 'Ctrl+K')
  assert.equal(result.success, true)
  assert.equal(store.bindings.quickOpen, 'Ctrl+K')
  assert.equal(store.isModified('quickOpen'), true)
  assert.equal(store.isModified('closeTab'), false)
})

test('shortcutStore.rebind detects conflict, sets pendingConflict, and confirmSwap swaps keys', async () => {
  setActivePinia(createPinia())
  await storage.removeData(SHORTCUTS_STORAGE_KEY)
  const store = useShortcutStore()
  await store.init(false)

  // 'Ctrl+W' is initially bound to closeTab
  const result = await store.rebind('quickOpen', 'Ctrl+W')
  assert.equal(result.success, false)
  assert.equal(result.conflictWith, 'closeTab')
  assert.ok(store.pendingConflict)
  assert.equal(store.pendingConflict.actionId, 'quickOpen')
  assert.equal(store.pendingConflict.conflictingActionId, 'closeTab')

  // Confirm swap
  await store.confirmConflictSwap()
  assert.equal(store.pendingConflict, null)
  assert.equal(store.bindings.quickOpen, 'Ctrl+W')
  assert.equal(store.bindings.closeTab, 'Ctrl+P')
})

test('shortcutStore.cancelConflict clears pendingConflict without changing bindings', async () => {
  setActivePinia(createPinia())
  await storage.removeData(SHORTCUTS_STORAGE_KEY)
  const store = useShortcutStore()
  await store.init(false)

  await store.rebind('quickOpen', 'Ctrl+W')
  assert.ok(store.pendingConflict)

  store.cancelConflict()
  assert.equal(store.pendingConflict, null)
  assert.equal(store.bindings.quickOpen, DEFAULT_SHORTCUT_BINDINGS.quickOpen)
  assert.equal(store.bindings.closeTab, DEFAULT_SHORTCUT_BINDINGS.closeTab)
})

test('shortcutStore.resetAction and resetAll restore defaults', async () => {
  setActivePinia(createPinia())
  const store = useShortcutStore()
  await store.init(false)

  await store.rebind('quickOpen', 'Ctrl+J')
  await store.rebind('openSettings', 'Ctrl+Alt+S')
  assert.equal(store.hasAnyCustomizations, true)

  await store.resetAction('quickOpen')
  assert.equal(store.bindings.quickOpen, DEFAULT_SHORTCUT_BINDINGS.quickOpen)
  assert.equal(store.bindings.openSettings, 'Ctrl+Alt+S')

  // Set active listening and a conflict to ensure resetAll cleans them up
  store.startListening('toggleSidebar')
  assert.equal(store.isListening, true)
  assert.equal(store.listeningActionId, 'toggleSidebar')

  await store.rebind('toggleSidebar', 'Ctrl+Alt+S') // conflicts with openSettings
  assert.ok(store.pendingConflict)

  await store.resetAll()
  assert.deepEqual(store.bindings, DEFAULT_SHORTCUT_BINDINGS)
  assert.equal(store.hasAnyCustomizations, false)
  assert.equal(store.isListening, false)
  assert.equal(store.listeningActionId, null)
  assert.equal(store.pendingConflict, null)
})

test('shortcutStore.hasAnyCustomizations normalizes combo strings and isModified handles case insensitivity', async () => {
  setActivePinia(createPinia())
  const store = useShortcutStore()
  await store.init(false)

  assert.equal(store.hasAnyCustomizations, false)
  assert.equal(store.isModified('quickOpen'), false)

  // Rebind to equivalent default key in lowercase 'ctrl+p'
  // Should NOT count as modified because it normalizes to 'Ctrl+P'
  await store.rebind('quickOpen', 'ctrl+p')
  assert.equal(store.isModified('quickOpen'), false)
  assert.equal(store.hasAnyCustomizations, false)

  // Rebind to truly customized key
  await store.rebind('quickOpen', 'Ctrl+Shift+O')
  assert.equal(store.isModified('quickOpen'), true)
  assert.equal(store.hasAnyCustomizations, true)

  await store.resetAll()
  assert.equal(store.isModified('quickOpen'), false)
  assert.equal(store.hasAnyCustomizations, false)
})

test('shortcutStore.matchesAction matches keyboard events against current bindings', async () => {
  setActivePinia(createPinia())
  const store = useShortcutStore()
  await store.init(false)

  const eventP = { key: 'p', ctrlKey: true }
  assert.equal(store.matchesAction(eventP, 'quickOpen'), true)
  assert.equal(store.matchesAction(eventP, 'closeTab'), false)

  await store.rebind('quickOpen', 'Ctrl+K')
  assert.equal(store.matchesAction(eventP, 'quickOpen'), false)

  const eventK = { key: 'k', ctrlKey: true }
  assert.equal(store.matchesAction(eventK, 'quickOpen'), true)
})
