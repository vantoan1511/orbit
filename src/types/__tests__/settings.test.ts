import assert from 'node:assert/strict'
import test from 'node:test'
import {
  CONFIG_KEYS,
  DEFAULT_NOTIFICATION_PREFERENCES,
  DEFAULT_PROXY_SETTINGS
} from '../settings.ts'

test('CONFIG_KEYS defines all well-known configuration keys', () => {
  assert.equal(CONFIG_KEYS.MAX_LOG_FILES, 'maxLogFiles')
  assert.equal(CONFIG_KEYS.CUSTOM_KUBECONFIG_PATHS, 'customKubeconfigPaths')
  assert.equal(CONFIG_KEYS.AUTO_CHECK_UPDATES, 'autoCheckUpdates')
  assert.equal(CONFIG_KEYS.LAUNCH_ON_STARTUP, 'launchOnStartup')
  assert.equal(CONFIG_KEYS.CLOSE_TO_TRAY, 'closeToTray')
  assert.equal('SHARE_TELEMETRY' in CONFIG_KEYS, false)
  assert.equal(CONFIG_KEYS.DEFAULT_TERMINAL_SHELL, 'defaultTerminalShell')
  assert.equal(CONFIG_KEYS.KEYBOARD_SHORTCUTS, 'keyboardShortcuts')
  assert.equal(CONFIG_KEYS.THEME_MODE, 'themeMode')
  assert.equal(CONFIG_KEYS.ACCENT_COLOR, 'accentColor')
  assert.equal(CONFIG_KEYS.UI_STYLE, 'uiStyle')
  assert.equal(CONFIG_KEYS.NOTIFICATION_PREFERENCES, 'notificationPreferences')
  assert.equal(CONFIG_KEYS.PROXY_SETTINGS, 'proxySettings')
})

test('isGeneralConfigKey filters out specialized keys', async () => {
  const { SPECIALIZED_CONFIG_KEYS, isGeneralConfigKey } = await import('../settings.ts')

  assert.ok(SPECIALIZED_CONFIG_KEYS.includes('keyboardShortcuts'))
  assert.ok(SPECIALIZED_CONFIG_KEYS.includes('themeMode'))
  assert.ok(SPECIALIZED_CONFIG_KEYS.includes('accentColor'))
  assert.ok(SPECIALIZED_CONFIG_KEYS.includes('uiStyle'))
  assert.ok(SPECIALIZED_CONFIG_KEYS.includes('notificationPreferences'))
  assert.ok(SPECIALIZED_CONFIG_KEYS.includes('proxySettings'))

  assert.equal(isGeneralConfigKey('maxLogFiles'), true)
  assert.equal(isGeneralConfigKey('customKubeconfigPaths'), true)
  assert.equal(isGeneralConfigKey('launchOnStartup'), true)
  assert.equal(isGeneralConfigKey('closeToTray'), true)
  assert.equal(isGeneralConfigKey('defaultTerminalShell'), true)
  assert.equal(isGeneralConfigKey('keyboardShortcuts'), false)
  assert.equal(isGeneralConfigKey('themeMode'), false)
  assert.equal(isGeneralConfigKey('accentColor'), false)
  assert.equal(isGeneralConfigKey('uiStyle'), false)
  assert.equal(isGeneralConfigKey('notificationPreferences'), false)
  assert.equal(isGeneralConfigKey('proxySettings'), false)
})

test('DEFAULT_NOTIFICATION_PREFERENCES and DEFAULT_PROXY_SETTINGS define canonical defaults', () => {
  assert.deepEqual(DEFAULT_NOTIFICATION_PREFERENCES, {
    enableToasts: true,
    notifyOnSuccess: true,
    notifyOnError: true,
    notifyOnUpdates: true
  })
  assert.deepEqual(DEFAULT_PROXY_SETTINGS, {
    enabled: false,
    httpProxy: '',
    httpsProxy: '',
    noProxy: ''
  })
})
