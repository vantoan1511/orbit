import assert from 'node:assert/strict'
import test from 'node:test'
import { DEFAULT_SEMANTIC_COLORS_PREFERENCES } from '../appearance.ts'
import { CONFIG_KEYS, SPECIALIZED_CONFIG_KEYS, isGeneralConfigKey } from '../settings.ts'

test('DEFAULT_SEMANTIC_COLORS_PREFERENCES defines valid defaults', () => {
  assert.equal(DEFAULT_SEMANTIC_COLORS_PREFERENCES.preset, 'default')
  assert.equal(DEFAULT_SEMANTIC_COLORS_PREFERENCES.success, 'emerald')
  assert.equal(DEFAULT_SEMANTIC_COLORS_PREFERENCES.warning, 'amber')
  assert.equal(DEFAULT_SEMANTIC_COLORS_PREFERENCES.danger, 'rose')
  assert.equal(DEFAULT_SEMANTIC_COLORS_PREFERENCES.info, 'blue')
})

test('SPECIALIZED_CONFIG_KEYS includes CONFIG_KEYS.SEMANTIC_COLORS', () => {
  assert.equal(CONFIG_KEYS.SEMANTIC_COLORS, 'semanticColors')
  assert.ok(SPECIALIZED_CONFIG_KEYS.includes(CONFIG_KEYS.SEMANTIC_COLORS))
  assert.equal(isGeneralConfigKey(CONFIG_KEYS.SEMANTIC_COLORS), false)
})
