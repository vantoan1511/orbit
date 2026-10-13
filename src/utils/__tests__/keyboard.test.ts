import test from 'node:test'
import assert from 'node:assert/strict'
import {
  isBrowserRefreshShortcut,
  handlePreventBrowserRefresh,
  type KeyboardEventLike
} from '../keyboard.ts'

test('isBrowserRefreshShortcut detects F5 with and without modifiers', () => {
  assert.equal(isBrowserRefreshShortcut({ key: 'F5' }), true)
  assert.equal(isBrowserRefreshShortcut({ key: 'F5', ctrlKey: true }), true)
  assert.equal(isBrowserRefreshShortcut({ key: 'F5', shiftKey: true }), true)
  assert.equal(isBrowserRefreshShortcut({ key: 'F5', ctrlKey: true, shiftKey: true }), true)
  assert.equal(isBrowserRefreshShortcut({ key: 'F5', metaKey: true }), true)
})

test('isBrowserRefreshShortcut detects Ctrl+R and Cmd+R in upper and lower case', () => {
  assert.equal(isBrowserRefreshShortcut({ key: 'r', ctrlKey: true }), true)
  assert.equal(isBrowserRefreshShortcut({ key: 'R', ctrlKey: true }), true)
  assert.equal(isBrowserRefreshShortcut({ key: 'r', metaKey: true }), true)
  assert.equal(isBrowserRefreshShortcut({ key: 'R', metaKey: true }), true)
})

test('isBrowserRefreshShortcut detects hard-refresh combinations', () => {
  assert.equal(isBrowserRefreshShortcut({ key: 'r', ctrlKey: true, shiftKey: true }), true)
  assert.equal(isBrowserRefreshShortcut({ key: 'R', ctrlKey: true, shiftKey: true }), true)
  assert.equal(isBrowserRefreshShortcut({ key: 'r', metaKey: true, shiftKey: true }), true)
  assert.equal(isBrowserRefreshShortcut({ key: 'R', metaKey: true, shiftKey: true }), true)
  assert.equal(isBrowserRefreshShortcut({ key: 'F5', ctrlKey: true }), true)
  assert.equal(isBrowserRefreshShortcut({ key: 'F5', shiftKey: true }), true)
})

test('isBrowserRefreshShortcut returns false for regular typing without reload modifiers', () => {
  assert.equal(isBrowserRefreshShortcut({ key: 'r' }), false)
  assert.equal(isBrowserRefreshShortcut({ key: 'R' }), false)
  assert.equal(isBrowserRefreshShortcut({ key: 'r', shiftKey: true }), false)
  assert.equal(isBrowserRefreshShortcut({ key: 'R', shiftKey: true }), false)
  assert.equal(isBrowserRefreshShortcut({ key: 'r', altKey: true }), false)
  assert.equal(isBrowserRefreshShortcut({ key: 'R', altKey: true }), false)
})

test('isBrowserRefreshShortcut returns false for unrelated shortcuts and other keys', () => {
  assert.equal(isBrowserRefreshShortcut({ key: 'w', ctrlKey: true }), false)
  assert.equal(isBrowserRefreshShortcut({ key: 'k', ctrlKey: true }), false)
  assert.equal(isBrowserRefreshShortcut({ key: 'c', ctrlKey: true }), false)
  assert.equal(isBrowserRefreshShortcut({ key: 'v', ctrlKey: true }), false)
  assert.equal(isBrowserRefreshShortcut({ key: 'f', ctrlKey: true }), false)
  assert.equal(isBrowserRefreshShortcut({ key: 'F4' }), false)
  assert.equal(isBrowserRefreshShortcut({ key: 'F6' }), false)
  assert.equal(isBrowserRefreshShortcut({ key: 'F11' }), false)
  assert.equal(isBrowserRefreshShortcut({ key: 'F12' }), false)
  assert.equal(isBrowserRefreshShortcut({ key: 'Enter' }), false)
  assert.equal(isBrowserRefreshShortcut({ key: 'Escape' }), false)
})

test('handlePreventBrowserRefresh calls preventDefault when refresh shortcut matched', () => {
  let preventDefaultCalled = 0
  const event: KeyboardEventLike = {
    key: 'F5',
    preventDefault: () => {
      preventDefaultCalled++
    }
  }

  const handled = handlePreventBrowserRefresh(event)
  assert.equal(handled, true)
  assert.equal(preventDefaultCalled, 1)

  const ctrlREvent: KeyboardEventLike = {
    key: 'r',
    ctrlKey: true,
    preventDefault: () => {
      preventDefaultCalled++
    }
  }
  const handledCtrlR = handlePreventBrowserRefresh(ctrlREvent)
  assert.equal(handledCtrlR, true)
  assert.equal(preventDefaultCalled, 2)
})

test('handlePreventBrowserRefresh does not call preventDefault for non-refresh keys', () => {
  let preventDefaultCalled = 0
  const event: KeyboardEventLike = {
    key: 'r',
    preventDefault: () => {
      preventDefaultCalled++
    }
  }

  const handled = handlePreventBrowserRefresh(event)
  assert.equal(handled, false)
  assert.equal(preventDefaultCalled, 0)

  const ctrlWEvent: KeyboardEventLike = {
    key: 'w',
    ctrlKey: true,
    preventDefault: () => {
      preventDefaultCalled++
    }
  }
  const handledCtrlW = handlePreventBrowserRefresh(ctrlWEvent)
  assert.equal(handledCtrlW, false)
  assert.equal(preventDefaultCalled, 0)
})

test('handlePreventBrowserRefresh handles events without preventDefault gracefully', () => {
  const handledF5 = handlePreventBrowserRefresh({ key: 'F5' })
  assert.equal(handledF5, true)

  const handledOther = handlePreventBrowserRefresh({ key: 'a' })
  assert.equal(handledOther, false)
})
