import assert from 'node:assert/strict'
import fs from 'node:fs'
import path from 'node:path'
import test from 'node:test'

test('ClusterEditModal component file exists and defines data-component="cluster-edit-modal"', () => {
  const filePath = path.resolve(process.cwd(), 'src/components/cluster/ClusterEditModal.vue')
  assert.ok(fs.existsSync(filePath), 'ClusterEditModal.vue must exist')

  const content = fs.readFileSync(filePath, 'utf-8')
  assert.ok(
    content.includes('data-component="cluster-edit-modal"'),
    'ClusterEditModal must define data-component="cluster-edit-modal"'
  )
})

test('ClusterEditModal uses PrimeVue controls and delegates to kubernetesService.updateClusterMetadata', () => {
  const filePath = path.resolve(process.cwd(), 'src/components/cluster/ClusterEditModal.vue')
  const content = fs.readFileSync(filePath, 'utf-8')

  assert.ok(content.includes('primevue'), 'ClusterEditModal must import from primevue')
  assert.ok(content.includes('Dialog'), 'ClusterEditModal must use PrimeVue Dialog')
  assert.ok(
    content.includes('InputText'),
    'ClusterEditModal must use PrimeVue InputText for alias and tags'
  )
  assert.ok(
    content.includes('Textarea'),
    'ClusterEditModal must use PrimeVue Textarea for description'
  )
  assert.ok(
    content.includes('kubernetesService.updateClusterMetadata'),
    'ClusterEditModal must call kubernetesService.updateClusterMetadata'
  )
  assert.ok(content.includes('v-tooltip'), 'ClusterEditModal must use PrimeVue v-tooltip directive')
  assert.ok(
    !content.includes(['@neutralinojs', 'lib'].join('/')),
    'ClusterEditModal must never import native runtime library directly'
  )
})

test('ClusterEditModal defines environment color swatches and tag management', () => {
  const filePath = path.resolve(process.cwd(), 'src/components/cluster/ClusterEditModal.vue')
  const content = fs.readFileSync(filePath, 'utf-8')

  assert.ok(content.includes('#ef4444'), 'ClusterEditModal must include red/prod preset swatch')
  assert.ok(content.includes('#10b981'), 'ClusterEditModal must include emerald/dev preset swatch')
  assert.ok(content.includes('addTag'), 'ClusterEditModal must support adding tags')
  assert.ok(content.includes('removeTag'), 'ClusterEditModal must support removing tags')
  assert.ok(
    content.includes('handleReset'),
    'ClusterEditModal must support resetting metadata to default'
  )
})
