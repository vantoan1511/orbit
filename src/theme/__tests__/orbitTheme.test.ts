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

test('Noir preset semantic tokens map border radii to custom properties', () => {
  const semantic = Noir.semantic as Record<string, unknown>
  assert.ok(semantic, 'semantic tokens must exist')

  const borderRadius = semantic.borderRadius as Record<string, string>
  assert.ok(borderRadius, 'borderRadius must exist')
  assert.equal(borderRadius.none, '0')
  assert.equal(borderRadius.xs, 'var(--ui-radius-control)')
  assert.equal(borderRadius.sm, 'var(--ui-radius-control)')
  assert.equal(borderRadius.md, 'var(--ui-radius-control)')
  assert.equal(borderRadius.lg, 'var(--ui-radius-card)')
  assert.equal(borderRadius.xl, 'var(--ui-radius-card)')

  const formField = semantic.formField as Record<string, string>
  assert.ok(formField, 'formField must exist')
  assert.equal(formField.borderRadius, 'var(--ui-radius-control)')

  const content = semantic.content as Record<string, string>
  assert.ok(content, 'content must exist')
  assert.equal(content.borderRadius, 'var(--ui-radius-card)')

  const overlay = semantic.overlay as Record<string, Record<string, string>>
  assert.ok(overlay, 'overlay must exist')
  assert.equal(overlay.select.borderRadius, 'var(--ui-radius-control)')
  assert.equal(overlay.popover.borderRadius, 'var(--ui-radius-card)')
  assert.equal(overlay.modal.borderRadius, 'var(--ui-radius-card)')
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

  const popover = components.popover as Record<string, Record<string, string>>
  assert.ok(popover, 'popover component config must exist')
  assert.equal(popover.root.borderRadius, 'var(--ui-radius-card)')

  const button = components.button as Record<string, Record<string, string>>
  assert.ok(button, 'button component config must exist')
  assert.equal(button.root.borderRadius, 'var(--ui-radius-control)')

  const select = components.select as Record<string, Record<string, string>>
  assert.ok(select, 'select component config must exist')
  assert.equal(select.root.borderRadius, 'var(--ui-radius-control)')

  const inputtext = components.inputtext as Record<string, Record<string, string>>
  assert.ok(inputtext, 'inputtext component config must exist')
  assert.equal(inputtext.root.borderRadius, 'var(--ui-radius-control)')

  const textarea = components.textarea as Record<string, Record<string, string>>
  assert.ok(textarea, 'textarea component config must exist')
  assert.equal(textarea.root.borderRadius, 'var(--ui-radius-control)')

  const tag = components.tag as Record<string, Record<string, string>>
  assert.ok(tag, 'tag component config must exist')
  assert.equal(tag.root.borderRadius, 'var(--ui-radius-control)')
})
