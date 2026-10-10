import { describe, it } from 'node:test'
import assert from 'node:assert/strict'
import fs from 'node:fs'
import path from 'node:path'

describe('Update Manifest and Release Packaging Contracts', () => {
  it('pins update-manifest.json permanently to bridge version 0.14.7', () => {
    const manifestPath = path.resolve('update-manifest.json')
    assert.ok(fs.existsSync(manifestPath), 'update-manifest.json must exist')

    const manifest = JSON.parse(fs.readFileSync(manifestPath, 'utf8'))
    assert.equal(
      manifest.version,
      '0.14.7',
      'update-manifest.json must be permanently pinned to bridge version 0.14.7'
    )
    assert.equal(
      manifest.url,
      'https://github.com/vantoan1511/orbit/releases/download/v0.14.7/orbit-update.zip',
      'manifest url must point to v0.14.7/orbit-update.zip'
    )
  })

  it('release.yml packages and publishes orbit-update.zip only for versions <= 0.14.7 using node semver', () => {
    const workflowPath = path.resolve('.github/workflows/release.yml')
    assert.ok(fs.existsSync(workflowPath), '.github/workflows/release.yml must exist')

    const content = fs.readFileSync(workflowPath, 'utf8')

    // Must package orbit-update.zip conditionally
    assert.ok(
      content.includes('Package Update Zip'),
      'release.yml must contain "Package Update Zip" step'
    )
    assert.ok(
      content.includes('0.14.7'),
      'release.yml must gate legacy packaging to bridge version 0.14.7'
    )
    assert.ok(
      content.includes('semver.lte') || content.includes('s.lte'),
      'release.yml must use semver lte comparison to support prereleases safely'
    )
    assert.ok(
      content.includes('orbit-win_x64.exe'),
      'release.yml must stage orbit-win_x64.exe inside the zip for legacy compatibility'
    )

    // Must upload orbit-update.zip conditionally or via glob
    assert.ok(
      content.includes('dist/*.zip') || content.includes('orbit-update.zip'),
      'release.yml must handle zip asset upload safely'
    )
  })
})
