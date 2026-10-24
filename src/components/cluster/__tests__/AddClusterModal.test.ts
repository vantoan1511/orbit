import assert from 'node:assert/strict'
import fs from 'node:fs'
import path from 'node:path'
import test from 'node:test'

test('AddClusterModal component file exists and defines data-component="add-cluster-modal"', () => {
  const filePath = path.resolve(process.cwd(), 'src/components/cluster/AddClusterModal.vue')
  assert.ok(fs.existsSync(filePath), 'AddClusterModal.vue must exist')

  const content = fs.readFileSync(filePath, 'utf-8')
  assert.ok(
    content.includes('data-component="add-cluster-modal"'),
    'AddClusterModal must define data-component="add-cluster-modal"'
  )
})

test('AddClusterModal file size is under 800 lines and contains zero raw button elements', () => {
  const filePath = path.resolve(process.cwd(), 'src/components/cluster/AddClusterModal.vue')
  const content = fs.readFileSync(filePath, 'utf-8')
  const lines = content.split('\n')

  assert.ok(
    lines.length < 800,
    `AddClusterModal.vue line count must be < 800 lines (current: ${lines.length})`
  )
  assert.ok(
    !content.includes('<button'),
    'AddClusterModal must not contain raw HTML <button> elements; use PrimeVue <Button>'
  )
})

test('AddClusterModal uses PrimeVue controls, composes ClusterCustomizationFields, and delegates to kubernetesService.addCluster', () => {
  const filePath = path.resolve(process.cwd(), 'src/components/cluster/AddClusterModal.vue')
  const content = fs.readFileSync(filePath, 'utf-8')

  assert.ok(content.includes('primevue'), 'AddClusterModal must import from primevue')
  assert.ok(content.includes('Dialog'), 'AddClusterModal must use PrimeVue Dialog')
  assert.ok(content.includes('Tabs'), 'AddClusterModal must use PrimeVue Tabs')
  assert.ok(content.includes('InputText'), 'AddClusterModal must use PrimeVue InputText')
  assert.ok(content.includes('Checkbox'), 'AddClusterModal must use PrimeVue Checkbox')
  assert.ok(content.includes('Button'), 'AddClusterModal must use PrimeVue Button')
  assert.ok(
    content.includes('ClusterCustomizationFields'),
    'AddClusterModal must compose ClusterCustomizationFields'
  )
  assert.ok(
    content.includes('kubernetesService.addCluster'),
    'AddClusterModal must call kubernetesService.addCluster'
  )
  assert.ok(content.includes('v-tooltip'), 'AddClusterModal must use PrimeVue v-tooltip directive')
  assert.ok(
    !content.includes('@tauri-apps/api'),
    'AddClusterModal must never import @tauri-apps/api directly'
  )
  assert.ok(
    !content.includes('@neutralinojs'),
    'AddClusterModal must never import native runtime library directly'
  )
})

test('AddClusterModal defines 3 input methods, collision feedback badge, and context selection', () => {
  const filePath = path.resolve(process.cwd(), 'src/components/cluster/AddClusterModal.vue')
  const content = fs.readFileSync(filePath, 'utf-8')

  // Tabs
  assert.ok(content.includes('file'), 'AddClusterModal must include file tab')
  assert.ok(content.includes('paste'), 'AddClusterModal must include paste YAML tab')
  assert.ok(content.includes('manual'), 'AddClusterModal must include manual endpoint tab')

  // Duplicate collision badge feedback
  assert.ok(
    content.includes('Already in Orbit'),
    'AddClusterModal must display "Already in Orbit" feedback for existing clusters'
  )

  // Context selection & auto-switch
  assert.ok(content.includes('switchContext'), 'AddClusterModal must support auto-switch context')
  assert.ok(content.includes('selectedContexts'), 'AddClusterModal must track selected contexts')
})

test('AddClusterModal enforces token hygiene, dynamic UI styling, and canonical underline tabs', () => {
  const filePath = path.resolve(process.cwd(), 'src/components/cluster/AddClusterModal.vue')
  const content = fs.readFileSync(filePath, 'utf-8')

  // Zero occurrences of text-primary utility class for generic labels/text (avoids electric blue)
  assert.ok(
    !/(?<![(-])text-primary\b/.test(content),
    'AddClusterModal must not apply text-primary utility class to generic labels or text'
  )

  // Dynamic UI style tokens
  assert.ok(
    content.includes('var(--ui-radius-card)'),
    'AddClusterModal must use dynamic var(--ui-radius-card)'
  )
  assert.ok(
    content.includes('var(--ui-radius-control)'),
    'AddClusterModal must use dynamic var(--ui-radius-control)'
  )

  // Token hygiene: No raw amber palette colors; use semantic warning tokens
  assert.ok(
    !content.includes('amber-500'),
    'AddClusterModal must not hardcode raw amber-500 palette colors'
  )
  assert.ok(
    content.includes('var(--warning)') || content.includes('--warning'),
    'AddClusterModal must use semantic warning tokens'
  )

  // Canonical underline tabs (border-b on TabList without container borders)
  assert.ok(
    content.includes('border-b'),
    'AddClusterModal TabList must use underline styling (border-b)'
  )
})
