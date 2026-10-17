import assert from 'node:assert/strict'
import test from 'node:test'
import {
  ACCENT_COLOR_DEFINITIONS,
  getAccentPalette,
  getPrimeVuePaletteForAccent,
  type PrimeVueColorPalette
} from '../appearanceConstants.ts'
import type { AccentColor } from '../../types/appearance.ts'

test('getPrimeVuePaletteForAccent returns valid 50-950 palette for all accent colors', () => {
  const accentColors: AccentColor[] = ['blue', 'emerald', 'violet', 'cyan', 'amber', 'rose', 'slate']

  for (const color of accentColors) {
    const palette = getPrimeVuePaletteForAccent(color)
    assert.ok(palette, `Palette for ${color} should exist`)
    const keys = ['50', '100', '200', '300', '400', '500', '600', '700', '800', '900', '950'] as const
    for (const key of keys) {
      assert.ok(palette[key], `Palette for ${color} should have ${key} shade`)
      assert.match(palette[key], /^#[0-9a-fA-F]{6}$/, `Shade ${key} for ${color} should be a valid hex color`)
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
