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

test('AddClusterModal uses PrimeVue controls and delegates to kubernetesService.addCluster', () => {
  const filePath = path.resolve(process.cwd(), 'src/components/cluster/AddClusterModal.vue')
  const content = fs.readFileSync(filePath, 'utf-8')

  assert.ok(content.includes('primevue'), 'AddClusterModal must import from primevue')
  assert.ok(content.includes('Dialog'), 'AddClusterModal must use PrimeVue Dialog')
  assert.ok(content.includes('Tabs'), 'AddClusterModal must use PrimeVue Tabs')
  assert.ok(content.includes('InputText'), 'AddClusterModal must use PrimeVue InputText')
  assert.ok(content.includes('Checkbox'), 'AddClusterModal must use PrimeVue Checkbox')
  assert.ok(content.includes('Button'), 'AddClusterModal must use PrimeVue Button')
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

test('AddClusterModal defines 3 input methods, preview table, color presets, and tags', () => {
  const filePath = path.resolve(process.cwd(), 'src/components/cluster/AddClusterModal.vue')
  const content = fs.readFileSync(filePath, 'utf-8')

  // Tabs
  assert.ok(content.includes('file'), 'AddClusterModal must include file tab')
  assert.ok(content.includes('paste'), 'AddClusterModal must include paste YAML tab')
  assert.ok(content.includes('manual'), 'AddClusterModal must include manual endpoint tab')

  // Color presets & tags
  assert.ok(content.includes('#ef4444'), 'AddClusterModal must include red/prod preset swatch')
  assert.ok(content.includes('#10b981'), 'AddClusterModal must include emerald/dev preset swatch')
  assert.ok(content.includes('addTag'), 'AddClusterModal must support adding tags')
  assert.ok(content.includes('removeTag'), 'AddClusterModal must support removing tags')

  // Context selection & auto-switch
  assert.ok(content.includes('switchContext'), 'AddClusterModal must support auto-switch context')
  assert.ok(content.includes('selectedContexts'), 'AddClusterModal must track selected contexts')
})
