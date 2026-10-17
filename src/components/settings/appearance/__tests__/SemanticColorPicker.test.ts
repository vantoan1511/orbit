import assert from 'node:assert/strict'
import test from 'node:test'
import {
  SEMANTIC_COLOR_PRESETS,
  SEMANTIC_ROLE_OPTIONS,
  resolveSemanticPalette
} from '../../../../utils/appearanceConstants.ts'
import { useAppearance } from '../../../../composables/useAppearance.ts'

test('SemanticColorPicker dependencies and preset models are properly configured', () => {
  const roles = ['success', 'warning', 'danger', 'info'] as const
  for (const role of roles) {
    assert.ok(SEMANTIC_ROLE_OPTIONS[role].length >= 4)
  }

  assert.deepEqual(SEMANTIC_COLOR_PRESETS.default, {
    success: 'emerald',
    warning: 'amber',
    danger: 'rose',
    info: 'blue'
  })
})

test('resolveSemanticPalette correctly computes active colors for presets and dark mode', () => {
  const lightDefault = resolveSemanticPalette({ preset: 'default' }, false)
  const darkDefault = resolveSemanticPalette({ preset: 'default' }, true)

  assert.equal(lightDefault.success.solid, '#28a745')
  assert.equal(darkDefault.success.solid, '#46d16e')

  const lightColorblind = resolveSemanticPalette({ preset: 'colorblind' }, false)
  assert.equal(lightColorblind.success.solid, '#0f766e')
  assert.equal(lightColorblind.danger.solid, '#dc2626')
})

test('useAppearance actions support changing preset and individual role', async () => {
  const { semanticColors, setSemanticPreset, setSemanticColorRole } = useAppearance()

  await setSemanticPreset('default')
  assert.equal(semanticColors.value.preset, 'default')
  assert.equal(semanticColors.value.success, 'emerald')

  await setSemanticPreset('high-contrast')
  assert.equal(semanticColors.value.preset, 'high-contrast')
  assert.equal(semanticColors.value.success, 'forest')

  await setSemanticColorRole('info', 'cyan')
  assert.equal(semanticColors.value.preset, 'custom')
  assert.equal(semanticColors.value.info, 'cyan')
})
