import test from 'node:test'
import assert from 'node:assert/strict'
import { withTimeout, DEFAULT_STORAGE_TIMEOUT_MS } from '../async.ts'

test('withTimeout resolves when promise settles before timeout', async () => {
  const result = await withTimeout(Promise.resolve('success'), 100)
  assert.equal(result, 'success')
})

test('withTimeout rejects when promise rejects before timeout', async () => {
  await assert.rejects(
    () => withTimeout(Promise.reject(new Error('underlying error')), 100),
    /underlying error/
  )
})

test('withTimeout rejects when operation exceeds timeout', async () => {
  const hangingPromise = new Promise<string>(() => {})
  await assert.rejects(
    () => withTimeout(hangingPromise, 20),
    /Storage operation timed out after 20ms/
  )
})

test('withTimeout respects custom error message', async () => {
  const hangingPromise = new Promise<string>(() => {})
  await assert.rejects(
    () => withTimeout(hangingPromise, 20, 'Custom timeout failure'),
    /Custom timeout failure/
  )
})

test('DEFAULT_STORAGE_TIMEOUT_MS is 1500', () => {
  assert.equal(DEFAULT_STORAGE_TIMEOUT_MS, 1500)
})
