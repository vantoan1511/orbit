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

test('Noir preset semantic tokens map border radii, backgrounds, and borders to centralized CSS variables', () => {
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
  assert.equal(formField.background, 'var(--ui-control-bg)')
  assert.equal(formField.borderRadius, 'var(--ui-radius-control)')
  assert.equal(formField.borderColor, 'var(--input-border, var(--border))')
  assert.equal(formField.hoverBorderColor, 'var(--border-strong)')
  assert.equal(formField.focusBorderColor, '{primary.color}')

  const content = semantic.content as Record<string, string>
  assert.ok(content, 'content must exist')
  assert.equal(content.background, 'var(--ui-card-bg)')
  assert.equal(content.hoverBackground, 'var(--bg-hover)')
  assert.equal(content.borderRadius, 'var(--ui-radius-card)')
  assert.equal(content.borderColor, 'var(--border)')

  const overlay = semantic.overlay as Record<string, Record<string, string>>
  assert.ok(overlay, 'overlay must exist')
  assert.equal(overlay.select.background, 'var(--ui-card-bg)')
  assert.equal(overlay.select.borderRadius, 'var(--ui-radius-control)')
  assert.equal(overlay.select.borderColor, 'var(--border)')
  assert.equal(overlay.popover.background, 'var(--ui-card-bg)')
  assert.equal(overlay.popover.borderRadius, 'var(--ui-radius-card)')
  assert.equal(overlay.popover.borderColor, 'var(--border)')
  assert.equal(overlay.modal.background, 'var(--ui-card-bg)')
  assert.equal(overlay.modal.borderRadius, 'var(--ui-radius-card)')
  assert.equal(overlay.modal.borderColor, 'var(--border)')

  const colorScheme = semantic.colorScheme as Record<string, Record<string, unknown>>
  for (const mode of ['light', 'dark']) {
    const scheme = colorScheme[mode] as Record<string, Record<string, unknown>>
    assert.ok(scheme, `colorScheme.${mode} should exist`)
    assert.equal(
      (scheme.formField as Record<string, string>)?.background,
      'var(--ui-control-bg)',
      `colorScheme.${mode}.formField.background must map to var(--ui-control-bg)`
    )
    assert.equal(
      (scheme.content as Record<string, string>)?.background,
      'var(--ui-card-bg)',
      `colorScheme.${mode}.content.background must map to var(--ui-card-bg)`
    )
    const schemeOverlay = scheme.overlay as Record<string, Record<string, string>>
    assert.equal(schemeOverlay?.select?.background, 'var(--ui-card-bg)')
    assert.equal(schemeOverlay?.popover?.background, 'var(--ui-card-bg)')
    assert.equal(schemeOverlay?.modal?.background, 'var(--ui-card-bg)')
  }
})

test('Noir preset components connect surfaces, radii, borders, glass blur, and sheen highlight to centralized CSS variables', () => {
  const components = Noir.components as Record<string, unknown>
  assert.ok(components, 'components must exist')

  const card = components.card as Record<string, Record<string, string>> | undefined
  assert.ok(card, 'card component config must exist')
  assert.equal(card.root.background, 'var(--ui-card-bg)')
  assert.equal(card.root.borderColor, 'var(--border)')
  assert.equal(card.root.borderRadius, 'var(--ui-radius-card)')
  assert.equal(card.root.backdropBlur, 'var(--ui-glass-blur)')
  assert.equal(card.root.sheenHighlight, 'var(--ui-sheen-highlight)')

  const datatable = components.datatable as Record<string, Record<string, string>>
  assert.ok(datatable, 'datatable component config must exist')
  assert.equal(datatable.root?.borderColor, 'var(--border)')
  assert.equal(datatable.headerCell?.borderColor, 'var(--border)')
  assert.equal(datatable.bodyCell?.borderColor, 'var(--border)')
  assert.equal(datatable.row?.background, 'var(--ui-card-bg)')
  assert.equal(datatable.row?.hoverBackground, 'var(--bg-hover)')

  const dialog = components.dialog as Record<string, Record<string, string>>
  assert.ok(dialog, 'dialog component config must exist')
  assert.equal(dialog.root.background, 'var(--ui-card-bg)')
  assert.equal(dialog.root.borderColor, 'var(--border)')
  assert.equal(dialog.root.borderRadius, 'var(--ui-radius-card)')
  assert.equal(dialog.root.backdropBlur, 'var(--ui-glass-blur)')
  assert.equal(dialog.root.sheenHighlight, 'var(--ui-sheen-highlight)')

  const drawer = components.drawer as Record<string, Record<string, string>>
  assert.ok(drawer, 'drawer component config must exist')
  assert.equal(drawer.root.background, 'var(--ui-card-bg)')
  assert.equal(drawer.root.borderColor, 'var(--border)')
  assert.equal(drawer.root.backdropBlur, 'var(--ui-glass-blur)')
  assert.equal(drawer.root.sheenHighlight, 'var(--ui-sheen-highlight)')

  const popover = components.popover as Record<string, Record<string, string>>
  assert.ok(popover, 'popover component config must exist')
  assert.equal(popover.root.background, 'var(--ui-card-bg)')
  assert.equal(popover.root.borderColor, 'var(--border)')
  assert.equal(popover.root.borderRadius, 'var(--ui-radius-card)')
  assert.equal(popover.root.backdropBlur, 'var(--ui-glass-blur)')
  assert.equal(popover.root.sheenHighlight, 'var(--ui-sheen-highlight)')

  const button = components.button as Record<string, Record<string, string>>
  assert.ok(button, 'button component config must exist')
  assert.equal(button.root.borderRadius, 'var(--ui-radius-control)')
  assert.equal(button.root.sheenHighlight, 'var(--ui-sheen-highlight)')

  const select = components.select as Record<string, Record<string, string>>
  assert.ok(select, 'select component config must exist')
  assert.equal(select.root.background, 'var(--ui-control-bg)')
  assert.equal(select.root.borderRadius, 'var(--ui-radius-control)')
  assert.equal(select.overlay.background, 'var(--ui-card-bg)')
  assert.equal(select.overlay.borderColor, 'var(--border)')
  assert.equal(select.overlay.borderRadius, 'var(--ui-radius-control)')
  assert.equal(select.overlay.backdropBlur, 'var(--ui-glass-blur)')

  const inputtext = components.inputtext as Record<string, Record<string, string>>
  assert.ok(inputtext, 'inputtext component config must exist')
  assert.equal(inputtext.root.background, 'var(--ui-control-bg)')
  assert.equal(inputtext.root.borderRadius, 'var(--ui-radius-control)')

  const textarea = components.textarea as Record<string, Record<string, string>>
  assert.ok(textarea, 'textarea component config must exist')
  assert.equal(textarea.root.background, 'var(--ui-control-bg)')
  assert.equal(textarea.root.borderRadius, 'var(--ui-radius-control)')

  const tag = components.tag as Record<string, Record<string, string>>
  assert.ok(tag, 'tag component config must exist')
  assert.equal(tag.root.borderRadius, 'var(--ui-radius-control)')

  const toast = components.toast as Record<string, Record<string, string>>
  assert.ok(toast, 'toast component config must exist')
  assert.equal(toast.root.borderRadius, 'var(--ui-radius-card)')
  assert.equal(toast.root.backdropBlur, 'var(--ui-glass-blur)')
  assert.equal(toast.root.sheenHighlight, 'var(--ui-sheen-highlight)')

  const tooltip = components.tooltip as Record<string, Record<string, string>>
  assert.ok(tooltip, 'tooltip component config must exist')
  assert.equal(tooltip.root.borderRadius, 'var(--ui-radius-control)')
})

test('Noir preset contextmenu and menu map root surfaces and item focus states to centralized tokens', () => {
  const components = Noir.components as Record<string, unknown>
  const contextmenu = components.contextmenu as Record<string, Record<string, string>>
  assert.ok(contextmenu, 'contextmenu config must exist')
  assert.equal(contextmenu.root.background, 'var(--ui-overlay-bg)')
  assert.equal(contextmenu.root.borderColor, 'var(--border)')
  assert.equal(contextmenu.root.borderRadius, 'var(--ui-radius-card)')
  assert.equal(contextmenu.root.backdropBlur, 'var(--ui-glass-blur)')
  assert.equal(contextmenu.root.sheenHighlight, 'var(--ui-sheen-highlight)')
  assert.equal(contextmenu.item.focusBackground, 'var(--bg-hover)')
  assert.equal(contextmenu.item.borderRadius, 'var(--ui-radius-control)')
  assert.equal(contextmenu.item.color, '{text.color}')
  assert.equal(contextmenu.item.focusColor, '{text.color}')

  const menu = components.menu as Record<string, Record<string, string>>
  assert.ok(menu, 'menu config must exist')
  assert.equal(menu.root.background, 'var(--ui-overlay-bg)')
  assert.equal(menu.root.borderColor, 'var(--border)')
  assert.equal(menu.root.borderRadius, 'var(--ui-radius-card)')
  assert.equal(menu.root.backdropBlur, 'var(--ui-glass-blur)')
  assert.equal(menu.root.sheenHighlight, 'var(--ui-sheen-highlight)')
  assert.equal(menu.item.focusBackground, 'var(--bg-hover)')
  assert.equal(menu.item.borderRadius, 'var(--ui-radius-control)')
  assert.equal(menu.item.color, '{text.color}')
  assert.equal(menu.item.focusColor, '{text.color}')
})
