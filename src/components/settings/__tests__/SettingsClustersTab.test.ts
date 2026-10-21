import assert from 'node:assert/strict'
import fs from 'node:fs'
import path from 'node:path'
import test from 'node:test'

test('SettingsClustersTab exists and defines data-component="settings-clusters-tab"', () => {
  const filePath = path.resolve(process.cwd(), 'src/components/settings/SettingsClustersTab.vue')
  assert.ok(fs.existsSync(filePath), 'SettingsClustersTab.vue must exist')

  const content = fs.readFileSync(filePath, 'utf-8')
  assert.ok(
    content.includes('data-component="settings-clusters-tab"'),
    'SettingsClustersTab must define data-component="settings-clusters-tab"'
  )
})

test('SettingsClustersTab uses PrimeVue DataTable, search filtering, and ClusterEditModal', () => {
  const filePath = path.resolve(process.cwd(), 'src/components/settings/SettingsClustersTab.vue')
  const content = fs.readFileSync(filePath, 'utf-8')

  assert.ok(content.includes('DataTable'), 'SettingsClustersTab must use PrimeVue DataTable')
  assert.ok(content.includes('Column'), 'SettingsClustersTab must use PrimeVue Column')
  assert.ok(content.includes('InputText'), 'SettingsClustersTab must use PrimeVue InputText for search')
  assert.ok(content.includes('ClusterEditModal'), 'SettingsClustersTab must integrate ClusterEditModal')
  assert.ok(content.includes('getClusterDisplayName'), 'SettingsClustersTab must use getClusterDisplayName')
  assert.ok(content.includes('filterClusters'), 'SettingsClustersTab must use filterClusters')
  assert.ok(content.includes('handleAddCluster'), 'SettingsClustersTab must support adding cluster via handleAddCluster')
  assert.ok(!content.includes(['@neutralinojs', 'lib'].join('/')), 'SettingsClustersTab must not directly import native runtime')
})
