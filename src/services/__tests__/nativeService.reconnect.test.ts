import assert from 'node:assert/strict'
import test from 'node:test'
import { extensions, probe, reconnect } from '../nativeService.ts'

test('probe resolves true when extensions.getStats resolves quickly', async () => {
  const originalGetStats = (extensions as unknown as { getStats?: () => Promise<unknown> }).getStats

  ;(extensions as unknown as { getStats?: () => Promise<unknown> }).getStats = async () => {
    return {
      loaded: ['vantoan1511.orbit.core.engine'],
      connected: ['vantoan1511.orbit.core.engine']
    }
  }

  try {
    const result = await probe(100)
    assert.equal(result, true)
  } finally {
    ;(extensions as unknown as { getStats?: () => Promise<unknown> }).getStats = originalGetStats
  }
})

test('probe resolves false when extensions.getStats hangs or times out', async () => {
  const originalGetStats = (extensions as unknown as { getStats?: () => Promise<unknown> }).getStats

  ;(extensions as unknown as { getStats?: () => Promise<unknown> }).getStats = () => {
    return new Promise(() => {}) // never resolves
  }

  try {
    const result = await probe(20)
    assert.equal(result, false)
  } finally {
    ;(extensions as unknown as { getStats?: () => Promise<unknown> }).getStats = originalGetStats
  }
})

test('probe resolves false when extensions.getStats rejects', async () => {
  const originalGetStats = (extensions as unknown as { getStats?: () => Promise<unknown> }).getStats

  ;(extensions as unknown as { getStats?: () => Promise<unknown> }).getStats = async () => {
    throw new Error('WebSocket connection is closed')
  }

  try {
    const result = await probe(100)
    assert.equal(result, false)
  } finally {
    ;(extensions as unknown as { getStats?: () => Promise<unknown> }).getStats = originalGetStats
  }
})

test('reconnect can be called safely without throwing', () => {
  assert.doesNotThrow(() => {
    reconnect()
  })
})
