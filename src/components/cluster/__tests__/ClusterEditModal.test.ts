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

test('ClusterEditModal uses PrimeVue controls, composes ClusterCustomizationFields, and delegates to updateClusterMetadata', () => {
  const filePath = path.resolve(process.cwd(), 'src/components/cluster/ClusterEditModal.vue')
  const content = fs.readFileSync(filePath, 'utf-8')

  assert.ok(content.includes('primevue'), 'ClusterEditModal must import from primevue')
  assert.ok(content.includes('Dialog'), 'ClusterEditModal must use PrimeVue Dialog')
  assert.ok(
    content.includes('ClusterCustomizationFields'),
    'ClusterEditModal must compose ClusterCustomizationFields'
  )
  assert.ok(
    content.includes('kubernetesService.updateClusterMetadata'),
    'ClusterEditModal must call kubernetesService.updateClusterMetadata'
  )
  assert.ok(
    !content.includes(['@neutralinojs', 'lib'].join('/')),
    'ClusterEditModal must never import native runtime library directly'
  )
})

test('ClusterEditModal delegates customization and supports metadata reset', () => {
  const filePath = path.resolve(process.cwd(), 'src/components/cluster/ClusterEditModal.vue')
  const content = fs.readFileSync(filePath, 'utf-8')

  assert.ok(
    content.includes('ClusterCustomizationFields'),
    'ClusterEditModal must use ClusterCustomizationFields for swatches and tags'
  )
  assert.ok(
    content.includes('handleReset'),
    'ClusterEditModal must support resetting metadata to default'
  )
})

test('ClusterEditModal inherits global dialog surface and contains zero hardcoded modal backgrounds', () => {
  const filePath = path.resolve(process.cwd(), 'src/components/cluster/ClusterEditModal.vue')
  const content = fs.readFileSync(filePath, 'utf-8')

  assert.ok(
    !content.includes('bg-(--bg-sidebar)'),
    'ClusterEditModal must not hardcode bg-(--bg-sidebar) for header/footer'
  )
})
