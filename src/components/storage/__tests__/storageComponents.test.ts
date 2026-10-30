import assert from 'node:assert/strict'
import test from 'node:test'
import fs from 'node:fs'
import path from 'node:path'

const STORAGE_METRICS_CARDS_PATH = path.resolve('src/components/storage/StorageMetricsCards.vue')
const STORAGE_CLASSES_SIDEBAR_PATH = path.resolve(
  'src/components/storage/StorageClassesSidebar.vue'
)

test('StorageMetricsCards.vue and StorageClassesSidebar.vue use shared storageQuantity parser', () => {
  const metricsCards = fs.readFileSync(STORAGE_METRICS_CARDS_PATH, 'utf-8')
  const sidebar = fs.readFileSync(STORAGE_CLASSES_SIDEBAR_PATH, 'utf-8')

  // Must import from storageQuantity
  assert.ok(
    metricsCards.includes('@/utils/storageQuantity'),
    'StorageMetricsCards.vue should import from @/utils/storageQuantity'
  )
  assert.ok(
    sidebar.includes('@/utils/storageQuantity'),
    'StorageClassesSidebar.vue should import from @/utils/storageQuantity'
  )

  // Must NOT define local parseCapacityToGiB
  assert.ok(
    !metricsCards.includes('const parseCapacityToGiB ='),
    'StorageMetricsCards.vue must not define local parseCapacityToGiB'
  )
  assert.ok(
    !sidebar.includes('const parseCapacityToGiB ='),
    'StorageClassesSidebar.vue must not define local parseCapacityToGiB'
  )
})

test('StorageClassesSidebar.vue has no hardcoded 2,620 total or raw palette icon colors', () => {
  const sidebar = fs.readFileSync(STORAGE_CLASSES_SIDEBAR_PATH, 'utf-8')

  assert.ok(
    !sidebar.includes('2,620'),
    'StorageClassesSidebar.vue must not contain hardcoded 2,620'
  )
  assert.ok(!sidebar.includes('text-amber-400'), 'Must not use text-amber-400')
  assert.ok(!sidebar.includes('text-violet-400'), 'Must not use text-violet-400')
  assert.ok(!sidebar.includes('text-emerald-400'), 'Must not use text-emerald-400')
})

test('useWorkloadActions does not exclude PersistentVolume or PersistentVolumeClaim from View Details', () => {
  const actionsContent = fs.readFileSync(
    path.resolve('src/composables/useWorkloadActions.ts'),
    'utf-8'
  )
  const match = actionsContent.match(/const noDetailsKinds:[^=]+=\s*\[([^\]]*)\]/)
  assert.ok(match, 'noDetailsKinds array must exist in useWorkloadActions.ts')
  const arrayText = match[1]
  assert.ok(
    !arrayText.includes('PersistentVolume'),
    'noDetailsKinds must not contain PersistentVolume'
  )
  assert.ok(
    !arrayText.includes('PersistentVolumeClaim'),
    'noDetailsKinds must not contain PersistentVolumeClaim'
  )
})

test('Storage details drawers exist and render BaseResourceDrawer, events, yaml, and edit button', () => {
  const drawers = [
    'src/components/storage/PersistentVolumeDetailsDrawer.vue',
    'src/components/storage/PersistentVolumeClaimDetailsDrawer.vue',
    'src/components/storage/StorageClassDetailsDrawer.vue'
  ]

  for (const relPath of drawers) {
    const filePath = path.resolve(relPath)
    assert.ok(fs.existsSync(filePath), `${relPath} must exist`)
    const content = fs.readFileSync(filePath, 'utf-8')
    assert.ok(content.includes('<BaseResourceDrawer'), `${relPath} must use BaseResourceDrawer`)
    assert.ok(content.includes('<WorkloadEventsTab'), `${relPath} must render WorkloadEventsTab`)
    assert.ok(content.includes('<ResourceYamlTab'), `${relPath} must render ResourceYamlTab`)
    assert.ok(
      content.includes('handleEdit') || content.includes('Edit'),
      `${relPath} must have Edit action`
    )
    assert.match(content, /data-component="[a-z0-9-]+"/, `${relPath} must have data-component`)
  }
})

test('Storage tables wire #drawer slot and contain zero native title attributes', () => {
  const tables = [
    'src/components/storage/StoragePVTable.vue',
    'src/components/storage/StoragePVCTable.vue',
    'src/components/storage/StorageClassesTable.vue'
  ]

  for (const relPath of tables) {
    const filePath = path.resolve(relPath)
    const content = fs.readFileSync(filePath, 'utf-8')
    assert.ok(
      content.includes('<template #drawer="{ visible, selectedItem, close }">'),
      `${relPath} must wire <template #drawer="{ visible, selectedItem, close }">`
    )
    assert.ok(!content.includes(':title='), `${relPath} must not use :title= native tooltip`)
    assert.ok(
      !content.match(/<[^>]+\stitle="[^"]*"/),
      `${relPath} must not use native title=" attribute`
    )
  }
})

test('EditView maps PersistentVolume, PersistentVolumeClaim, and StorageClass and handles cluster-scoped header', () => {
  const editViewContent = fs.readFileSync(path.resolve('src/views/EditView.vue'), 'utf-8')
  assert.ok(
    editViewContent.includes('PersistentVolumeEditForm'),
    'Must import PersistentVolumeEditForm'
  )
  assert.ok(
    editViewContent.includes('PersistentVolumeClaimEditForm'),
    'Must import PersistentVolumeClaimEditForm'
  )
  assert.ok(editViewContent.includes('StorageClassEditForm'), 'Must import StorageClassEditForm')
  assert.ok(editViewContent.includes('isClusterScoped'), 'Must compute isClusterScoped')
})

test('StorageView wires useDialog and per-tab createAction for PV, PVC, and StorageClass', () => {
  const storageViewContent = fs.readFileSync(path.resolve('src/views/StorageView.vue'), 'utf-8')
  assert.ok(storageViewContent.includes('useDialog'), 'StorageView must import useDialog')
  assert.ok(
    storageViewContent.includes('CreatePVDialog'),
    'StorageView must reference CreatePVDialog'
  )
  assert.ok(
    storageViewContent.includes('CreatePVCDialog'),
    'StorageView must reference CreatePVCDialog'
  )
  assert.ok(
    storageViewContent.includes('CreateStorageClassDialog'),
    'StorageView must reference CreateStorageClassDialog'
  )
  assert.ok(storageViewContent.includes('createAction'), 'StorageView must define createAction')
})

test('Storage create dialogs exist and call kubernetesService.createResource', () => {
  const dialogs = [
    'src/components/storage/CreatePVDialog.vue',
    'src/components/storage/CreatePVCDialog.vue',
    'src/components/storage/CreateStorageClassDialog.vue'
  ]

  for (const relPath of dialogs) {
    const filePath = path.resolve(relPath)
    assert.ok(fs.existsSync(filePath), `${relPath} must exist`)
    const content = fs.readFileSync(filePath, 'utf-8')
    assert.ok(
      content.includes('kubernetesService.createResource'),
      `${relPath} must call kubernetesService.createResource`
    )
    assert.match(content, /data-component="[a-z0-9-]+"/, `${relPath} must have data-component`)
  }
})
