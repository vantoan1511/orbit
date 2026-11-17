import assert from 'node:assert/strict'
import test from 'node:test'
import { formatAgeSeconds, parseAgeToSeconds, resolveAgeTickCadence } from '../reactiveAge.ts'

test('resolveAgeTickCadence returns second for s, minute for m, and none for h/d/unknown/empty', () => {
  assert.equal(resolveAgeTickCadence('0s'), 'second')
  assert.equal(resolveAgeTickCadence('45s'), 'second')
  assert.equal(resolveAgeTickCadence(' 12s '), 'second')

  assert.equal(resolveAgeTickCadence('1m'), 'minute')
  assert.equal(resolveAgeTickCadence('59m'), 'minute')

  assert.equal(resolveAgeTickCadence('1h'), 'none')
  assert.equal(resolveAgeTickCadence('24h'), 'none')
  assert.equal(resolveAgeTickCadence('3d'), 'none')
  assert.equal(resolveAgeTickCadence('-'), 'none')
  assert.equal(resolveAgeTickCadence('Unknown'), 'none')
  assert.equal(resolveAgeTickCadence(''), 'none')
  assert.equal(resolveAgeTickCadence(undefined), 'none')
})

test('parseAgeToSeconds parses s, m, h, d units and returns 0 for invalid values', () => {
  assert.equal(parseAgeToSeconds('15s'), 15)
  assert.equal(parseAgeToSeconds('2m'), 120)
  assert.equal(parseAgeToSeconds('3h'), 10800)
  assert.equal(parseAgeToSeconds('1d'), 86400)
  assert.equal(parseAgeToSeconds('-'), 0)
  assert.equal(parseAgeToSeconds('Unknown'), 0)
  assert.equal(parseAgeToSeconds(undefined), 0)
})

test('formatAgeSeconds formats seconds into compact k8s age units', () => {
  assert.equal(formatAgeSeconds(-5), '0s')
  assert.equal(formatAgeSeconds(0), '0s')
  assert.equal(formatAgeSeconds(59), '59s')
  assert.equal(formatAgeSeconds(60), '1m')
  assert.equal(formatAgeSeconds(3599), '59m')
  assert.equal(formatAgeSeconds(3600), '1h')
  assert.equal(formatAgeSeconds(86399), '23h')
  assert.equal(formatAgeSeconds(86400), '1d')
})
