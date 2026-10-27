import assert from 'node:assert/strict'
import test from 'node:test'
import {
  CONTROL_TIERS,
  OVERLAY_SURFACE_TOKENS,
  OVERLAY_SURFACES,
  SIZED_CONTROLS,
  buildControlSizingCss,
  buildOverlaySurfaceCss
} from '../uiContract.ts'

test('(a) every OVERLAY_SURFACES row has >= 1 selector starting with .p-<component>', () => {
  assert.ok(OVERLAY_SURFACES.length >= 6, 'Must have at least 6 overlay surfaces')
  for (const spec of OVERLAY_SURFACES) {
    assert.ok(
      spec.selectors.length >= 1,
      `Spec for ${spec.component} must have at least 1 selector`
    )
    const hasComponentPrefix = spec.selectors.some((sel) => sel.startsWith(`.p-${spec.component}`))
    assert.ok(
      hasComponentPrefix,
      `Spec for ${spec.component} must have at least 1 selector starting with .p-${spec.component}, got: ${spec.selectors.join(', ')}`
    )
  }
})

test('(b) OVERLAY_SURFACE_TOKENS values match PRD §3.1 exactly', () => {
  assert.deepEqual(OVERLAY_SURFACE_TOKENS, {
    background: 'var(--ui-overlay-bg)',
    borderColor: 'var(--border)',
    borderRadius: 'var(--ui-radius-card)',
    shadow: 'var(--ui-card-shadow, var(--shadow))',
    backdropBlur: 'var(--ui-overlay-blur)',
    sheenHighlight: 'var(--ui-sheen-highlight)'
  })
})

test('(c) CONTROL_TIERS tokens match expected compact and standard definitions', () => {
  assert.equal(CONTROL_TIERS.compact.height, 'var(--ui-control-height-sm)')
  assert.equal(CONTROL_TIERS.compact.fontSize, 'var(--ui-control-font-size-sm)')
  assert.equal(CONTROL_TIERS.compact.paddingX, 'var(--ui-control-padding-x-sm)')

  assert.equal(CONTROL_TIERS.standard.height, 'var(--ui-control-height-md)')
  assert.equal(CONTROL_TIERS.standard.fontSize, 'var(--ui-control-font-size-md)')
  assert.equal(CONTROL_TIERS.standard.paddingX, 'var(--ui-control-padding-x-md)')
})

test('(d) buildOverlaySurfaceCss with stub dt emits backdrop-filter, -webkit-backdrop-filter, background-image for each selector', () => {
  const stubDt = (p: string) => `{${p}}`
  for (const spec of OVERLAY_SURFACES) {
    const css = buildOverlaySurfaceCss(spec, stubDt)
    for (const sel of spec.selectors) {
      assert.ok(css.includes(sel), `CSS must include selector ${sel}`)
    }
    assert.ok(css.includes('backdrop-filter:'), 'CSS must include backdrop-filter')
    assert.ok(css.includes('-webkit-backdrop-filter:'), 'CSS must include -webkit-backdrop-filter')
    assert.ok(css.includes('background-image:'), 'CSS must include background-image')
  }
})

test('(e) buildControlSizingCss emits height for fixed, min-height for min, none for textarea', () => {
  const buttonSpec = SIZED_CONTROLS.find((s) => s.component === 'button')!
  assert.ok(buttonSpec, 'button spec must exist')
  const buttonCss = buildControlSizingCss(buttonSpec)
  assert.ok(buttonCss.includes(buttonSpec.rootSelector), 'includes root selector')
  assert.ok(buttonCss.includes(buttonSpec.smSelector), 'includes sm selector')
  assert.ok(
    buttonCss.includes(`height: ${CONTROL_TIERS.standard.height}`),
    'fixed emits standard height'
  )
  assert.ok(
    buttonCss.includes(`height: ${CONTROL_TIERS.compact.height}`),
    'fixed emits compact height'
  )
  assert.ok(
    buttonCss.includes(`font-size: ${CONTROL_TIERS.standard.fontSize}`),
    'emits standard font-size'
  )

  const multiSpec = SIZED_CONTROLS.find((s) => s.component === 'multiselect')!
  assert.ok(multiSpec, 'multiselect spec must exist')
  const multiCss = buildControlSizingCss(multiSpec)
  assert.ok(
    multiCss.includes(`min-height: ${CONTROL_TIERS.standard.height}`),
    'min emits standard min-height'
  )
  assert.ok(
    multiCss.includes(`min-height: ${CONTROL_TIERS.compact.height}`),
    'min emits compact min-height'
  )
  assert.ok(
    multiCss.includes(`font-size: ${CONTROL_TIERS.standard.fontSize}`),
    'emits standard font-size'
  )

  const textareaSpec = SIZED_CONTROLS.find((s) => s.component === 'textarea')!
  assert.ok(textareaSpec, 'textarea spec must exist')
  const textareaCss = buildControlSizingCss(textareaSpec)
  assert.ok(!textareaCss.includes('height:'), 'textarea must not include height:')
  assert.ok(!textareaCss.includes('min-height:'), 'textarea must not include min-height:')
  assert.ok(
    textareaCss.includes(`font-size: ${CONTROL_TIERS.standard.fontSize}`),
    'textarea emits standard font-size'
  )
})
