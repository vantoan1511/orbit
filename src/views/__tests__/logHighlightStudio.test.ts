import test from 'node:test'
import assert from 'node:assert/strict'
import { useLogHighlighting, PRESETS } from '../../composables/useLogHighlighting.ts'

test('reorderRules moves custom rules correctly within range', () => {
  const { customRules, reorderRules } = useLogHighlighting()
  customRules.value = [
    { id: '1', pattern: 'first', color: 'rose', bold: false, caseSensitive: false, isRegex: false },
    {
      id: '2',
      pattern: 'second',
      color: 'amber',
      bold: false,
      caseSensitive: false,
      isRegex: false
    },
    { id: '3', pattern: 'third', color: 'sky', bold: false, caseSensitive: false, isRegex: false }
  ]

  const success = reorderRules(0, 2)
  assert.equal(success, true)
  assert.equal(customRules.value[0].id, '2')
  assert.equal(customRules.value[1].id, '3')
  assert.equal(customRules.value[2].id, '1')

  // Out of bounds checks
  assert.equal(reorderRules(-1, 1), false)
  assert.equal(reorderRules(0, 10), false)
})

test('reorderRules moves rules within active customPreset', () => {
  const { selectedPreset, customPresets, reorderRules } = useLogHighlighting()
  customPresets.value = {
    testPreset: [
      {
        id: 'p1',
        pattern: 'one',
        color: 'rose',
        bold: false,
        caseSensitive: false,
        isRegex: false
      },
      {
        id: 'p2',
        pattern: 'two',
        color: 'amber',
        bold: false,
        caseSensitive: false,
        isRegex: false
      }
    ]
  }
  selectedPreset.value = 'custom:testPreset'

  const success = reorderRules(0, 1)
  assert.equal(success, true)
  assert.equal(customPresets.value.testPreset[0].id, 'p2')
  assert.equal(customPresets.value.testPreset[1].id, 'p1')
})

test('toggleRule flips enabled flag from true/undefined to false and vice-versa', () => {
  const { customRules, toggleRule } = useLogHighlighting()
  customRules.value = [
    { id: 'r1', pattern: 'warn', color: 'amber', bold: false, caseSensitive: false, isRegex: false }
  ]

  // Default undefined enabled should toggle to false
  const res1 = toggleRule('r1')
  assert.equal(res1, true)
  assert.equal(customRules.value[0].enabled, false)

  // Toggle again should become true
  const res2 = toggleRule('r1')
  assert.equal(res2, true)
  assert.equal(customRules.value[0].enabled, true)
})

test('getLogLevelColor ignores disabled rules even if text matches pattern', () => {
  const { customRules, selectedPreset, getLogLevelColor } = useLogHighlighting()
  selectedPreset.value = 'none'
  customRules.value = [
    {
      id: 'r-crit',
      pattern: 'CRITICAL',
      color: 'rose',
      bold: true,
      caseSensitive: true,
      isRegex: false,
      enabled: false
    },
    {
      id: 'r-fallback',
      pattern: 'CRITICAL',
      color: 'sky',
      bold: false,
      caseSensitive: true,
      isRegex: false,
      enabled: true
    }
  ]

  // Since r-crit is disabled, r-fallback should match instead of r-crit
  const color = getLogLevelColor('CRITICAL error happened')
  assert.equal(color.includes('text-sky-500'), true)
  assert.equal(color.includes('text-rose-500'), false)
})

test('clonePreset creates a new custom preset populated with copies of source rules', () => {
  const { selectedPreset, customPresets, clonePreset } = useLogHighlighting()
  selectedPreset.value = 'standard'

  const success = clonePreset('standard', 'My Dev Preset')
  assert.equal(success, true)
  assert.equal(selectedPreset.value, 'custom:My Dev Preset')
  assert.equal(Array.isArray(customPresets.value['My Dev Preset']), true)
  assert.equal(customPresets.value['My Dev Preset'].length, PRESETS.standard.length)
  // Check that cloned rules are non-builtin / editable
  assert.equal(customPresets.value['My Dev Preset'][0].isPreset, true)
  assert.notEqual(customPresets.value['My Dev Preset'][0].id, PRESETS.standard[0].id)
})

test('importRules and exportRules work seamlessly for backup and restore', () => {
  const { customRules, exportRules, importRules } = useLogHighlighting()
  customRules.value = [
    {
      id: 'imp-1',
      pattern: '404 Not Found',
      color: 'amber',
      bold: true,
      caseSensitive: false,
      isRegex: false,
      enabled: true
    }
  ]

  const exported = exportRules()
  assert.equal(typeof exported, 'string')
  assert.equal(exported.includes('404 Not Found'), true)

  const importedSuccess = importRules([
    {
      id: 'new-1',
      pattern: '500 Server Error',
      color: 'rose',
      bold: true,
      caseSensitive: false,
      isRegex: false,
      enabled: true
    }
  ])
  assert.equal(importedSuccess, true)
  assert.equal(customRules.value.length, 1)
  assert.equal(customRules.value[0].pattern, '500 Server Error')
})
