import test from 'node:test'
import assert from 'node:assert/strict'
import {
  ACCENT_COLOR_DEFINITIONS,
  UI_STYLE_DEFINITIONS,
  getAccentPalette,
  formatThemeModeLabel,
  formatUiStyleLabel,
  isTranslucentStyle,
  getUiStyleTokens
} from '../appearanceConstants.ts'
import type { AccentColor, UiStyle } from '../../types/appearance.ts'

test('ACCENT_COLOR_DEFINITIONS defines all 7 curated accent colors with light and dark palettes', () => {
  assert.equal(ACCENT_COLOR_DEFINITIONS.length, 7)
  const expectedIds: AccentColor[] = ['blue', 'emerald', 'violet', 'cyan', 'amber', 'rose', 'slate']

  for (const expectedId of expectedIds) {
    const def = ACCENT_COLOR_DEFINITIONS.find((c) => c.id === expectedId)
    assert.ok(def, `Missing color definition for ${expectedId}`)
    assert.ok(def.name.length > 0)
    assert.ok(def.light.accent.startsWith('#'))
    assert.ok(def.light.hover.startsWith('#'))
    assert.ok(def.light.active.startsWith('#'))
    assert.ok(def.light.soft.startsWith('rgba('))
    assert.ok(def.dark.accent.startsWith('#'))
    assert.ok(def.dark.hover.startsWith('#'))
    assert.ok(def.dark.active.startsWith('#'))
    assert.ok(def.dark.soft.startsWith('rgba('))
  }
})

test('UI_STYLE_DEFINITIONS contains all 5 supported UI styles with names and descriptions', () => {
  assert.equal(UI_STYLE_DEFINITIONS.length, 5)
  const expectedStyles: UiStyle[] = ['flat', 'glass', 'aero', 'liquidglass', 'ios']

  for (const expectedStyle of expectedStyles) {
    const def = UI_STYLE_DEFINITIONS.find((s) => s.id === expectedStyle)
    assert.ok(def, `Missing UI style definition for ${expectedStyle}`)
    assert.ok(def.name.length > 0)
    assert.ok(def.description.length > 0)
  }
})

test('getAccentPalette returns valid palette for specified color and mode', () => {
  const lightBlue = getAccentPalette('blue', false)
  assert.equal(lightBlue.accent, '#4f8cff')

  const darkBlue = getAccentPalette('blue', true)
  assert.equal(darkBlue.accent, '#6aa8ff')

  const fallback = getAccentPalette('non-existent' as AccentColor, true)
  assert.equal(fallback.accent, '#6aa8ff')
})

test('formatThemeModeLabel returns user-friendly label with OS hint', () => {
  assert.equal(formatThemeModeLabel('dark'), 'Dark')
  assert.equal(formatThemeModeLabel('light'), 'Light')
  assert.equal(formatThemeModeLabel('system'), 'System')
})

test('formatUiStyleLabel returns title and description', () => {
  const flatInfo = formatUiStyleLabel('flat')
  assert.equal(flatInfo.name, 'Flat (Default)')
  assert.ok(flatInfo.description.length > 0)
})

test('isTranslucentStyle identifies non-flat styles', () => {
  assert.equal(isTranslucentStyle('flat'), false)
  assert.equal(isTranslucentStyle('glass'), true)
  assert.equal(isTranslucentStyle('aero'), true)
  assert.equal(isTranslucentStyle('liquidglass'), true)
  assert.equal(isTranslucentStyle('ios'), true)
})

test('getUiStyleTokens returns distinct radii, sheens, and shadows for all 5 styles', () => {
  const styles: UiStyle[] = ['flat', 'glass', 'aero', 'liquidglass', 'ios']

  for (const style of styles) {
    const lightTokens = getUiStyleTokens(style, false)
    const darkTokens = getUiStyleTokens(style, true)
    assert.ok(lightTokens.radiusCard, `${style} light must have radiusCard`)
    assert.ok(darkTokens.radiusCard, `${style} dark must have radiusCard`)
    assert.ok(lightTokens.radiusControl, `${style} light must have radiusControl`)
  }

  // Verify key differentiators
  const flat = getUiStyleTokens('flat', false)
  const ios = getUiStyleTokens('ios', false)
  const aero = getUiStyleTokens('aero', false)
  const liquid = getUiStyleTokens('liquidglass', false)

  assert.equal(flat.radiusCard, '0.5rem')
  assert.equal(ios.radiusCard, '1rem')
  assert.equal(ios.radiusControl, '0.625rem')
  assert.ok(aero.cardSheen.includes('linear-gradient'))
  assert.ok(aero.cardShadow.includes('inset 0 1px'))
  assert.ok(liquid.cardShadow.includes('inset'))
  assert.ok(liquid.backdropBlur.includes('saturate'))
})
