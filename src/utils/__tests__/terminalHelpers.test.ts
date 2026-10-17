import assert from 'node:assert/strict'
import test from 'node:test'
import {
  getTerminalPanelTransitionClass,
  shouldDispatchTerminalResize
} from '../terminalHelpers.ts'

test('shouldDispatchTerminalResize returns false for null, undefined, or non-positive dimensions', () => {
  assert.equal(shouldDispatchTerminalResize(null, undefined), false)
  assert.equal(shouldDispatchTerminalResize(undefined, undefined), false)
  assert.equal(shouldDispatchTerminalResize({ cols: 0, rows: 24 }, undefined), false)
  assert.equal(shouldDispatchTerminalResize({ cols: 80, rows: 0 }, undefined), false)
  assert.equal(shouldDispatchTerminalResize({ cols: -1, rows: 24 }, undefined), false)
  assert.equal(shouldDispatchTerminalResize({ cols: 80, rows: -5 }, undefined), false)
})

test('shouldDispatchTerminalResize returns true when lastDims is undefined/null and newDims is valid', () => {
  assert.equal(shouldDispatchTerminalResize({ cols: 80, rows: 24 }, undefined), true)
  assert.equal(shouldDispatchTerminalResize({ cols: 80, rows: 24 }, null), true)
})

test('shouldDispatchTerminalResize returns false when newDims matches lastDims exactly', () => {
  assert.equal(shouldDispatchTerminalResize({ cols: 80, rows: 24 }, { cols: 80, rows: 24 }), false)
  assert.equal(
    shouldDispatchTerminalResize({ cols: 120, rows: 40 }, { cols: 120, rows: 40 }),
    false
  )
})

test('shouldDispatchTerminalResize returns true when cols or rows differ', () => {
  assert.equal(shouldDispatchTerminalResize({ cols: 85, rows: 24 }, { cols: 80, rows: 24 }), true)
  assert.equal(shouldDispatchTerminalResize({ cols: 80, rows: 25 }, { cols: 80, rows: 24 }), true)
})

test('getTerminalPanelTransitionClass returns transition-none when isDragging is true', () => {
  assert.equal(getTerminalPanelTransitionClass(true), 'transition-none')
})

test('getTerminalPanelTransitionClass returns smooth height transition when isDragging is false', () => {
  assert.equal(getTerminalPanelTransitionClass(false), 'transition-[height] duration-150 ease-out')
})
