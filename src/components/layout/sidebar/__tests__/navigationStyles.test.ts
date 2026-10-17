import assert from 'node:assert/strict'
import test from 'node:test'
import {
  getActivityBarItemClass,
  getActivityBarIndicatorClass,
  getNavMenuItemClass
} from '../navigationStyles.ts'

test('getActivityBarItemClass returns accent styling when active', () => {
  const activeClass = getActivityBarItemClass(true, false)
  assert.ok(activeClass.includes('text-(--accent)'), 'active item must use accent text')
  assert.ok(
    activeClass.includes('bg-(--accent-soft)'),
    'active item must use accent soft background'
  )

  const inactiveClass = getActivityBarItemClass(false, false)
  assert.ok(inactiveClass.includes('text-muted-color'), 'inactive item must use muted text')
  assert.ok(!inactiveClass.includes('text-(--accent)'), 'inactive item must not use accent text')

  const disabledClass = getActivityBarItemClass(false, true)
  assert.ok(disabledClass.includes('opacity-40'), 'disabled item must have reduced opacity')
})

test('getActivityBarIndicatorClass returns active indicator bar when active', () => {
  const activeIndicator = getActivityBarIndicatorClass(true)
  assert.ok(activeIndicator.includes('bg-(--accent)'), 'indicator must use accent color')
  assert.ok(!activeIndicator.includes('hidden'), 'indicator must be visible when active')

  const inactiveIndicator = getActivityBarIndicatorClass(false)
  assert.ok(inactiveIndicator.includes('hidden'), 'indicator must be hidden when inactive')
})

test('getNavMenuItemClass returns accent styling when active', () => {
  const activeClass = getNavMenuItemClass(true)
  assert.ok(activeClass.includes('text-(--accent)'), 'active nav link must use accent text')
  assert.ok(
    activeClass.includes('bg-(--accent-soft)'),
    'active nav link must use accent soft background'
  )

  const inactiveClass = getNavMenuItemClass(false)
  assert.ok(inactiveClass.includes('text-muted-color'), 'inactive nav link must use muted text')
  assert.ok(
    !inactiveClass.includes('text-(--accent)'),
    'inactive nav link must not use accent text'
  )
})
