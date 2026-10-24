import assert from 'node:assert/strict'
import fs from 'node:fs'
import path from 'node:path'
import test from 'node:test'
import { COLOR_PRESETS } from '../../../types/cluster.ts'

test('COLOR_PRESETS defines 8 standard color presets with valid hex values', () => {
  assert.equal(COLOR_PRESETS.length, 8)
  const hexes = COLOR_PRESETS.map((p) => p.hex)
  assert.ok(hexes.includes('#ef4444'))
  assert.ok(hexes.includes('#10b981'))
  assert.ok(hexes.includes('#3b82f6'))
  assert.ok(hexes.includes('#64748b'))
})

test('ClusterCustomizationFields component exists and defines data-component="cluster-customization-fields"', () => {
  const filePath = path.resolve(
    process.cwd(),
    'src/components/cluster/ClusterCustomizationFields.vue'
  )
  assert.ok(fs.existsSync(filePath), 'ClusterCustomizationFields.vue must exist')

  const content = fs.readFileSync(filePath, 'utf-8')
  assert.ok(
    content.includes('data-component="cluster-customization-fields"'),
    'ClusterCustomizationFields must define data-component="cluster-customization-fields"'
  )
})

test('ClusterCustomizationFields uses PrimeVue controls and zero raw button elements', () => {
  const filePath = path.resolve(
    process.cwd(),
    'src/components/cluster/ClusterCustomizationFields.vue'
  )
  assert.ok(fs.existsSync(filePath), 'ClusterCustomizationFields.vue must exist')
  const content = fs.readFileSync(filePath, 'utf-8')

  assert.ok(content.includes('Button'), 'ClusterCustomizationFields must use PrimeVue Button')
  assert.ok(content.includes('InputText'), 'ClusterCustomizationFields must use PrimeVue InputText')
  assert.ok(content.includes('v-tooltip'), 'ClusterCustomizationFields must use v-tooltip')
  assert.ok(
    !content.includes('<button'),
    'ClusterCustomizationFields must contain zero raw <button> elements'
  )
})

test('ClusterCustomizationFields enforces label styling, token hygiene, and dynamic control radius', () => {
  const filePath = path.resolve(
    process.cwd(),
    'src/components/cluster/ClusterCustomizationFields.vue'
  )
  const content = fs.readFileSync(filePath, 'utf-8')

  // Field labels and text must not use text-primary utility class (maps to bright primary color)
  assert.ok(
    !/(?<![(-])text-primary\b/.test(content),
    'ClusterCustomizationFields must not use text-primary utility class on field labels'
  )

  // Dynamic control radius
  assert.ok(
    content.includes('var(--ui-radius-control)'),
    'ClusterCustomizationFields must use dynamic var(--ui-radius-control)'
  )
})
