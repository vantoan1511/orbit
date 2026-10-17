import assert from 'node:assert/strict'
import test from 'node:test'
import { getPrimeVuePaletteForAccent } from '../appearanceConstants.ts'
import type { AccentColor } from '../../types/appearance.ts'

test('getPrimeVuePaletteForAccent returns valid 50-950 palette for all accent colors', () => {
  const accentColors: AccentColor[] = [
    'blue',
    'emerald',
    'violet',
    'cyan',
    'amber',
    'rose',
    'slate'
  ]

  for (const color of accentColors) {
    const palette = getPrimeVuePaletteForAccent(color)
    assert.ok(palette, `Palette for ${color} should exist`)
    const keys = [
      '50',
      '100',
      '200',
      '300',
      '400',
      '500',
      '600',
      '700',
      '800',
      '900',
      '950'
    ] as const
    for (const key of keys) {
      assert.ok(palette[key], `Palette for ${color} should have ${key} shade`)
      assert.match(
        palette[key],
        /^#[0-9a-fA-F]{6}$/,
        `Shade ${key} for ${color} should be a valid hex color`
      )
    }
  }
})

test('getPrimeVuePaletteForAccent falls back gracefully on unknown accent color', () => {
  // @ts-expect-error testing invalid input
  const fallback = getPrimeVuePaletteForAccent('unknown_color')
  assert.ok(fallback)
  assert.ok(fallback['500'])
  // Should match default blue palette
  const blue = getPrimeVuePaletteForAccent('blue')
  assert.equal(fallback['500'], blue['500'])
})

test('SEMANTIC_ROLE_OPTIONS provides at least 4 options per role with valid light and dark shades', async () => {
  const { SEMANTIC_ROLE_OPTIONS } = await import('../appearanceConstants.ts')
  const roles = ['success', 'warning', 'danger', 'info'] as const
  for (const role of roles) {
    const options = SEMANTIC_ROLE_OPTIONS[role]
    assert.ok(Array.isArray(options), `SEMANTIC_ROLE_OPTIONS[${role}] should be an array`)
    assert.ok(options.length >= 4, `Role ${role} should have at least 4 options`)
    for (const opt of options) {
      assert.ok(opt.id, 'Option should have an id')
      assert.ok(opt.name, 'Option should have a name')
      assert.match(opt.light.solid, /^#[0-9a-fA-F]{6}$/)
      assert.match(opt.dark.solid, /^#[0-9a-fA-F]{6}$/)
      assert.ok(opt.light.soft.startsWith('rgba('))
      assert.ok(opt.dark.soft.startsWith('rgba('))
    }
  }
})

test('SEMANTIC_COLOR_PRESETS defines default, colorblind, and high-contrast palettes', async () => {
  const { SEMANTIC_COLOR_PRESETS } = await import('../appearanceConstants.ts')
  assert.ok(SEMANTIC_COLOR_PRESETS['default'])
  assert.ok(SEMANTIC_COLOR_PRESETS['colorblind'])
  assert.ok(SEMANTIC_COLOR_PRESETS['high-contrast'])
  assert.equal(SEMANTIC_COLOR_PRESETS['default'].success, 'emerald')
})

test('resolveSemanticPalette returns standard default palette when preferences are undefined', async () => {
  const { resolveSemanticPalette } = await import('../appearanceConstants.ts')
  const light = resolveSemanticPalette(undefined, false)
  assert.ok(light.success.solid)
  assert.ok(light.warning.solid)
  assert.ok(light.danger.solid)
  assert.ok(light.info.solid)

  const dark = resolveSemanticPalette(undefined, true)
  assert.notEqual(light.success.solid, dark.success.solid)
})

test('resolveSemanticPalette resolves colorblind preset values accurately', async () => {
  const { SEMANTIC_ROLE_OPTIONS, SEMANTIC_COLOR_PRESETS, resolveSemanticPalette } =
    await import('../appearanceConstants.ts')
  const palette = resolveSemanticPalette({ preset: 'colorblind' }, false)
  const colorblindDef = SEMANTIC_COLOR_PRESETS['colorblind']
  const tealOpt = SEMANTIC_ROLE_OPTIONS.success.find((o) => o.id === colorblindDef.success)!
  assert.equal(palette.success.solid, tealOpt.light.solid)
})

test('resolveSemanticPalette applies custom overrides to individual roles', async () => {
  const { SEMANTIC_ROLE_OPTIONS, resolveSemanticPalette } =
    await import('../appearanceConstants.ts')
  const palette = resolveSemanticPalette(
    {
      preset: 'custom',
      success: 'mint',
      warning: 'amber',
      danger: 'crimson',
      info: 'cyan'
    },
    false
  )
  const mintOpt = SEMANTIC_ROLE_OPTIONS.success.find((o) => o.id === 'mint')!
  const crimsonOpt = SEMANTIC_ROLE_OPTIONS.danger.find((o) => o.id === 'crimson')!
  assert.equal(palette.success.solid, mintOpt.light.solid)
  assert.equal(palette.danger.solid, crimsonOpt.light.solid)
})
