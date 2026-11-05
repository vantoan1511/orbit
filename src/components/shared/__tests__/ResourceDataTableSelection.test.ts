import assert from 'node:assert/strict'
import fs from 'node:fs'
import path from 'node:path'
import test from 'node:test'
import { resolveFieldData } from '@primeuix/utils/object'
import { getResourceCompositeKey, type IdentifiableResource } from '../../../utils/resourceKey.ts'

test('GenericResourceTable defines dataKey default as a factory returning a key function', () => {
  const filePath = path.resolve('src/components/shared/GenericResourceTable.vue')
  const content = fs.readFileSync(filePath, 'utf-8')

  // Must declare dataKey default with () => factory wrapper to avoid Vue executing it with rawProps
  assert.match(
    content,
    /dataKey:\s*\(\)\s*=>\s*\(r:\s*IdentifiableResource\)\s*=>\s*getResourceCompositeKey\(r\)/,
    'GenericResourceTable must declare dataKey default as () => (r: IdentifiableResource) => getResourceCompositeKey(r)'
  )
})

test('ResourceDataTable defines dataKey default as a factory returning getResourceCompositeKey', () => {
  const filePath = path.resolve('src/components/shared/ResourceDataTable.vue')
  const content = fs.readFileSync(filePath, 'utf-8')

  // Must declare dataKey default with () => factory wrapper and use composite key instead of static 'name'
  assert.match(
    content,
    /dataKey:\s*\(\)\s*=>\s*\(r:\s*any\)\s*=>\s*getResourceCompositeKey\(r\)/,
    'ResourceDataTable must declare dataKey default as () => (r: any) => getResourceCompositeKey(r)'
  )
  assert.doesNotMatch(
    content,
    /dataKey:\s*'name'/,
    'ResourceDataTable must not default dataKey to static "name"'
  )
})

test('Single row selection generates unique selection key and does not mark other rows selected', () => {
  const rowA: IdentifiableResource = { name: 'coredns', namespace: 'kube-system', uid: 'uid-a' }
  const rowB: IdentifiableResource = { name: 'coredns', namespace: 'default', uid: 'uid-b' }
  const rowC: IdentifiableResource = { name: 'metrics-server', namespace: 'kube-system' }

  const keyFn = (r: IdentifiableResource) => getResourceCompositeKey(r)

  // Verify resolveFieldData with keyFn returns unique composite strings
  const keyA = resolveFieldData(rowA, keyFn)
  const keyB = resolveFieldData(rowB, keyFn)
  const keyC = resolveFieldData(rowC, keyFn)

  assert.equal(keyA, 'uid-a')
  assert.equal(keyB, 'uid-b')
  assert.equal(keyC, 'kube-system/metrics-server')
  assert.notEqual(keyA, keyB)
  assert.notEqual(keyA, keyC)
  assert.notEqual(keyB, keyC)

  // Simulate PrimeVue DataTable selection logic:
  // User selects only rowA:
  const selection = [rowA]
  const d_selectionKeys: Record<string, number> = {}
  for (const item of selection) {
    const k = resolveFieldData(item, keyFn)
    if (k != null) {
      d_selectionKeys[String(k)] = 1
    }
  }

  // Row A is selected:
  const isRowASelected = d_selectionKeys[String(resolveFieldData(rowA, keyFn))] !== undefined
  assert.equal(isRowASelected, true, 'rowA must be selected')

  // Row B and Row C must NOT be selected:
  const isRowBSelected = d_selectionKeys[String(resolveFieldData(rowB, keyFn))] !== undefined
  const isRowCSelected = d_selectionKeys[String(resolveFieldData(rowC, keyFn))] !== undefined
  assert.equal(isRowBSelected, false, 'rowB must not be selected when rowA is selected')
  assert.equal(isRowCSelected, false, 'rowC must not be selected when rowA is selected')
})
