import test from 'node:test'
import assert from 'node:assert/strict'
import {
  useLogHighlighting,
  PRESETS,
  type HighlightRule
} from '../../composables/useLogHighlighting.ts'

test('updateCustomRule successfully updates rule fields in ad-hoc customRules', () => {
  const { customRules, updateCustomRule } = useLogHighlighting()
  customRules.value = [
    {
      id: 'rule-1',
      pattern: 'old-pattern',
      color: 'rose',
      bold: false,
      caseSensitive: false,
      isRegex: false
    }
  ]

  const updated = updateCustomRule('rule-1', {
    pattern: 'new-pattern',
    color: 'emerald',
    bold: true,
    caseSensitive: true,
    isRegex: true
  })

  assert.equal(updated, true)
  assert.equal(customRules.value[0].pattern, 'new-pattern')
  assert.equal(customRules.value[0].color, 'emerald')
  assert.equal(customRules.value[0].bold, true)
  assert.equal(customRules.value[0].caseSensitive, true)
  assert.equal(customRules.value[0].isRegex, true)
})

test('updateCustomRule rejects update with false if pattern is invalid', () => {
  const { customRules, updateCustomRule } = useLogHighlighting()
  customRules.value = [
    {
      id: 'rule-1',
      pattern: 'valid-pattern',
      color: 'rose',
      bold: false,
      caseSensitive: false,
      isRegex: false
    }
  ]

  const emptyResult = updateCustomRule('rule-1', { pattern: '   ' })
  assert.equal(emptyResult, false)
  assert.equal(customRules.value[0].pattern, 'valid-pattern')

  const invalidRegexResult = updateCustomRule('rule-1', {
    pattern: '[unclosed-regex',
    isRegex: true
  })
  assert.equal(invalidRegexResult, false)
  assert.equal(customRules.value[0].pattern, 'valid-pattern')
})

test('updateCustomRule updates rule within active customPreset when custom preset is selected', () => {
  const { selectedPreset, customPresets, updateCustomRule } = useLogHighlighting()
  customPresets.value = {
    teamLogs: [
      {
        id: 'cp-rule-1',
        pattern: 'auth-error',
        color: 'rose',
        bold: false,
        caseSensitive: false,
        isRegex: false,
        isPreset: true
      }
    ]
  }
  selectedPreset.value = 'custom:teamLogs'

  const updated = updateCustomRule('cp-rule-1', {
    pattern: 'auth-(failure|error)',
    color: 'amber',
    isRegex: true
  })

  assert.equal(updated, true)
  const presetRule = customPresets.value.teamLogs[0]
  assert.equal(presetRule.pattern, 'auth-(failure|error)')
  assert.equal(presetRule.color, 'amber')
  assert.equal(presetRule.isRegex, true)
})

test('deleteCustomRule removes rule from active customPreset when custom preset is selected', () => {
  const { selectedPreset, customPresets, deleteCustomRule } = useLogHighlighting()
  customPresets.value = {
    teamLogs: [
      {
        id: 'cp-rule-1',
        pattern: 'to-delete',
        color: 'rose',
        bold: false,
        caseSensitive: false,
        isRegex: false,
        isPreset: true
      },
      {
        id: 'cp-rule-2',
        pattern: 'to-keep',
        color: 'sky',
        bold: false,
        caseSensitive: false,
        isRegex: false,
        isPreset: true
      }
    ]
  }
  selectedPreset.value = 'custom:teamLogs'

  deleteCustomRule('cp-rule-1')
  assert.equal(customPresets.value.teamLogs.length, 1)
  assert.equal(customPresets.value.teamLogs[0].id, 'cp-rule-2')
})

test('isBuiltInRule identifies rules from standard presets as built-in and custom rules as editable', () => {
  const { isBuiltInRule } = useLogHighlighting()
  const standardRule = PRESETS.standard[0]
  assert.equal(isBuiltInRule(standardRule), true)

  const adHocRule: HighlightRule = {
    id: 'ad-hoc-1',
    pattern: 'panic',
    color: 'rose',
    bold: false,
    caseSensitive: false,
    isRegex: false
  }
  assert.equal(isBuiltInRule(adHocRule), false)

  const customPresetRule: HighlightRule = {
    id: 'preset-rule-1',
    pattern: 'panic',
    color: 'rose',
    bold: false,
    caseSensitive: false,
    isRegex: false,
    isPreset: true
  }
  assert.equal(isBuiltInRule(customPresetRule), false)
})
