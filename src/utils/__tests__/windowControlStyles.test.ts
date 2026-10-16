import assert from 'node:assert/strict'
import test from 'node:test'
import {
  getMaximizeAriaLabel,
  getMaximizeTooltip,
  getWindowControlButtonClass
} from '../windowControlStyles.ts'

test('getWindowControlButtonClass returns monochrome hover for minimize and maximize', () => {
  const minClass = getWindowControlButtonClass('minimize')
  assert.equal(minClass.includes('hover:bg-(--bg-hover)'), true)
  assert.equal(minClass.includes('hover:text-primary'), true)
  assert.equal(minClass.includes('hover:bg-red'), false)
  assert.equal(minClass.includes('w-11'), true)
  assert.equal(minClass.includes('h-8'), true)
  assert.equal(minClass.includes('rounded-none'), true)

  const maxClass = getWindowControlButtonClass('maximize')
  assert.equal(maxClass.includes('hover:bg-(--bg-hover)'), true)
  assert.equal(maxClass.includes('hover:text-primary'), true)
  assert.equal(maxClass.includes('hover:bg-red'), false)
  assert.equal(maxClass.includes('rounded-none'), true)
})

test('getWindowControlButtonClass returns red danger hover for close button', () => {
  const closeClass = getWindowControlButtonClass('close')
  assert.equal(closeClass.includes('hover:bg-red-500/80'), true)
  assert.equal(closeClass.includes('hover:text-white'), true)
  assert.equal(closeClass.includes('w-11'), true)
  assert.equal(closeClass.includes('h-8'), true)
  assert.equal(closeClass.includes('rounded-none'), true)
})

test('getMaximizeTooltip and getMaximizeAriaLabel return dynamic labels based on state', () => {
  assert.equal(getMaximizeTooltip(false), 'Maximize')
  assert.equal(getMaximizeTooltip(true), 'Restore')

  assert.equal(getMaximizeAriaLabel(false), 'Maximize window')
  assert.equal(getMaximizeAriaLabel(true), 'Restore window')
})
