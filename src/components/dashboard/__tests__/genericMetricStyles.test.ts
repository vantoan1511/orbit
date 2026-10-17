import assert from 'node:assert/strict'
import test from 'node:test'
import {
  getGenericMetricIconClass,
  getGenericMetricSubtextClass,
  getGenericProgressClass
} from '../../../utils/metricCardStyles.ts'

test('generic metric styling helpers return accent CSS variable utilities without hardcoded blue', () => {
  const iconClass = getGenericMetricIconClass()
  assert.ok(iconClass.includes('text-(--accent)'), 'iconClass must use text-(--accent)')
  assert.ok(iconClass.includes('bg-(--accent-soft)'), 'iconClass must use bg-(--accent-soft)')
  assert.ok(!iconClass.includes('blue'), 'iconClass must not contain hardcoded blue')

  const subtextClass = getGenericMetricSubtextClass()
  assert.ok(subtextClass.includes('text-(--accent)'), 'subtextClass must use text-(--accent)')
  assert.ok(!subtextClass.includes('blue'), 'subtextClass must not contain hardcoded blue')

  const progressClass = getGenericProgressClass()
  assert.ok(progressClass.includes('bg-(--accent)'), 'progressClass must use bg-(--accent)')
  assert.ok(!progressClass.includes('blue'), 'progressClass must not contain hardcoded blue')
})
