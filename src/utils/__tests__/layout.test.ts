import test from 'node:test'
import assert from 'node:assert/strict'
import {
  getLayoutMainClass,
  getLayoutContentPaddingClass,
  resolveRouteLayoutMeta,
  ROUTE_LAYOUT_CONFIG
} from '../layout.ts'

test('getLayoutMainClass returns overflow-hidden when fullHeight is true', () => {
  const result = getLayoutMainClass({ fullHeight: true })
  assert.match(result, /flex-1 min-h-0 relative flex flex-col/)
  assert.match(result, /overflow-hidden/)
  assert.doesNotMatch(result, /overflow-y-auto/)
})

test('getLayoutMainClass returns overflow-y-auto when fullHeight is false or omitted', () => {
  const resultFalse = getLayoutMainClass({ fullHeight: false })
  assert.match(resultFalse, /flex-1 min-h-0 relative flex flex-col/)
  assert.match(resultFalse, /overflow-y-auto/)
  assert.doesNotMatch(resultFalse, /overflow-hidden/)

  const resultUndefined = getLayoutMainClass(undefined)
  assert.match(resultUndefined, /flex-1 min-h-0 relative flex flex-col/)
  assert.match(resultUndefined, /overflow-y-auto/)
  assert.doesNotMatch(resultUndefined, /overflow-hidden/)
})

test('getLayoutContentPaddingClass returns p-4 for compact padding', () => {
  const result = getLayoutContentPaddingClass({ padding: 'compact' })
  assert.match(result, /flex-1 min-h-0 flex flex-col/)
  assert.match(result, /p-4/)
  assert.doesNotMatch(result, /p-8/)
  assert.doesNotMatch(result, /p-0/)
})

test('getLayoutContentPaddingClass returns p-0 for none padding', () => {
  const result = getLayoutContentPaddingClass({ padding: 'none' })
  assert.match(result, /flex-1 min-h-0 flex flex-col/)
  assert.match(result, /p-0/)
  assert.doesNotMatch(result, /p-8/)
  assert.doesNotMatch(result, /p-4/)
})

test('getLayoutContentPaddingClass returns p-8 for standard or omitted padding', () => {
  const resultStandard = getLayoutContentPaddingClass({ padding: 'standard' })
  assert.match(resultStandard, /flex-1 min-h-0 flex flex-col/)
  assert.match(resultStandard, /p-8/)

  const resultUndefined = getLayoutContentPaddingClass(undefined)
  assert.match(resultUndefined, /flex-1 min-h-0 flex flex-col/)
  assert.match(resultUndefined, /p-8/)
})

test('resolveRouteLayoutMeta resolves /logs route default config', () => {
  const logsMeta = resolveRouteLayoutMeta('/logs')
  assert.equal(logsMeta.fullHeight, true)
  assert.equal(logsMeta.padding, 'compact')
})

test('resolveRouteLayoutMeta respects meta overrides', () => {
  const customMeta = resolveRouteLayoutMeta('/logs', { padding: 'none' })
  assert.equal(customMeta.fullHeight, true)
  assert.equal(customMeta.padding, 'none')
})

test('resolveRouteLayoutMeta handles unknown paths with empty default', () => {
  const normalMeta = resolveRouteLayoutMeta('/workloads')
  assert.equal(normalMeta.fullHeight, undefined)
  assert.equal(normalMeta.padding, undefined)
})

test('ROUTE_LAYOUT_CONFIG has /logs configured', () => {
  assert.deepEqual(ROUTE_LAYOUT_CONFIG['/logs'], {
    fullHeight: true,
    padding: 'compact'
  })
})
