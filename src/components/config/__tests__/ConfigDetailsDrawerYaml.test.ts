import assert from 'node:assert/strict'
import test from 'node:test'
import fs from 'node:fs'
import path from 'node:path'

const DRAWER_PATH = path.resolve('src/components/config/ConfigDetailsDrawer.vue')

test('ConfigDetailsDrawer.vue uses useResourceRawManifest and ResourceYamlTab instead of generateYaml', () => {
  const content = fs.readFileSync(DRAWER_PATH, 'utf-8')

  // Must NOT define synthetic generateYaml
  assert.ok(
    !content.includes('generateYaml'),
    'ConfigDetailsDrawer.vue must not contain generateYaml'
  )

  // Must import useResourceRawManifest and ResourceYamlTab
  assert.ok(
    content.includes('@/composables/useResourceRawManifest'),
    'ConfigDetailsDrawer.vue must import useResourceRawManifest'
  )
  assert.ok(
    content.includes('@/components/shared/ResourceYamlTab.vue'),
    'ConfigDetailsDrawer.vue must import ResourceYamlTab'
  )

  // Must invoke useResourceRawManifest
  assert.ok(
    content.includes('useResourceRawManifest('),
    'ConfigDetailsDrawer.vue must invoke useResourceRawManifest'
  )

  // Must render ResourceYamlTab with standard props
  assert.ok(
    content.includes('<ResourceYamlTab'),
    'ConfigDetailsDrawer.vue must render ResourceYamlTab'
  )
  assert.ok(
    content.includes(':displayed-yaml="displayedYaml"'),
    'ConfigDetailsDrawer.vue must pass :displayed-yaml="displayedYaml"'
  )
  assert.ok(
    content.includes(':is-yaml-loading="isYamlLoading"'),
    'ConfigDetailsDrawer.vue must pass :is-yaml-loading="isYamlLoading"'
  )
  assert.ok(
    content.includes(':copied="copied"'),
    'ConfigDetailsDrawer.vue must pass :copied="copied"'
  )
  assert.ok(
    content.includes('@copy-yaml="copyYaml"'),
    'ConfigDetailsDrawer.vue must handle @copy-yaml="copyYaml"'
  )
})
