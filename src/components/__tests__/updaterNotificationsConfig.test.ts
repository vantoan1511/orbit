import assert from 'node:assert/strict'
import fs from 'node:fs'
import path from 'node:path'
import test from 'node:test'
import { CONFIG_KEYS } from '../../types/settings.ts'

test('CONFIG_KEYS defines AUTO_CHECK_UPDATES as "autoCheckUpdates"', () => {
  assert.equal(CONFIG_KEYS.AUTO_CHECK_UPDATES, 'autoCheckUpdates')
})

test('UpdaterNotifications.vue integrates useSettingsStore and respects CONFIG_KEYS.AUTO_CHECK_UPDATES', () => {
  const filePath = path.resolve(process.cwd(), 'src/components/UpdaterNotifications.vue')
  assert.ok(fs.existsSync(filePath), 'UpdaterNotifications.vue must exist')

  const content = fs.readFileSync(filePath, 'utf-8')

  assert.ok(
    content.includes('useSettingsStore'),
    'UpdaterNotifications.vue must import useSettingsStore'
  )
  assert.ok(content.includes('CONFIG_KEYS'), 'UpdaterNotifications.vue must import CONFIG_KEYS')
  assert.ok(
    content.includes('CONFIG_KEYS.AUTO_CHECK_UPDATES') ||
      content.includes("CONFIG_KEYS['AUTO_CHECK_UPDATES']") ||
      content.includes('autoCheckUpdates'),
    'UpdaterNotifications.vue must query AUTO_CHECK_UPDATES before checking for updates'
  )
  assert.ok(
    content.includes('getConfigValue'),
    'UpdaterNotifications.vue must read config via settingsStore.getConfigValue'
  )
})

test('UpdaterNotifications.vue checks NOTIFICATION_PREFERENCES enableToasts and notifyOnUpdates before triggering toast', () => {
  const filePath = path.resolve(process.cwd(), 'src/components/UpdaterNotifications.vue')
  const content = fs.readFileSync(filePath, 'utf-8')

  assert.ok(
    content.includes('CONFIG_KEYS.NOTIFICATION_PREFERENCES') ||
      content.includes('NOTIFICATION_PREFERENCES'),
    'UpdaterNotifications.vue must reference NOTIFICATION_PREFERENCES'
  )
  assert.ok(
    content.includes('DEFAULT_NOTIFICATION_PREFERENCES'),
    'UpdaterNotifications.vue must reference DEFAULT_NOTIFICATION_PREFERENCES'
  )
  assert.ok(
    content.includes('enableToasts') && content.includes('notifyOnUpdates'),
    'UpdaterNotifications.vue must check enableToasts and notifyOnUpdates before displaying toast'
  )
})
