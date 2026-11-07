import assert from 'node:assert/strict'
import fs from 'node:fs'
import path from 'node:path'
import test from 'node:test'

test('enforces clean dependencies without legacy xterm duplication', () => {
  const packageJsonPath = path.resolve(process.cwd(), 'package.json')
  const packageJson = JSON.parse(fs.readFileSync(packageJsonPath, 'utf-8'))

  assert.equal(
    packageJson.dependencies?.['xterm'],
    undefined,
    'Legacy "xterm" dependency must be removed in favor of "@xterm/xterm"'
  )
  assert.ok(packageJson.dependencies?.['@xterm/xterm'], '"@xterm/xterm" must remain installed')
})

test('enforces local monaco-editor dependency without CDN loader wrapper', () => {
  const packageJsonPath = path.resolve(process.cwd(), 'package.json')
  const packageJson = JSON.parse(fs.readFileSync(packageJsonPath, 'utf-8'))

  assert.equal(
    packageJson.dependencies?.['@guolao/vue-monaco-editor'],
    undefined,
    'CDN loader wrapper "@guolao/vue-monaco-editor" must be removed'
  )
  assert.ok(
    packageJson.dependencies?.['monaco-editor'],
    'Bundled "monaco-editor" must be installed directly'
  )
})

test('enforces non-null Content Security Policy in tauri.conf.json', () => {
  const tauriConfPath = path.resolve(process.cwd(), 'src-tauri/tauri.conf.json')
  const tauriConf = JSON.parse(fs.readFileSync(tauriConfPath, 'utf-8'))

  const csp = tauriConf.app?.security?.csp
  assert.ok(csp, 'CSP in tauri.conf.json must not be null or empty')
  assert.match(csp, /default-src 'self'/)
  assert.match(csp, /script-src 'self'/)
  assert.match(csp, /worker-src 'self' blob:/)
  assert.ok(!csp.includes('cdn.jsdelivr.net'), 'CSP must not allow cdn.jsdelivr.net')
})

test('enforces canonical Tauri v2 $schema in tauri.conf.json', () => {
  const tauriConfPath = path.resolve(process.cwd(), 'src-tauri/tauri.conf.json')
  const tauriConf = JSON.parse(fs.readFileSync(tauriConfPath, 'utf-8'))

  assert.equal(
    tauriConf.$schema,
    'https://schema.tauri.app/config/2',
    'tauri.conf.json must point to official Tauri v2 schema'
  )
})

test('enforces documentation and Winget locale manifest reflect Tauri v2 and direct monaco-editor', () => {
  const readmePath = path.resolve(process.cwd(), 'README.md')
  const readmeContent = fs.readFileSync(readmePath, 'utf-8')
  assert.ok(
    !readmeContent.includes('@guolao/vue-monaco-editor'),
    'README.md must not reference legacy @guolao/vue-monaco-editor'
  )
  assert.ok(
    readmeContent.includes('(`monaco-editor`)'),
    'README.md must reference direct monaco-editor'
  )

  const wingetLocalePath = path.resolve(
    process.cwd(),
    'scripts/winget/vantoan1511.Orbit.locale.en-US.yaml'
  )
  const wingetLocaleContent = fs.readFileSync(wingetLocalePath, 'utf-8')
  assert.ok(
    !wingetLocaleContent.toLowerCase().includes('neutralino'),
    'Winget locale manifest must not reference Neutralinojs'
  )
  assert.ok(
    wingetLocaleContent.includes('Tauri v2'),
    'Winget locale manifest must reference Tauri v2'
  )
})
