import assert from 'node:assert/strict'
import fs from 'node:fs'
import path from 'node:path'
import test from 'node:test'

test('enforces tauri.dev.conf.json exists with isolated identifier and package script', () => {
  const devConfPath = path.resolve(process.cwd(), 'src-tauri/tauri.dev.conf.json')
  assert.ok(fs.existsSync(devConfPath), 'src-tauri/tauri.dev.conf.json must exist')

  const devConf = JSON.parse(fs.readFileSync(devConfPath, 'utf-8'))
  assert.equal(devConf.identifier, 'vantoan1511.orbit.dev', 'Dev identifier must be vantoan1511.orbit.dev')
  assert.equal(devConf.productName, 'Orbit Dev', 'Dev productName must be Orbit Dev')
  assert.equal(devConf.app?.windows?.[0]?.title, 'Orbit [Dev]', 'Dev window title must be Orbit [Dev]')

  const packageJsonPath = path.resolve(process.cwd(), 'package.json')
  const packageJson = JSON.parse(fs.readFileSync(packageJsonPath, 'utf-8'))
  assert.match(
    packageJson.scripts?.['tauri:dev'] ?? '',
    /--config src-tauri\/tauri\.dev\.conf\.json/,
    'package.json tauri:dev script must use --config src-tauri/tauri.dev.conf.json'
  )
})
