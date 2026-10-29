import assert from 'node:assert/strict'
import test from 'node:test'
import { Noir } from '../orbitTheme.ts'
import { CONTROL_TIERS, OVERLAY_SURFACES, OVERLAY_SURFACE_TOKENS } from '../uiContract.ts'

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
  assert.equal(overlay.select.background, 'var(--ui-overlay-bg)')
  assert.equal(overlay.select.borderRadius, 'var(--ui-radius-card)')
  assert.equal(overlay.select.borderColor, 'var(--border)')
  assert.equal(overlay.popover.background, 'var(--ui-overlay-bg)')
  assert.equal(overlay.popover.borderRadius, 'var(--ui-radius-card)')
  assert.equal(overlay.popover.borderColor, 'var(--border)')
  assert.equal(overlay.modal.background, 'var(--ui-dialog-bg)')
  assert.equal(overlay.modal.borderRadius, 'var(--ui-radius-card)')
  assert.equal(overlay.modal.borderColor, 'var(--border)')

  const colorScheme = semantic.colorScheme as Record<string, Record<string, unknown>>
  for (const mode of ['light', 'dark']) {
    const scheme = colorScheme[mode] as Record<string, Record<string, unknown>>
    assert.ok(scheme, `colorScheme.${mode} should exist`)
    assert.equal(
      (scheme.mask as Record<string, string>)?.background,
      'var(--ui-mask-bg)',
      `colorScheme.${mode}.mask.background must map to var(--ui-mask-bg)`
    )
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
    assert.equal(schemeOverlay?.select?.background, 'var(--ui-overlay-bg)')
    assert.equal(schemeOverlay?.popover?.background, 'var(--ui-overlay-bg)')
    assert.equal(schemeOverlay?.modal?.background, 'var(--ui-dialog-bg)')
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
  assert.equal(datatable.bodyCell?.borderColor, 'var(--border-subtle, var(--border))')
  assert.equal(datatable.row?.background, 'var(--ui-card-bg)')
  assert.equal(datatable.row?.hoverBackground, 'var(--bg-hover)')

  const dialog = components.dialog as Record<string, Record<string, string>>
  assert.ok(dialog, 'dialog component config must exist')
  assert.equal(dialog.root.background, 'var(--ui-dialog-bg)')
  assert.equal(dialog.root.borderColor, 'var(--border)')
  assert.equal(dialog.root.borderRadius, 'var(--ui-radius-card)')
  assert.equal(dialog.root.backdropBlur, 'var(--ui-dialog-blur)')
  assert.equal(dialog.root.sheenHighlight, 'var(--ui-sheen-highlight)')

  const drawer = components.drawer as Record<string, Record<string, string>>
  assert.ok(drawer, 'drawer component config must exist')
  assert.equal(drawer.root.background, 'var(--ui-card-bg)')
  assert.equal(drawer.root.borderColor, 'var(--border)')
  assert.equal(drawer.root.backdropBlur, 'var(--ui-glass-blur)')
  assert.equal(drawer.root.sheenHighlight, 'var(--ui-sheen-highlight)')

  const popover = components.popover as Record<string, Record<string, string>>
  assert.ok(popover, 'popover component config must exist')
  assert.equal(popover.root.background, 'var(--ui-overlay-bg)')
  assert.equal(popover.root.borderColor, 'var(--border)')
  assert.equal(popover.root.borderRadius, 'var(--ui-radius-card)')
  assert.equal(popover.root.backdropBlur, 'var(--ui-overlay-blur)')
  assert.equal(popover.root.sheenHighlight, 'var(--ui-sheen-highlight)')

  const button = components.button as Record<string, Record<string, string>>
  assert.ok(button, 'button component config must exist')
  assert.equal(button.root.borderRadius, 'var(--ui-radius-control)')
  assert.equal(button.root.sheenHighlight, 'var(--ui-sheen-highlight)')

  const select = components.select as Record<string, Record<string, string>>
  assert.ok(select, 'select component config must exist')
  assert.equal(select.root.background, 'var(--ui-control-bg)')
  assert.equal(select.root.borderRadius, 'var(--ui-radius-control)')
  assert.equal(select.overlay.background, 'var(--ui-overlay-bg)')
  assert.equal(select.overlay.borderColor, 'var(--border)')
  assert.equal(select.overlay.borderRadius, 'var(--ui-radius-card)')
  assert.equal(select.overlay.backdropBlur, 'var(--ui-overlay-blur)')
  assert.equal(select.overlay.sheenHighlight, 'var(--ui-sheen-highlight)')

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
  assert.equal(contextmenu.root.backdropBlur, 'var(--ui-overlay-blur)')
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
  assert.equal(menu.root.backdropBlur, 'var(--ui-overlay-blur)')
  assert.equal(menu.root.sheenHighlight, 'var(--ui-sheen-highlight)')
  assert.equal(menu.item.focusBackground, 'var(--bg-hover)')
  assert.equal(menu.item.borderRadius, 'var(--ui-radius-control)')
  assert.equal(menu.item.color, '{text.color}')
  assert.equal(menu.item.focusColor, '{text.color}')
})

test('all OVERLAY_SURFACES map to canonical overlay tokens', () => {
  const components = Noir.components as Record<string, Record<string, Record<string, string>>>
  assert.ok(components, 'components must exist')

  for (const spec of OVERLAY_SURFACES) {
    const compConfig = components[spec.component]
    assert.ok(compConfig, `Component ${spec.component} must be configured in Noir preset`)
    const group = compConfig[spec.tokenGroup]
    assert.ok(group, `Token group ${spec.tokenGroup} must exist on component ${spec.component}`)

    assert.equal(
      group.background,
      OVERLAY_SURFACE_TOKENS.background,
      `${spec.component}.${spec.tokenGroup}.background must match canonical token`
    )
    assert.equal(
      group.borderColor,
      OVERLAY_SURFACE_TOKENS.borderColor,
      `${spec.component}.${spec.tokenGroup}.borderColor must match canonical token`
    )
    assert.equal(
      group.borderRadius,
      OVERLAY_SURFACE_TOKENS.borderRadius,
      `${spec.component}.${spec.tokenGroup}.borderRadius must match canonical token`
    )
    assert.equal(
      group.shadow,
      OVERLAY_SURFACE_TOKENS.shadow,
      `${spec.component}.${spec.tokenGroup}.shadow must match canonical token`
    )
    assert.equal(
      group.backdropBlur,
      OVERLAY_SURFACE_TOKENS.backdropBlur,
      `${spec.component}.${spec.tokenGroup}.backdropBlur must match canonical token`
    )
    assert.equal(
      group.sheenHighlight,
      OVERLAY_SURFACE_TOKENS.sheenHighlight,
      `${spec.component}.${spec.tokenGroup}.sheenHighlight must match canonical token`
    )
  }
})

test('tieredmenu and multiselect overlay are configured with shadow, blur, and sheen', () => {
  const components = Noir.components as Record<string, Record<string, Record<string, string>>>
  assert.ok(components, 'components must exist')

  const tieredmenu = components.tieredmenu
  assert.ok(tieredmenu, 'tieredmenu must be configured in Noir preset')
  assert.equal(tieredmenu.root.shadow, OVERLAY_SURFACE_TOKENS.shadow)
  assert.equal(tieredmenu.root.backdropBlur, OVERLAY_SURFACE_TOKENS.backdropBlur)
  assert.equal(tieredmenu.root.sheenHighlight, OVERLAY_SURFACE_TOKENS.sheenHighlight)

  const multiselect = components.multiselect
  assert.ok(multiselect, 'multiselect must be configured in Noir preset')
  assert.equal(multiselect.overlay.shadow, OVERLAY_SURFACE_TOKENS.shadow)
  assert.equal(multiselect.overlay.backdropBlur, OVERLAY_SURFACE_TOKENS.backdropBlur)
  assert.equal(multiselect.overlay.sheenHighlight, OVERLAY_SURFACE_TOKENS.sheenHighlight)
})

test('overlay components expose css hooks that consume backdrop and sheen tokens', () => {
  const components = Noir.components as Record<string, Record<string, unknown>>
  assert.ok(components, 'components must exist')

  const stubDt = (p: string) => `var(--p-${p.replace(/\./g, '-')})`

  for (const spec of OVERLAY_SURFACES) {
    const compConfig = components[spec.component]
    assert.ok(compConfig, `Component ${spec.component} must be configured in Noir preset`)
    const cssFn = compConfig.css as
      (({ dt }: { dt: (path: string) => string }) => string) | undefined
    assert.ok(
      typeof cssFn === 'function',
      `Component ${spec.component} must expose a css hook function`
    )

    const cssOutput = cssFn({ dt: stubDt })
    assert.ok(
      cssOutput.includes(
        `backdrop-filter: var(--p-${spec.component}-${spec.tokenGroup}-backdrop-blur)`
      ),
      `${spec.component} css hook must contain backdrop-filter with component token`
    )
    assert.ok(
      cssOutput.includes(
        `-webkit-backdrop-filter: var(--p-${spec.component}-${spec.tokenGroup}-backdrop-blur)`
      ),
      `${spec.component} css hook must contain -webkit-backdrop-filter with component token`
    )
    assert.ok(
      cssOutput.includes(
        `background-image: var(--p-${spec.component}-${spec.tokenGroup}-sheen-highlight)`
      ),
      `${spec.component} css hook must contain background-image with component token`
    )
    for (const sel of spec.selectors) {
      assert.ok(cssOutput.includes(sel), `${spec.component} css hook must target selector ${sel}`)
    }
  }
})

test('sizing parity: sized controls define consistent standard and compact tokens in Noir preset', () => {
  const components = Noir.components as Record<string, Record<string, Record<string, unknown>>>
  assert.ok(components, 'components must exist')

  const sizedComponents = ['button', 'inputtext', 'select', 'multiselect', 'textarea'] as const

  for (const name of sizedComponents) {
    const root = components[name]?.root as Record<string, unknown>
    assert.ok(root, `Component ${name}.root must exist in preset`)

    assert.equal(
      root.paddingX,
      CONTROL_TIERS.standard.paddingX,
      `${name}.root.paddingX must match Standard tier paddingX token`
    )

    const sm = root.sm as Record<string, unknown>
    assert.ok(sm, `${name}.root.sm must exist in preset`)
    assert.equal(
      sm.fontSize,
      CONTROL_TIERS.compact.fontSize,
      `${name}.root.sm.fontSize must match Compact tier fontSize token`
    )
    assert.equal(
      sm.paddingX,
      CONTROL_TIERS.compact.paddingX,
      `${name}.root.sm.paddingX must match Compact tier paddingX token`
    )
  }
})

test('icon-only button width equals tier height', () => {
  const components = Noir.components as Record<string, Record<string, Record<string, unknown>>>
  const buttonRoot = components.button?.root as Record<string, unknown>
  assert.ok(buttonRoot, 'button.root must exist in preset')

  assert.equal(
    buttonRoot.iconOnlyWidth,
    CONTROL_TIERS.standard.height,
    'button.root.iconOnlyWidth must equal Standard tier height'
  )

  const sm = buttonRoot.sm as Record<string, unknown>
  assert.ok(sm, 'button.root.sm must exist')
  assert.equal(
    sm.iconOnlyWidth,
    CONTROL_TIERS.compact.height,
    'button.root.sm.iconOnlyWidth must equal Compact tier height'
  )
})

test('css hook output contains height: var(--ui-control-height-md) for .p-inputtext and min-height for .p-multiselect', () => {
  const components = Noir.components as Record<string, Record<string, unknown>>
  const stubDt = (p: string) => `var(--p-${p.replace(/\./g, '-')})`

  const inputtext = components.inputtext
  assert.ok(inputtext, 'inputtext must exist')
  const inputCssFn = inputtext.css as
    (({ dt }: { dt: (path: string) => string }) => string) | undefined
  assert.ok(typeof inputCssFn === 'function', 'inputtext must have a css hook')
  const inputCss = inputCssFn({ dt: stubDt })
  assert.ok(
    inputCss.includes(`height: ${CONTROL_TIERS.standard.height}`),
    'inputtext css must contain standard height'
  )
  assert.ok(
    inputCss.includes(`height: ${CONTROL_TIERS.compact.height}`),
    'inputtext css must contain compact height'
  )

  const multiselect = components.multiselect
  assert.ok(multiselect, 'multiselect must exist')
  const multiCssFn = multiselect.css as
    (({ dt }: { dt: (path: string) => string }) => string) | undefined
  assert.ok(typeof multiCssFn === 'function', 'multiselect must have a css hook')
  const multiCss = multiCssFn({ dt: stubDt })
  assert.ok(
    multiCss.includes(`min-height: ${CONTROL_TIERS.standard.height}`),
    'multiselect css must contain standard min-height'
  )
  assert.ok(
    multiCss.includes(`min-height: ${CONTROL_TIERS.compact.height}`),
    'multiselect css must contain compact min-height'
  )
})

interface ToastSeverityTokens {
  background: string
  borderColor: string
  color: string
  detailColor: string
  shadow: string
  closeButton: {
    hoverBackground: string
    focusRing: {
      color: string
      shadow: string
    }
  }
}

interface ToastPresetTokens {
  root: Record<string, string>
  icon: Record<string, string>
  content: Record<string, string>
  text: Record<string, string>
  summary: Record<string, string>
  detail: Record<string, string>
  closeButton: {
    width: string
    height: string
    borderRadius: string
    focusRing: Record<string, string>
  }
  closeIcon: Record<string, string>
  colorScheme: Record<string, Record<string, ToastSeverityTokens>>
  css?: ({ dt }: { dt: (path: string) => string }) => string
}

test('Noir preset maps toast root, typography, padding, and dual-theme color schemes to centralized tokens', () => {
  const components = Noir.components as Record<string, unknown>
  const toast = components.toast as unknown as ToastPresetTokens
  assert.ok(toast, 'toast component config must exist')

  // Root geometry & backdrop
  assert.equal(toast.root.width, '24rem')
  assert.equal(toast.root.borderRadius, 'var(--ui-radius-card)')
  assert.equal(toast.root.borderWidth, '1px 1px 1px 3px')
  assert.equal(toast.root.transitionDuration, '{transition.duration}')
  assert.equal(toast.root.backdropBlur, 'var(--ui-glass-blur)')
  assert.equal(toast.root.sheenHighlight, 'var(--ui-sheen-highlight)')

  // Icon sizing
  assert.equal(toast.icon.size, '1rem')

  // Content padding & layout
  assert.equal(toast.content.padding, '0.75rem 0.875rem')
  assert.equal(toast.content.gap, '0.625rem')
  assert.equal(toast.text.gap, '0.25rem')

  // Typography
  assert.equal(toast.summary.fontWeight, '600')
  assert.equal(toast.summary.fontSize, 'var(--ui-font-size-base)')
  assert.equal(toast.detail.fontWeight, '400')
  assert.equal(toast.detail.fontSize, 'var(--ui-font-size-xs)')

  // Close button
  assert.equal(toast.closeButton.width, '1.5rem')
  assert.equal(toast.closeButton.height, '1.5rem')
  assert.equal(toast.closeButton.borderRadius, 'var(--ui-radius-control)')
  assert.equal(toast.closeButton.focusRing.width, '1px')
  assert.equal(toast.closeButton.focusRing.style, 'solid')
  assert.equal(toast.closeButton.focusRing.offset, '1px')
  assert.equal(toast.closeIcon.size, '0.875rem')

  // Dual-theme color schemes
  const colorScheme = toast.colorScheme
  assert.ok(colorScheme, 'toast colorScheme must exist')

  for (const mode of ['light', 'dark']) {
    const scheme = colorScheme[mode]
    assert.ok(scheme, `toast colorScheme.${mode} must exist`)

    // Info
    assert.equal(
      scheme.info.background,
      'linear-gradient(var(--info-soft), var(--info-soft)), var(--ui-overlay-bg)'
    )
    assert.equal(scheme.info.borderColor, 'var(--border) var(--border) var(--border) var(--info)')
    assert.equal(scheme.info.color, 'var(--info)')
    assert.equal(scheme.info.detailColor, 'var(--text-secondary)')
    assert.equal(scheme.info.shadow, 'var(--ui-card-shadow, var(--shadow))')
    assert.equal(scheme.info.closeButton.hoverBackground, 'var(--bg-hover)')
    assert.equal(scheme.info.closeButton.focusRing.color, 'var(--info)')
    assert.equal(scheme.info.closeButton.focusRing.shadow, 'none')

    // Success
    assert.equal(
      scheme.success.background,
      'linear-gradient(var(--success-soft), var(--success-soft)), var(--ui-overlay-bg)'
    )
    assert.equal(
      scheme.success.borderColor,
      'var(--border) var(--border) var(--border) var(--success)'
    )
    assert.equal(scheme.success.color, 'var(--success)')
    assert.equal(scheme.success.detailColor, 'var(--text-secondary)')
    assert.equal(scheme.success.shadow, 'var(--ui-card-shadow, var(--shadow))')
    assert.equal(scheme.success.closeButton.hoverBackground, 'var(--bg-hover)')
    assert.equal(scheme.success.closeButton.focusRing.color, 'var(--success)')
    assert.equal(scheme.success.closeButton.focusRing.shadow, 'none')

    // Warn
    assert.equal(
      scheme.warn.background,
      'linear-gradient(var(--warning-soft), var(--warning-soft)), var(--ui-overlay-bg)'
    )
    assert.equal(
      scheme.warn.borderColor,
      'var(--border) var(--border) var(--border) var(--warning)'
    )
    assert.equal(scheme.warn.color, 'var(--warning)')
    assert.equal(scheme.warn.detailColor, 'var(--text-secondary)')
    assert.equal(scheme.warn.shadow, 'var(--ui-card-shadow, var(--shadow))')
    assert.equal(scheme.warn.closeButton.hoverBackground, 'var(--bg-hover)')
    assert.equal(scheme.warn.closeButton.focusRing.color, 'var(--warning)')
    assert.equal(scheme.warn.closeButton.focusRing.shadow, 'none')

    // Error
    assert.equal(
      scheme.error.background,
      'linear-gradient(var(--danger-soft), var(--danger-soft)), var(--ui-overlay-bg)'
    )
    assert.equal(
      scheme.error.borderColor,
      'var(--border) var(--border) var(--border) var(--danger)'
    )
    assert.equal(scheme.error.color, 'var(--danger)')
    assert.equal(scheme.error.detailColor, 'var(--text-secondary)')
    assert.equal(scheme.error.shadow, 'var(--ui-card-shadow, var(--shadow))')
    assert.equal(scheme.error.closeButton.hoverBackground, 'var(--bg-hover)')
    assert.equal(scheme.error.closeButton.focusRing.color, 'var(--danger)')
    assert.equal(scheme.error.closeButton.focusRing.shadow, 'none')

    // Secondary
    assert.equal(scheme.secondary.background, 'var(--ui-overlay-bg)')
    assert.equal(scheme.secondary.borderColor, 'var(--border)')
    assert.equal(scheme.secondary.color, 'var(--text-primary)')
    assert.equal(scheme.secondary.detailColor, 'var(--text-secondary)')
    assert.equal(scheme.secondary.shadow, 'var(--ui-card-shadow, var(--shadow))')
    assert.equal(scheme.secondary.closeButton.hoverBackground, 'var(--bg-hover)')
    assert.equal(scheme.secondary.closeButton.focusRing.color, 'var(--border-strong)')
    assert.equal(scheme.secondary.closeButton.focusRing.shadow, 'none')

    // Contrast
    assert.equal(scheme.contrast.background, 'var(--ui-overlay-bg)')
    assert.equal(scheme.contrast.borderColor, 'var(--border-strong)')
    assert.equal(scheme.contrast.color, 'var(--text-primary)')
    assert.equal(scheme.contrast.detailColor, 'var(--text-secondary)')
    assert.equal(scheme.contrast.shadow, 'var(--ui-card-shadow, var(--shadow))')
    assert.equal(scheme.contrast.closeButton.hoverBackground, 'var(--bg-hover)')
    assert.equal(scheme.contrast.closeButton.focusRing.color, 'var(--border-strong)')
    assert.equal(scheme.contrast.closeButton.focusRing.shadow, 'none')
  }

  // Toast CSS hook for dynamic UI style backdrop-filter
  assert.equal(typeof toast.css, 'function', 'toast must configure a css hook for backdrop-filter')
  const stubDt = (p: string) => `var(--p-${p.replace(/\./g, '-')})`
  const cssOutput = toast.css!({ dt: stubDt })
  assert.ok(
    cssOutput.includes('backdrop-filter: var(--p-toast-root-backdrop-blur)'),
    'toast css hook must contain backdrop-filter with toast backdrop blur token'
  )
  assert.ok(
    cssOutput.includes('-webkit-backdrop-filter: var(--p-toast-root-backdrop-blur)'),
    'toast css hook must contain -webkit-backdrop-filter with toast backdrop blur token'
  )
  assert.ok(cssOutput.includes('.p-toast-message'), 'toast css hook must target .p-toast-message')
})

interface TagPresetTokens {
  root: Record<string, string>
  icon?: Record<string, string>
  colorScheme: Record<string, Record<string, { background: string; color: string }>>
  css?: ({ dt }: { dt: (path: string) => string }) => string
}

interface BadgePresetTokens {
  root: Record<string, string>
  dot?: Record<string, string>
  sm?: Record<string, string>
  lg?: Record<string, string>
  xl?: Record<string, string>
  colorScheme: Record<string, Record<string, { background: string; color: string }>>
  css?: ({ dt }: { dt: (path: string) => string }) => string
}

test('Noir preset configures Tag component with soft design tokens and adaptive styling', () => {
  const components = Noir.components as Record<string, unknown>
  const tag = components.tag as unknown as TagPresetTokens
  assert.ok(tag, 'tag component config must exist')

  assert.equal(tag.root.borderRadius, 'var(--ui-radius-control)')
  assert.equal(tag.root.roundedBorderRadius, '9999px')

  const colorScheme = tag.colorScheme
  assert.ok(colorScheme, 'tag colorScheme must exist')

  assert.equal(colorScheme.light.success.background, 'var(--success-soft)')
  assert.equal(colorScheme.light.success.color, 'var(--success)')
  assert.equal(colorScheme.dark.danger.background, 'var(--danger-soft)')
  assert.equal(colorScheme.dark.danger.color, 'var(--danger)')
  assert.equal(colorScheme.light.secondary.background, 'var(--bg-hover)')
  assert.equal(colorScheme.light.secondary.color, 'var(--text-secondary)')

  assert.equal(typeof tag.css, 'function', 'tag must configure a css hook')
  const stubDt = (p: string) => `var(--p-${p.replace(/\./g, '-')})`
  const cssOutput = tag.css!({ dt: stubDt })
  assert.ok(
    cssOutput.includes('border: 1px solid color-mix(in srgb, currentColor 20%, transparent)'),
    'tag css hook must contain 1px color-mix border'
  )
  assert.ok(
    cssOutput.includes('backdrop-filter: var(--p-tag-root-backdrop-blur)'),
    'tag css hook must contain backdrop-filter'
  )
})

test('Noir preset configures Badge component with soft design tokens and adaptive styling', () => {
  const components = Noir.components as Record<string, unknown>
  const badge = components.badge as unknown as BadgePresetTokens
  assert.ok(badge, 'badge component config must exist')

  assert.equal(badge.root.borderRadius, 'var(--ui-radius-control)')

  const colorScheme = badge.colorScheme
  assert.ok(colorScheme, 'badge colorScheme must exist')

  assert.equal(colorScheme.light.info.background, 'var(--info-soft)')
  assert.equal(colorScheme.light.info.color, 'var(--info)')
  assert.equal(colorScheme.dark.warn.background, 'var(--warning-soft)')
  assert.equal(colorScheme.dark.warn.color, 'var(--warning)')

  assert.equal(typeof badge.css, 'function', 'badge must configure a css hook')
  const stubDt = (p: string) => `var(--p-${p.replace(/\./g, '-')})`
  const cssOutput = badge.css!({ dt: stubDt })
  assert.ok(cssOutput.includes('.p-badge'), 'badge css hook must target .p-badge')
  assert.ok(
    cssOutput.includes('border: 1px solid color-mix(in srgb, currentColor 20%, transparent)'),
    'badge css hook must contain 1px color-mix border'
  )
  assert.ok(
    cssOutput.includes('backdrop-filter: var(--p-badge-root-backdrop-blur)'),
    'badge css hook must contain backdrop-filter'
  )
  assert.ok(cssOutput.includes('.p-badge-dot'), 'badge css hook must target .p-badge-dot')
})

test('Noir preset configures Tooltip component with compact typography and adaptive styling', () => {
  const components = Noir.components as Record<string, unknown>
  const tooltip = components.tooltip as Record<string, unknown>
  assert.ok(tooltip, 'tooltip component config must exist')

  const root = tooltip.root as Record<string, string>
  assert.ok(root, 'tooltip.root must exist')
  assert.equal(root.fontSize, 'var(--ui-font-size-xs)')
  assert.equal(root.lineHeight, 'var(--ui-line-height-xs)')
  assert.equal(root.fontWeight, '500')
  assert.equal(root.background, 'var(--ui-overlay-bg)')
  assert.equal(root.color, 'var(--text-primary)')
  assert.equal(root.borderColor, 'var(--border)')
  assert.equal(root.backdropBlur, 'var(--ui-overlay-blur)')
  assert.equal(root.sheenHighlight, 'var(--ui-sheen-highlight)')

  const colorScheme = tooltip.colorScheme as Record<string, Record<string, Record<string, string>>>
  assert.ok(colorScheme, 'tooltip colorScheme must exist')
  assert.equal(colorScheme.light.root.background, 'var(--ui-overlay-bg)')
  assert.equal(colorScheme.light.root.color, 'var(--text-primary)')
  assert.equal(colorScheme.dark.root.background, 'var(--ui-overlay-bg)')
  assert.equal(colorScheme.dark.root.color, 'var(--text-primary)')

  assert.equal(typeof tooltip.css, 'function', 'tooltip must configure a css hook')
  const stubDt = (p: string) => `var(--p-${p.replace(/\./g, '-')})`
  const cssOutput = (tooltip.css as (opt: { dt: (p: string) => string }) => string)({ dt: stubDt })
  assert.ok(cssOutput.includes('.p-tooltip-text'), 'tooltip css must target .p-tooltip-text')
  assert.ok(
    cssOutput.includes('font-size: var(--p-tooltip-root-font-size)'),
    'tooltip css must configure font-size via dt'
  )
  assert.ok(
    cssOutput.includes('line-height: var(--p-tooltip-root-line-height)'),
    'tooltip css must configure line-height via dt'
  )
  assert.ok(
    cssOutput.includes('border: 1px solid var(--border)'),
    'tooltip css must configure 1px border'
  )
  assert.ok(
    cssOutput.includes('background-color: var(--ui-overlay-bg)'),
    'tooltip css must configure background-color via var(--ui-overlay-bg)'
  )
  assert.ok(
    cssOutput.includes('background-image: var(--p-tooltip-root-sheen-highlight)'),
    'tooltip css must configure sheen highlight background-image'
  )
  assert.ok(
    cssOutput.includes('backdrop-filter: var(--p-tooltip-root-backdrop-blur)'),
    'tooltip css must configure backdrop-filter'
  )
  assert.ok(
    cssOutput.includes('.p-tooltip-arrow') && cssOutput.includes('display: none'),
    'tooltip css must hide border triangle arrow for clean glass blur aesthetic'
  )
})

test('Noir preset components harmonize typography with centralized design tokens', () => {
  const components = Noir.components as Record<string, Record<string, unknown>>

  const tag = components.tag as Record<string, Record<string, string>>
  assert.equal(tag.root.fontSize, 'var(--ui-font-size-xs)')
  assert.equal(tag.icon.size, 'var(--ui-font-size-xs)')

  const badge = components.badge as Record<string, Record<string, string>>
  assert.equal(badge.root.fontSize, 'var(--ui-font-size-2xs)')
  assert.equal(badge.sm.fontSize, 'var(--ui-font-size-3xs)')
  assert.equal(badge.lg.fontSize, 'var(--ui-font-size-base)')

  const card = components.card as Record<string, Record<string, string>>
  assert.equal(card.title.fontSize, 'var(--ui-font-size-base)')

  const dialog = components.dialog as Record<string, Record<string, string>>
  assert.ok(dialog.title, 'dialog.title must exist')
  assert.equal(dialog.title.fontSize, 'var(--ui-font-size-base)')
  assert.equal(dialog.title.fontWeight, '600')

  const toast = components.toast as Record<string, Record<string, string>>
  assert.equal(toast.summary.fontSize, 'var(--ui-font-size-base)')
  assert.equal(toast.detail.fontSize, 'var(--ui-font-size-xs)')
})
