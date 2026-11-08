import assert from 'node:assert/strict'
import fs from 'node:fs'
import path from 'node:path'
import test from 'node:test'

test('GenericResourceTable: implements async tab backend data hydration contract', () => {
  const filePath = path.resolve('src/components/shared/GenericResourceTable.vue')
  const content = fs.readFileSync(filePath, 'utf-8')

  // 1. Must import useDeferredMount and call it
  assert.match(
    content,
    /import\s+\{[^}]*useDeferredMount[^}]*\}\s+from\s+['"]@\/composables\/useDeferredMount['"]/,
    'GenericResourceTable must import useDeferredMount'
  )
  assert.match(
    content,
    /const\s+\{\s*isHydrated\s*\}\s*=\s*useDeferredMount\(\)/,
    'GenericResourceTable must call useDeferredMount()'
  )

  // 2. Must pass isHydrated to useResourceFilters
  assert.match(
    content,
    /useResourceFilters\(\s*dataRef,\s*props\.searchFields\s+as\s+\(keyof\s+T\)\[\],\s*resolvedStoreKey,\s*hideNamespaceFilterRef,\s*isHydrated\s*\)/,
    'GenericResourceTable must pass isHydrated to useResourceFilters'
  )

  // 3. Must ensure Namespace is loaded when !props.hideNamespaceFilter
  assert.match(
    content,
    /ensureTableKindsLoaded/,
    'GenericResourceTable must define ensureTableKindsLoaded helper'
  )
  assert.match(
    content,
    /k8sStore\.ensureKindLoaded\(KUBERNETES_RESOURCE_KIND\.Namespace\)/,
    'GenericResourceTable must ensure Namespace kind is loaded when namespace filter is visible'
  )

  // 4. Must bind hydratedNamespaces and hydratedSelectedNamespace to NamespaceFilter
  assert.match(
    content,
    /const\s+hydratedNamespaces\s*=\s*computed\(\(\)\s*=>\s*\(isHydrated\.value\s*\?\s*k8sStore\.namespaces\s*:\s*\[\]\)\)/,
    'GenericResourceTable must compute hydratedNamespaces guarded by isHydrated'
  )
  assert.match(
    content,
    /const\s+hydratedSelectedNamespace\s*=\s*computed/,
    'GenericResourceTable must compute hydratedSelectedNamespace'
  )
  assert.match(
    content,
    /<NamespaceFilter[\s\S]*?:namespaces="hydratedNamespaces"/,
    'GenericResourceTable must bind :namespaces="hydratedNamespaces" to NamespaceFilter'
  )
  assert.match(
    content,
    /<NamespaceFilter[\s\S]*?v-model="hydratedSelectedNamespace"/,
    'GenericResourceTable must bind v-model="hydratedSelectedNamespace" to NamespaceFilter'
  )

  // 5. Must guard filteredData, portForwardsMap, and storedRowKeys behind isHydrated
  assert.match(
    content,
    /const\s+filteredData\s*=\s*computed\(\(\)\s*=>\s*\{\s*if\s*\(!isHydrated\.value\)\s*return\s*\[\]/,
    'filteredData must return [] when !isHydrated.value'
  )
  assert.match(
    content,
    /if\s*\(!isHydrated\.value\s*\|\|\s*!props\.kind\s*\|\|\s*k8sStore\.activePortForwards\.length\s*===\s*0\)\s*return\s*map/,
    'portForwardsMap must return empty map when !isHydrated.value'
  )
  assert.doesNotMatch(
    content,
    /if\s*\(storedRowKeys\.length\s*>\s*0\)\s*\{\s*const\s*rowKeySet\s*=\s*new\s*Set\(storedRowKeys\)/,
    'storedRowKeys restoration should not run synchronously on frame 0 in setup'
  )
  assert.match(
    content,
    /watch\(\s*\[\(\)\s*=>\s*props\.data,\s*isHydrated\]\s*as\s+const,\s*\(\[newData,\s*hydrated\]\)\s*=>\s*\{\s*if\s*\(!hydrated\)\s*return/,
    'storedRowKeys restoration must be watched with isHydrated'
  )

  // 6. Must expose isHydrated on filters slot
  assert.match(
    content,
    /<slot\s+name="filters"\s+:isHydrated="isHydrated">\s*<\/slot>/,
    'GenericResourceTable must expose :isHydrated on filters slot'
  )
})
