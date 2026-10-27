export type ControlTier = 'compact' | 'standard'

export interface ControlTierTokens {
  readonly height: string
  readonly fontSize: string
  readonly paddingX: string
}

export interface SizedControlSpec {
  readonly component: 'button' | 'inputtext' | 'select' | 'multiselect' | 'inputnumber' | 'textarea'
  readonly rootSelector: string
  readonly smSelector: string
  readonly heightMode: 'fixed' | 'min' | 'none'
}

export interface OverlaySurfaceSpec {
  readonly component: 'menu' | 'contextmenu' | 'tieredmenu' | 'select' | 'multiselect' | 'popover'
  readonly tokenGroup: 'root' | 'overlay'
  readonly selectors: readonly string[]
}

export const OVERLAY_SURFACE_TOKENS: Readonly<{
  background: string
  borderColor: string
  borderRadius: string
  shadow: string
  backdropBlur: string
  sheenHighlight: string
}> = Object.freeze({
  background: 'var(--ui-overlay-bg)',
  borderColor: 'var(--border)',
  borderRadius: 'var(--ui-radius-card)',
  shadow: 'var(--ui-card-shadow, var(--shadow))',
  backdropBlur: 'var(--ui-overlay-blur)',
  sheenHighlight: 'var(--ui-sheen-highlight)'
})

export const CONTROL_TIERS: Readonly<Record<ControlTier, ControlTierTokens>> = Object.freeze({
  compact: Object.freeze({
    height: 'var(--ui-control-height-sm)',
    fontSize: 'var(--ui-control-font-size-sm)',
    paddingX: 'var(--ui-control-padding-x-sm)'
  }),
  standard: Object.freeze({
    height: 'var(--ui-control-height-md)',
    fontSize: 'var(--ui-control-font-size-md)',
    paddingX: 'var(--ui-control-padding-x-md)'
  })
})

export const OVERLAY_SURFACES: readonly OverlaySurfaceSpec[] = Object.freeze([
  Object.freeze({
    component: 'menu',
    tokenGroup: 'root',
    selectors: Object.freeze(['.p-menu-overlay'])
  }),
  Object.freeze({
    component: 'contextmenu',
    tokenGroup: 'root',
    selectors: Object.freeze(['.p-contextmenu', '.p-contextmenu-submenu'])
  }),
  Object.freeze({
    component: 'tieredmenu',
    tokenGroup: 'root',
    selectors: Object.freeze(['.p-tieredmenu-overlay', '.p-tieredmenu-submenu'])
  }),
  Object.freeze({
    component: 'select',
    tokenGroup: 'overlay',
    selectors: Object.freeze(['.p-select-overlay'])
  }),
  Object.freeze({
    component: 'multiselect',
    tokenGroup: 'overlay',
    selectors: Object.freeze(['.p-multiselect-overlay'])
  }),
  Object.freeze({
    component: 'popover',
    tokenGroup: 'root',
    selectors: Object.freeze(['.p-popover'])
  })
])

export const SIZED_CONTROLS: readonly SizedControlSpec[] = Object.freeze([
  Object.freeze({
    component: 'button',
    rootSelector: '.p-button',
    smSelector: '.p-button-sm',
    heightMode: 'fixed'
  }),
  Object.freeze({
    component: 'inputtext',
    rootSelector: '.p-inputtext',
    smSelector: '.p-inputtext-sm',
    heightMode: 'fixed'
  }),
  Object.freeze({
    component: 'select',
    rootSelector: '.p-select',
    smSelector: '.p-select-sm',
    heightMode: 'fixed'
  }),
  Object.freeze({
    component: 'multiselect',
    rootSelector: '.p-multiselect',
    smSelector: '.p-multiselect-sm',
    heightMode: 'min'
  }),
  Object.freeze({
    component: 'inputnumber',
    rootSelector: '.p-inputnumber',
    smSelector: '.p-inputnumber-sm',
    heightMode: 'fixed'
  }),
  Object.freeze({
    component: 'textarea',
    rootSelector: '.p-textarea',
    smSelector: '.p-textarea-sm',
    heightMode: 'none'
  })
])

const HEIGHT_PROP_LOOKUP: Readonly<Record<'fixed' | 'min' | 'none', string | null>> = Object.freeze(
  {
    fixed: 'height',
    min: 'min-height',
    none: null
  }
)

export function buildOverlaySurfaceCss(
  spec: OverlaySurfaceSpec,
  dt: (path: string) => unknown
): string {
  const blur = String(dt(`${spec.component}.${spec.tokenGroup}.backdrop.blur`) ?? '')
  const sheen = String(dt(`${spec.component}.${spec.tokenGroup}.sheen.highlight`) ?? '')
  return spec.selectors
    .map(
      (selector) => `
${selector} {
  -webkit-backdrop-filter: ${blur};
  backdrop-filter: ${blur};
  background-image: ${sheen};
}`
    )
    .join('\n')
}

export function buildControlSizingCss(spec: SizedControlSpec): string {
  const heightProp = HEIGHT_PROP_LOOKUP[spec.heightMode]
  const standardHeightDecl = heightProp
    ? `\n  ${heightProp}: ${CONTROL_TIERS.standard.height};`
    : ''
  const compactHeightDecl = heightProp ? `\n  ${heightProp}: ${CONTROL_TIERS.compact.height};` : ''

  return `
${spec.rootSelector} {${standardHeightDecl}
  font-size: ${CONTROL_TIERS.standard.fontSize};
}

${spec.smSelector} {${compactHeightDecl}
}`
}
