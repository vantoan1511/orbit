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
    content.includes('v-tooltip.right=') &&
      content.includes("'New UI (GPUI)'") &&
      content.includes("'Launching New UI (GPUI)...'"),
    'New UI button must use PrimeVue v-tooltip.right directive with both idle and launching labels'
  )
  assert.ok(
    content.includes(':loading="isLaunchingGpui"'),
    'New UI button must bind :loading="isLaunchingGpui"'
  )
  assert.ok(
    content.includes(':disabled="isLaunchingGpui"'),
    'New UI button must bind :disabled="isLaunchingGpui"'
  )
  assert.ok(
    content.includes('appSettingsService.launchGpuiShell()'),
    'New UI button must call appSettingsService.launchGpuiShell()'
  )
  assert.ok(
    content.includes('try {') &&
      content.includes('catch') &&
      content.includes('notificationStore.addNotification'),
    'New UI launch handler must catch IPC dispatch errors and report via notificationStore'
  )
  assert.ok(
    content.includes('onUnmounted'),
    'AppSidebarActivityBar must clean up timers and listeners in onUnmounted'
  )
  assert.ok(
    !content.includes('@tauri-apps/api'),
    'AppSidebarActivityBar must not import @tauri-apps/api directly'
  )
})
