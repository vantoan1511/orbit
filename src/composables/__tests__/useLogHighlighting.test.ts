import assert from 'node:assert/strict'
import test from 'node:test'
import { useLogHighlighting, clearRegexCacheForTesting } from '../useLogHighlighting.ts'

test('getLogLevelColor does not create new RegExp instances for unchanged rules', () => {
  const { getLogLevelColor } = useLogHighlighting()

  clearRegexCacheForTesting?.()

  let regExpInstantiations = 0
  const OriginalRegExp = globalThis.RegExp

  // Proxy or wrap RegExp constructor
  const RegExpProxy = function (pattern: string | RegExp, flags?: string) {
    regExpInstantiations++
    return new OriginalRegExp(pattern, flags)
  }
  RegExpProxy.prototype = OriginalRegExp.prototype
  Object.assign(RegExpProxy, OriginalRegExp)

  globalThis.RegExp = RegExpProxy as unknown as RegExpConstructor

  try {
    for (let i = 0; i < 100; i++) {
      getLogLevelColor(`line ${i}: info server listening on 8080`)
    }

    // Standard preset has 4 rules. Calling 100 times should NOT create 100 * 4 = 400 RegExps.
    // It should create at most 4 (or number of regex rules in the active preset).
    assert.ok(
      regExpInstantiations <= 10,
      `Expected RegExp instantiations to be <= 10 due to caching, got ${regExpInstantiations}`
    )
  } finally {
    globalThis.RegExp = OriginalRegExp
    clearRegexCacheForTesting?.()
  }
})
