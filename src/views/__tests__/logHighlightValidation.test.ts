import test from 'node:test'
import assert from 'node:assert/strict'
import { validateRule } from '../../composables/useLogHighlighting.ts'

test('validateRule returns invalid when pattern is empty or whitespace', () => {
  const emptyRes = validateRule('', false, false)
  assert.equal(emptyRes.valid, false)
  assert.equal(emptyRes.error, 'Pattern cannot be empty')

  const whitespaceRes = validateRule('   ', false, false)
  assert.equal(whitespaceRes.valid, false)
  assert.equal(whitespaceRes.error, 'Pattern cannot be empty')
})

test('validateRule returns valid for standard text pattern', () => {
  const res = validateRule('connection reset', false, false)
  assert.equal(res.valid, true)
  assert.equal(res.error, undefined)
})

test('validateRule returns valid for well-formed regex pattern', () => {
  const res = validateRule('\\b(error|fatal|fail)\\b', true, false)
  assert.equal(res.valid, true)
  assert.equal(res.error, undefined)
})

test('validateRule returns invalid with descriptive error for malformed regex pattern', () => {
  const res = validateRule('\\b(error|fatal|fail', true, false)
  assert.equal(res.valid, false)
  assert.ok(typeof res.error === 'string' && res.error.length > 0)
})
