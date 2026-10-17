import assert from 'node:assert/strict'
import test from 'node:test'
import { Noir } from '../orbitTheme.ts'

test('Noir preset primary maps to dynamic primary palette rather than hardcoded zinc', () => {
  const semantic = Noir.semantic as Record<string, unknown>
  assert.ok(semantic, 'semantic tokens must exist')

  const primary = semantic.primary as Record<string, string>
  assert.ok(primary, 'primary palette must exist')
  assert.equal(primary['500'], '{primary.500}')

  const colorScheme = semantic.colorScheme as Record<string, Record<string, Record<string, string>>>
  assert.ok(colorScheme, 'colorScheme must exist')

  // Light mode primary
  const lightPrimary = colorScheme.light.primary
  assert.equal(lightPrimary.color, '{primary.500}')
  assert.equal(lightPrimary.hoverColor, '{primary.600}')
  assert.equal(lightPrimary.activeColor, '{primary.700}')
  assert.equal(lightPrimary.contrastColor, '#ffffff')

  // Dark mode primary
  const darkPrimary = colorScheme.dark.primary
  assert.equal(darkPrimary.color, '{primary.400}')
  assert.equal(darkPrimary.hoverColor, '{primary.300}')
  assert.equal(darkPrimary.activeColor, '{primary.200}')

  // Focus ring
  const focusRing = semantic.focusRing as Record<string, string>
  assert.equal(focusRing.color, '{primary.color}')
})

test('Noir preset components connect surfaces and radii to custom properties', () => {
  const components = Noir.components as Record<string, unknown>
  assert.ok(components, 'components must exist')

  const card = components.card as Record<string, Record<string, string>> | undefined
  assert.ok(card, 'card component config must exist')
  assert.equal(card.root.background, 'var(--bg-card)')
  assert.equal(card.root.borderRadius, 'var(--ui-radius-card)')

  const dialog = components.dialog as Record<string, Record<string, string>>
  assert.ok(dialog, 'dialog component config must exist')
  assert.equal(dialog.root.borderRadius, 'var(--ui-radius-card)')
})
