import test from 'node:test'
import assert from 'node:assert/strict'
import {
  isBrowserRefreshShortcut,
  handlePreventBrowserRefresh,
  isBrowserPrintShortcut,
  isBrowserSaveOrOpenShortcut,
  isBrowserViewSourceShortcut,
  isBrowserNavigationShortcut,
  isBrowserPromptShortcut,
  isBrowserZoomShortcut,
  isBrowserDevToolsShortcut,
  isBlockedBrowserShortcut,
  handlePreventBrowserShortcuts,
  handlePreventBrowserZoomWheel,
  type KeyboardEventLike,
  type WheelEventLike
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

test('isBrowserPrintShortcut detects Ctrl+P, Ctrl+Shift+P, Cmd+P, and Cmd+Shift+P', () => {
  assert.equal(isBrowserPrintShortcut({ key: 'p', ctrlKey: true }), true)
  assert.equal(isBrowserPrintShortcut({ key: 'P', ctrlKey: true }), true)
  assert.equal(isBrowserPrintShortcut({ key: 'p', ctrlKey: true, shiftKey: true }), true)
  assert.equal(isBrowserPrintShortcut({ key: 'P', ctrlKey: true, shiftKey: true }), true)
  assert.equal(isBrowserPrintShortcut({ key: 'p', metaKey: true }), true)
  assert.equal(isBrowserPrintShortcut({ key: 'P', metaKey: true, shiftKey: true }), true)

  // False for normal p typing
  assert.equal(isBrowserPrintShortcut({ key: 'p' }), false)
  assert.equal(isBrowserPrintShortcut({ key: 'P' }), false)
  assert.equal(isBrowserPrintShortcut({ key: 'p', shiftKey: true }), false)
  assert.equal(isBrowserPrintShortcut({ key: 'p', altKey: true }), false)
})

test('isBrowserSaveOrOpenShortcut detects Ctrl+S, Cmd+S, Ctrl+O, Cmd+O', () => {
  assert.equal(isBrowserSaveOrOpenShortcut({ key: 's', ctrlKey: true }), true)
  assert.equal(isBrowserSaveOrOpenShortcut({ key: 'S', ctrlKey: true }), true)
  assert.equal(isBrowserSaveOrOpenShortcut({ key: 's', metaKey: true }), true)
  assert.equal(isBrowserSaveOrOpenShortcut({ key: 'o', ctrlKey: true }), true)
  assert.equal(isBrowserSaveOrOpenShortcut({ key: 'O', ctrlKey: true }), true)
  assert.equal(isBrowserSaveOrOpenShortcut({ key: 'o', metaKey: true }), true)

  // False for normal typing
  assert.equal(isBrowserSaveOrOpenShortcut({ key: 's' }), false)
  assert.equal(isBrowserSaveOrOpenShortcut({ key: 'o' }), false)
})

test('isBrowserViewSourceShortcut detects Ctrl+U and Cmd+Option+U', () => {
  assert.equal(isBrowserViewSourceShortcut({ key: 'u', ctrlKey: true }), true)
  assert.equal(isBrowserViewSourceShortcut({ key: 'U', ctrlKey: true }), true)
  assert.equal(isBrowserViewSourceShortcut({ key: 'u', metaKey: true, altKey: true }), true)
  assert.equal(isBrowserViewSourceShortcut({ key: 'u' }), false)
})

test('isBrowserNavigationShortcut detects Ctrl+N, Ctrl+Shift+N, Ctrl+T, Ctrl+Shift+T, Ctrl+D, Ctrl+Shift+O, Ctrl+H, Ctrl+J, Ctrl+Shift+Delete, Alt+Left, Alt+Right', () => {
  assert.equal(isBrowserNavigationShortcut({ key: 'n', ctrlKey: true }), true)
  assert.equal(isBrowserNavigationShortcut({ key: 'N', ctrlKey: true, shiftKey: true }), true)
  assert.equal(isBrowserNavigationShortcut({ key: 'n', metaKey: true }), true)
  assert.equal(isBrowserNavigationShortcut({ key: 't', ctrlKey: true }), true)
  assert.equal(isBrowserNavigationShortcut({ key: 'T', ctrlKey: true, shiftKey: true }), true)
  assert.equal(isBrowserNavigationShortcut({ key: 't', metaKey: true }), true)
  assert.equal(isBrowserNavigationShortcut({ key: 'd', ctrlKey: true }), true)
  assert.equal(isBrowserNavigationShortcut({ key: 'd', metaKey: true }), true)
  assert.equal(isBrowserNavigationShortcut({ key: 'o', ctrlKey: true, shiftKey: true }), true)
  assert.equal(isBrowserNavigationShortcut({ key: 'b', metaKey: true, altKey: true }), true)
  assert.equal(isBrowserNavigationShortcut({ key: 'h', ctrlKey: true }), true)
  assert.equal(isBrowserNavigationShortcut({ key: 'y', metaKey: true }), true)
  assert.equal(isBrowserNavigationShortcut({ key: 'j', ctrlKey: true }), true)
  assert.equal(isBrowserNavigationShortcut({ key: 'l', metaKey: true, altKey: true }), true)
  assert.equal(isBrowserNavigationShortcut({ key: 'Delete', ctrlKey: true, shiftKey: true }), true)
  assert.equal(
    isBrowserNavigationShortcut({ key: 'Backspace', ctrlKey: true, shiftKey: true }),
    true
  )
  assert.equal(isBrowserNavigationShortcut({ key: 'ArrowLeft', altKey: true }), true)
  assert.equal(isBrowserNavigationShortcut({ key: 'ArrowRight', altKey: true }), true)
  assert.equal(isBrowserNavigationShortcut({ key: '[', metaKey: true }), true)
  assert.equal(isBrowserNavigationShortcut({ key: ']', metaKey: true }), true)

  // False for normal keys
  assert.equal(isBrowserNavigationShortcut({ key: 'ArrowLeft' }), false)
  assert.equal(isBrowserNavigationShortcut({ key: 't' }), false)
})

test('isBrowserPromptShortcut detects F7 caret browsing and F1 help', () => {
  assert.equal(isBrowserPromptShortcut({ key: 'F7' }), true)
  assert.equal(isBrowserPromptShortcut({ key: 'F1' }), true)
  assert.equal(isBrowserPromptShortcut({ key: 'F2' }), false)
  assert.equal(isBrowserPromptShortcut({ key: 'F8' }), false)
})

test('isBrowserZoomShortcut detects Ctrl/Cmd with +, -, and 0', () => {
  assert.equal(isBrowserZoomShortcut({ key: '+', ctrlKey: true }), true)
  assert.equal(isBrowserZoomShortcut({ key: '=', ctrlKey: true }), true)
  assert.equal(isBrowserZoomShortcut({ key: '=', ctrlKey: true, shiftKey: true }), true)
  assert.equal(isBrowserZoomShortcut({ key: '-', ctrlKey: true }), true)
  assert.equal(isBrowserZoomShortcut({ key: '_', ctrlKey: true }), true)
  assert.equal(isBrowserZoomShortcut({ key: '0', ctrlKey: true }), true)
  assert.equal(isBrowserZoomShortcut({ key: '0', metaKey: true }), true)
  assert.equal(isBrowserZoomShortcut({ key: '+', metaKey: true }), true)
  assert.equal(isBrowserZoomShortcut({ key: '-', metaKey: true }), true)

  // False without modifiers
  assert.equal(isBrowserZoomShortcut({ key: '+' }), false)
  assert.equal(isBrowserZoomShortcut({ key: '-' }), false)
  assert.equal(isBrowserZoomShortcut({ key: '0' }), false)
})

test('isBrowserDevToolsShortcut detects F12, Ctrl+Shift+I, Ctrl+Shift+J, Ctrl+Shift+C', () => {
  assert.equal(isBrowserDevToolsShortcut({ key: 'F12' }), true)
  assert.equal(isBrowserDevToolsShortcut({ key: 'i', ctrlKey: true, shiftKey: true }), true)
  assert.equal(isBrowserDevToolsShortcut({ key: 'I', ctrlKey: true, shiftKey: true }), true)
  assert.equal(isBrowserDevToolsShortcut({ key: 'j', ctrlKey: true, shiftKey: true }), true)
  assert.equal(isBrowserDevToolsShortcut({ key: 'c', ctrlKey: true, shiftKey: true }), true)
  assert.equal(isBrowserDevToolsShortcut({ key: 'i', metaKey: true, altKey: true }), true)
  assert.equal(isBrowserDevToolsShortcut({ key: 'j', metaKey: true, altKey: true }), true)
  assert.equal(isBrowserDevToolsShortcut({ key: 'c', metaKey: true, altKey: true }), true)

  // False for normal keys or solitary modifiers
  assert.equal(isBrowserDevToolsShortcut({ key: 'i', ctrlKey: true }), false)
  assert.equal(isBrowserDevToolsShortcut({ key: 'F11' }), false)
})

test('isBlockedBrowserShortcut respects allowDevTools option', () => {
  // DevTools allowed when allowDevTools: true
  assert.equal(isBlockedBrowserShortcut({ key: 'F12' }, { allowDevTools: true }), false)
  assert.equal(
    isBlockedBrowserShortcut({ key: 'I', ctrlKey: true, shiftKey: true }, { allowDevTools: true }),
    false
  )

  // DevTools blocked when allowDevTools: false or omitted
  assert.equal(isBlockedBrowserShortcut({ key: 'F12' }), true)
  assert.equal(isBlockedBrowserShortcut({ key: 'F12' }, { allowDevTools: false }), true)
  assert.equal(
    isBlockedBrowserShortcut({ key: 'I', ctrlKey: true, shiftKey: true }, { allowDevTools: false }),
    true
  )

  // Other blocked shortcuts remain blocked regardless of allowDevTools
  assert.equal(
    isBlockedBrowserShortcut({ key: 'p', ctrlKey: true, shiftKey: true }, { allowDevTools: true }),
    true
  )
  assert.equal(isBlockedBrowserShortcut({ key: 's', ctrlKey: true }, { allowDevTools: true }), true)
  assert.equal(isBlockedBrowserShortcut({ key: 'F7' }, { allowDevTools: true }), true)
  assert.equal(isBlockedBrowserShortcut({ key: 'F5' }, { allowDevTools: true }), true)
})

test('isBlockedBrowserShortcut returns false for normal typing and text editing (Ctrl+A, Ctrl+C, Ctrl+V, Ctrl+X, Ctrl+Z, Ctrl+Y)', () => {
  assert.equal(isBlockedBrowserShortcut({ key: 'a', ctrlKey: true }), false)
  assert.equal(isBlockedBrowserShortcut({ key: 'c', ctrlKey: true }), false)
  assert.equal(isBlockedBrowserShortcut({ key: 'v', ctrlKey: true }), false)
  assert.equal(isBlockedBrowserShortcut({ key: 'x', ctrlKey: true }), false)
  assert.equal(isBlockedBrowserShortcut({ key: 'z', ctrlKey: true }), false)
  assert.equal(isBlockedBrowserShortcut({ key: 'y', ctrlKey: true }), false)
  assert.equal(isBlockedBrowserShortcut({ key: 'z', ctrlKey: true, shiftKey: true }), false)
  assert.equal(isBlockedBrowserShortcut({ key: 'a' }), false)
  assert.equal(isBlockedBrowserShortcut({ key: 'Enter' }), false)
  assert.equal(isBlockedBrowserShortcut({ key: 'Escape' }), false)
  assert.equal(isBlockedBrowserShortcut({ key: 'Tab' }), false)
})

test('handlePreventBrowserShortcuts executes preventDefault on matched events', () => {
  let preventDefaultCalls = 0
  const event: KeyboardEventLike = {
    key: 'p',
    ctrlKey: true,
    shiftKey: true,
    preventDefault: () => {
      preventDefaultCalls++
    }
  }

  const handled = handlePreventBrowserShortcuts(event)
  assert.equal(handled, true)
  assert.equal(preventDefaultCalls, 1)

  // Non-blocked shortcut
  const copyEvent: KeyboardEventLike = {
    key: 'c',
    ctrlKey: true,
    preventDefault: () => {
      preventDefaultCalls++
    }
  }
  const handledCopy = handlePreventBrowserShortcuts(copyEvent)
  assert.equal(handledCopy, false)
  assert.equal(preventDefaultCalls, 1)
})

test('handlePreventBrowserZoomWheel cancels wheel when ctrlKey or metaKey is pressed', () => {
  let preventDefaultCalls = 0
  const zoomWheelEvent: WheelEventLike = {
    ctrlKey: true,
    preventDefault: () => {
      preventDefaultCalls++
    }
  }

  assert.equal(handlePreventBrowserZoomWheel(zoomWheelEvent), true)
  assert.equal(preventDefaultCalls, 1)

  const macZoomWheelEvent: WheelEventLike = {
    metaKey: true,
    preventDefault: () => {
      preventDefaultCalls++
    }
  }

  assert.equal(handlePreventBrowserZoomWheel(macZoomWheelEvent), true)
  assert.equal(preventDefaultCalls, 2)

  const normalScrollWheelEvent: WheelEventLike = {
    preventDefault: () => {
      preventDefaultCalls++
    }
  }

  assert.equal(handlePreventBrowserZoomWheel(normalScrollWheelEvent), false)
  assert.equal(preventDefaultCalls, 2)
})
