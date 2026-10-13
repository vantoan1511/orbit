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

test('enforces non-null Content Security Policy in tauri.conf.json', () => {
  const tauriConfPath = path.resolve(process.cwd(), 'src-tauri/tauri.conf.json')
  const tauriConf = JSON.parse(fs.readFileSync(tauriConfPath, 'utf-8'))

  const csp = tauriConf.app?.security?.csp
  assert.ok(csp, 'CSP in tauri.conf.json must not be null or empty')
  assert.match(csp, /default-src 'self'/)
  assert.match(csp, /script-src 'self'/)
})
