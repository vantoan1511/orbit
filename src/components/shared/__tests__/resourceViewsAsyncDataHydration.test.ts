import assert from 'node:assert/strict'
import fs from 'node:fs'
import path from 'node:path'
import test from 'node:test'

test('NamespacesDataTable, ConfigDataTable, and StoragePVTable gate data and dynamic filters behind isHydrated', () => {
  // NamespacesDataTable
  const nsContent = fs.readFileSync(
    path.resolve('src/components/namespaces/NamespacesDataTable.vue'),
    'utf-8'
  )
  assert.match(nsContent, /useDeferredMount/, 'NamespacesDataTable must import useDeferredMount')
  assert.match(
    nsContent,
    /const\s+labelOptions\s*=\s*computed\(\(\)\s*=>\s*\{\s*if\s*\(!isHydrated\.value\)\s*return\s*\['All Labels'\]/,
    'NamespacesDataTable must guard labelOptions with isHydrated'
  )
  assert.match(
    nsContent,
    /const\s+mappedNamespaces\s*=\s*computed\(\(\)\s*=>\s*\{\s*if\s*\(!isHydrated\.value\)\s*return\s*\[\]/,
    'NamespacesDataTable must guard mappedNamespaces with isHydrated'
  )

  // ConfigDataTable
  const cfgContent = fs.readFileSync(
    path.resolve('src/components/config/ConfigDataTable.vue'),
    'utf-8'
  )
  assert.match(cfgContent, /useDeferredMount/, 'ConfigDataTable must import useDeferredMount')
  assert.match(
    cfgContent,
    /const\s+labels\s*=\s*computed\(\(\)\s*=>\s*\{\s*if\s*\(!isHydrated\.value\)\s*return\s*\['All Labels'\]/,
    'ConfigDataTable must guard labels with isHydrated'
  )
  assert.match(
    cfgContent,
    /const\s+filteredItems\s*=\s*computed\(\(\)\s*=>\s*\{\s*if\s*\(!isHydrated\.value\)\s*return\s*\[\]/,
    'ConfigDataTable must guard filteredItems with isHydrated'
  )

  // StoragePVTable
  const pvContent = fs.readFileSync(
    path.resolve('src/components/storage/StoragePVTable.vue'),
    'utf-8'
  )
  assert.match(pvContent, /useDeferredMount/, 'StoragePVTable must import useDeferredMount')
  assert.match(
    pvContent,
    /const\s+storageClasses\s*=\s*computed\(\(\)\s*=>\s*\{\s*if\s*\(!isHydrated\.value\)\s*return\s*\['All Storage Classes'\]/,
    'StoragePVTable must guard storageClasses with isHydrated'
  )
  assert.match(
    pvContent,
    /const\s+filteredPVs\s*=\s*computed\(\(\)\s*=>\s*\{\s*if\s*\(!isHydrated\.value\)\s*return\s*\[\]/,
    'StoragePVTable must guard filteredPVs with isHydrated'
  )
})

test('ServicesTable, IngressesTable, PortForwardsTable, EventsDataTable, NodesTable, PoliciesDataTable, StoragePVCTable, and StorageClassesTable gate data behind isHydrated', () => {
  const tables = [
    {
      file: 'src/components/network/ServicesTable.vue',
      guard: /if\s*\(!isHydrated\.value\)\s*return\s*\[\]/
    },
    {
      file: 'src/components/network/IngressesTable.vue',
      guard: /computed\(\(\)\s*=>\s*\(isHydrated\.value\s*\?\s*k8sStore\.ingresses\s*:\s*\[\]\)\)/
    },
    {
      file: 'src/components/network/PortForwardsTable.vue',
      guard: /if\s*\(!isHydrated\.value\)\s*return\s*\[\]/
    },
    {
      file: 'src/components/events/EventsDataTable.vue',
      guard: /if\s*\(!isHydrated\.value\)\s*return\s*\[\]/
    },
    {
      file: 'src/components/nodes/NodesTable.vue',
      guard: /computed\(\(\)\s*=>\s*\(isHydrated\.value\s*\?\s*k8sStore\.nodes\s*:\s*\[\]\)\)/
    },
    {
      file: 'src/components/policies/PoliciesDataTable.vue',
      guard: /if\s*\(!isHydrated\.value\)\s*return\s*\[\]/
    },
    {
      file: 'src/components/storage/StoragePVCTable.vue',
      guard:
        /computed\(\(\)\s*=>\s*\(isHydrated\.value\s*\?\s*k8sStore\.persistentVolumeClaims\s*:\s*\[\]\)\)/
    },
    {
      file: 'src/components/storage/StorageClassesTable.vue',
      guard:
        /computed\(\(\)\s*=>\s*\(isHydrated\.value\s*\?\s*k8sStore\.storageClasses\s*:\s*\[\]\)\)/
    }
  ]

  for (const { file, guard } of tables) {
    const content = fs.readFileSync(path.resolve(file), 'utf-8')
    assert.match(content, /useDeferredMount/, `${file} must import useDeferredMount`)
    assert.match(content, guard, `${file} must gate table data with isHydrated`)
  }
})

test('NamespaceMetricsCards and StorageClassesSidebar gate chart and capacity watchers behind isHydrated', () => {
  // NamespaceMetricsCards
  const nmcContent = fs.readFileSync(
    path.resolve('src/components/namespaces/NamespaceMetricsCards.vue'),
    'utf-8'
  )
  assert.match(
    nmcContent,
    /watch\(\s*\[isHydrated,\s*activeCount,\s*terminatingCount,\s*systemCount,\s*totalNamespaces\],\s*\(\[hydrated\]\)\s*=>\s*\{\s*if\s*\(!hydrated\)\s*return/,
    'NamespaceMetricsCards must gate donut watcher behind isHydrated'
  )
  assert.match(
    nmcContent,
    /const\s+activeCount\s*=\s*computed\(\(\)\s*=>\s*\{\s*if\s*\(!isHydrated\.value\)\s*return\s*0/,
    'NamespaceMetricsCards must guard activeCount when !isHydrated.value'
  )

  // StorageClassesSidebar
  const scsContent = fs.readFileSync(
    path.resolve('src/components/storage/StorageClassesSidebar.vue'),
    'utf-8'
  )
  assert.match(scsContent, /useDeferredMount/, 'StorageClassesSidebar must import useDeferredMount')
  assert.match(
    scsContent,
    /const\s+scCapacityData\s*=\s*computed\(\(\)\s*=>\s*\{\s*if\s*\(!isHydrated\.value\)\s*return\s*\{\}/,
    'StorageClassesSidebar must guard scCapacityData when !isHydrated.value'
  )
  assert.match(
    scsContent,
    /watch\(\s*\[isHydrated,\s*scCapacityData,\s*isDark\],\s*\(\[hydrated\]\)\s*=>\s*\{\s*if\s*\(!hydrated\)\s*return/,
    'StorageClassesSidebar must gate updateChart watcher behind isHydrated'
  )
  assert.match(
    scsContent,
    /<ResourceMetricsSkeleton[\s\S]*?v-if="[\s\S]*?shouldShowSkeleton/,
    'StorageClassesSidebar must render ResourceMetricsSkeleton with shouldShowSkeleton'
  )
})
