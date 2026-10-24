import assert from 'node:assert/strict'
import fs from 'node:fs'
import path from 'node:path'
import test from 'node:test'

test('enforces borderless decorations in tauri.conf.json', () => {
  const tauriConfPath = path.resolve(process.cwd(), 'src-tauri/tauri.conf.json')
  const content = fs.readFileSync(tauriConfPath, 'utf-8')
  const conf = JSON.parse(content)

  const mainWindow = conf.app?.windows?.find((w: { title?: string }) => w.title === 'Orbit')
  assert.ok(mainWindow, 'Orbit window must exist in tauri.conf.json')
  assert.equal(
    mainWindow.decorations,
    false,
    'Main window must have decorations: false for borderless style'
  )
  assert.equal(
    mainWindow.dragDropEnabled,
    false,
    'Main window must have dragDropEnabled: false to enable HTML5 webview drag-and-drop'
  )
})

test('enforces borderless decorations and window settings in tauri.dev.conf.json', () => {
  const tauriDevConfPath = path.resolve(process.cwd(), 'src-tauri/tauri.dev.conf.json')
  const content = fs.readFileSync(tauriDevConfPath, 'utf-8')
  const conf = JSON.parse(content)

  const devWindow = conf.app?.windows?.find((w: { title?: string }) => w.title === 'Orbit [Dev]')
  assert.ok(devWindow, 'Orbit [Dev] window must exist in tauri.dev.conf.json')
  assert.equal(
    devWindow.decorations,
    false,
    'Dev window must have decorations: false for borderless style'
  )
  assert.equal(
    devWindow.dragDropEnabled,
    false,
    'Dev window must have dragDropEnabled: false to enable HTML5 webview drag-and-drop'
  )
  assert.equal(devWindow.width, 1280, 'Dev window width must match main window')
  assert.equal(devWindow.height, 720, 'Dev window height must match main window')
  assert.equal(devWindow.minWidth, 1280, 'Dev window minWidth must match main window')
  assert.equal(devWindow.minHeight, 720, 'Dev window minHeight must match main window')
  assert.equal(devWindow.resizable, true, 'Dev window resizable must match main window')
})

test('enforces required core:window permissions in capabilities/default.json', () => {
  const capabilitiesPath = path.resolve(process.cwd(), 'src-tauri/capabilities/default.json')
  const content = fs.readFileSync(capabilitiesPath, 'utf-8')
  const capabilities = JSON.parse(content)

  const requiredPermissions = [
    'core:window:allow-minimize',
    'core:window:allow-maximize',
    'core:window:allow-unmaximize',
    'core:window:allow-toggle-maximize',
    'core:window:allow-close',
    'core:window:allow-is-maximized',
    'core:window:allow-start-dragging'
  ]

  for (const perm of requiredPermissions) {
    assert.ok(
      capabilities.permissions?.includes(perm),
      `capabilities/default.json must include permission ${perm}`
    )
  }
})
