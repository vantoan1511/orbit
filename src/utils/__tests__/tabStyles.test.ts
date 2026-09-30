import assert from 'node:assert/strict'
import test from 'node:test'
import { getTabCloseButtonClass, getTabContainerClass } from '../tabStyles.ts'

test('getTabContainerClass returns active tab styling without blue border-t-(--accent)', () => {
  const activeClass = getTabContainerClass(true)

  // Anti-patterns: no decorative blue top border
  assert.equal(activeClass.includes('border-t-(--accent)'), false)
  assert.equal(activeClass.includes('border-t-2'), false)

  // Desired monochrome/noir Nora principles
  assert.equal(activeClass.includes('bg-(--bg-card)'), true)
  assert.equal(activeClass.includes('text-primary'), true)
  assert.equal(activeClass.includes('font-medium'), true)
  assert.equal(activeClass.includes('border-r'), true)
  assert.equal(activeClass.includes('border-(--border)'), true)
  assert.equal(activeClass.includes('border-b-(--bg-card)'), true)
})

test('getTabContainerClass returns inactive tab styling', () => {
  const inactiveClass = getTabContainerClass(false)

  // Inactive tab should be recessed and muted
  assert.equal(inactiveClass.includes('bg-(--bg-sidebar)/70'), true)
  assert.equal(inactiveClass.includes('text-muted-color'), true)
  assert.equal(inactiveClass.includes('hover:bg-(--bg-hover)/60'), true)
  assert.equal(inactiveClass.includes('hover:text-primary'), true)
  assert.equal(inactiveClass.includes('border-b-(--border)'), true)
})

test('getTabCloseButtonClass returns visible opacity on active tab', () => {
  const activeBtnClass = getTabCloseButtonClass(true)

  assert.equal(activeBtnClass.includes('opacity-100'), true)
  assert.equal(activeBtnClass.includes('opacity-0'), false)
})

test('getTabCloseButtonClass returns hover-only opacity on inactive tab', () => {
  const inactiveBtnClass = getTabCloseButtonClass(false)

  assert.equal(inactiveBtnClass.includes('opacity-0'), true)
  assert.equal(inactiveBtnClass.includes('group-hover:opacity-100'), true)
  assert.equal(inactiveBtnClass.includes('opacity-100 group-hover:opacity-100'), false)
})
