import assert from 'node:assert/strict'
import fs from 'node:fs'
import path from 'node:path'
import test from 'node:test'

test('AppSidebarActivityBar renders New UI button only when enableExperienceFeatures is enabled', () => {
  const filePath = path.resolve(
    process.cwd(),
    'src/components/layout/sidebar/AppSidebarActivityBar.vue'
  )
  const content = fs.readFileSync(filePath, 'utf-8')

  assert.ok(
    content.includes('CONFIG_KEYS.ENABLE_EXPERIENCE_FEATURES'),
    'AppSidebarActivityBar must reference CONFIG_KEYS.ENABLE_EXPERIENCE_FEATURES'
  )
  assert.ok(
    content.includes('v-tooltip.right="\'New UI (GPUI)\'"'),
    'New UI button must use PrimeVue v-tooltip.right directive'
  )
  assert.ok(
    content.includes('appSettingsService.launchGpuiShell()'),
    'New UI button must call appSettingsService.launchGpuiShell()'
  )
  assert.ok(
    !content.includes('@tauri-apps/api'),
    'AppSidebarActivityBar must not import @tauri-apps/api directly'
  )
})
