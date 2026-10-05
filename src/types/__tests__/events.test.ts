import assert from 'node:assert/strict'
import test from 'node:test'
import { OrbitEvents } from '../events.ts'

test('OrbitEvents contains ServerOffline event identifier', () => {
  assert.equal(OrbitEvents.ServerOffline, 'serverOffline')
})
