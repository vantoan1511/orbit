import { describe, it } from 'node:test'
import assert from 'node:assert/strict'
import fs from 'node:fs'
import path from 'node:path'

describe('Update Manifest and Release Packaging Contracts', () => {
  it('generates update-manifest.json with orbit-update.zip artifact contract', () => {
    const pkg = JSON.parse(fs.readFileSync('package.json', 'utf8'))
    const manifestPath = path.resolve('update-manifest.json')
    assert.ok(fs.existsSync(manifestPath), 'update-manifest.json must exist')

    const manifest = JSON.parse(fs.readFileSync(manifestPath, 'utf8'))
    assert.equal(manifest.version, pkg.version, 'manifest version must match package.json version')
    assert.ok(manifest.url, 'manifest must have url')
    assert.ok(
      manifest.url.endsWith('/orbit-update.zip'),
      `manifest.url must end with orbit-update.zip for backward compatibility, got: ${manifest.url}`
    )
  })

  it('release.yml packages and publishes orbit-update.zip with orbit-win_x64.exe payload', () => {
    const workflowPath = path.resolve('.github/workflows/release.yml')
    assert.ok(fs.existsSync(workflowPath), '.github/workflows/release.yml must exist')

    const content = fs.readFileSync(workflowPath, 'utf8')

    // Must package orbit-update.zip
    assert.ok(
      content.includes('Package Update Zip'),
      'release.yml must contain "Package Update Zip" step'
    )
    assert.ok(
      content.includes('orbit-win_x64.exe'),
      'release.yml must stage orbit-win_x64.exe inside the zip for 0.14.4 compatibility'
    )
    assert.ok(content.includes('orbit-update.zip'), 'release.yml must generate orbit-update.zip')

    // Must upload orbit-update.zip in release assets
    assert.ok(
      content.includes('dist/orbit-update.zip'),
      'release.yml must include dist/orbit-update.zip in release assets'
    )
  })
})
