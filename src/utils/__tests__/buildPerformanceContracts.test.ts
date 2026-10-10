import { describe, it } from 'node:test'
import assert from 'node:assert/strict'
import fs from 'node:fs'
import path from 'node:path'

describe('Build Performance & Release Packaging Contracts', () => {
  it('enforces codegen-units = 16 in root Cargo.toml release profile', () => {
    const cargoTomlPath = path.resolve('Cargo.toml')
    assert.ok(fs.existsSync(cargoTomlPath), 'Cargo.toml must exist')

    const content = fs.readFileSync(cargoTomlPath, 'utf8')
    assert.match(
      content,
      /\[profile\.release\][\s\S]*?\bcodegen-units\s*=\s*16\b/,
      'Cargo.toml [profile.release] must configure codegen-units = 16 for parallel compilation'
    )
    assert.match(
      content,
      /\[profile\.release\][\s\S]*?\blto\s*=\s*"thin"/,
      'Cargo.toml [profile.release] must retain lto = "thin"'
    )
  })

  it('enforces swatinem/rust-cache@v2 in release.yml', () => {
    const workflowPath = path.resolve('.github/workflows/release.yml')
    assert.ok(fs.existsSync(workflowPath), '.github/workflows/release.yml must exist')

    const content = fs.readFileSync(workflowPath, 'utf8')
    assert.ok(
      content.includes('swatinem/rust-cache@v2'),
      'release.yml must include swatinem/rust-cache@v2 to cache dependencies across releases'
    )
  })

  it('enforces npm cache and npm ci in release.yml', () => {
    const workflowPath = path.resolve('.github/workflows/release.yml')
    const content = fs.readFileSync(workflowPath, 'utf8')

    assert.match(
      content,
      /actions\/setup-node@v4[\s\S]*?cache:\s*['"]npm['"]/,
      'release.yml actions/setup-node@v4 must specify cache: npm'
    )
    assert.match(
      content,
      /run:\s*npm\s+ci\b/,
      'release.yml must run npm ci instead of npm install for clean deterministic installs'
    )
  })

  it('enforces resilient NSIS installer matching (*setup.exe) in release.yml', () => {
    const workflowPath = path.resolve('.github/workflows/release.yml')
    const content = fs.readFileSync(workflowPath, 'utf8')

    // In Package Update Zip:
    assert.match(
      content,
      /target\/release\/bundle\/nsis\/\*setup\.exe/,
      'Package Update Zip must search for *setup.exe to match Tauri output (e.g. Orbit_0.14.7_x64-setup.exe)'
    )
    assert.ok(
      !content.includes('target/release/bundle/nsis/*_setup.exe'),
      'release.yml must not use restrictive *_setup.exe pattern which misses *-setup.exe'
    )

    // In Create Release files list:
    assert.match(
      content,
      /files:\s*\|[\s\S]*?target\/release\/bundle\/nsis\/\*setup\.exe/,
      'Create Release files list must include target/release/bundle/nsis/*setup.exe'
    )
  })

  it('verifies *setup.exe glob pattern matches both hyphen and underscore naming schemes', () => {
    const nsisGlobRegex = /^.*setup\.exe$/i
    const hyphenFilename = 'Orbit_0.14.7_x64-setup.exe'
    const underscoreFilename = 'Orbit_0.14.7_x64_setup.exe'
    const oldUnderscoreGlobRegex = /^.*_setup\.exe$/i

    // The old pattern fails on Tauri's actual output
    assert.equal(
      oldUnderscoreGlobRegex.test(hyphenFilename),
      false,
      'Old pattern *_setup.exe fails on hyphenated setup filename'
    )

    // The new pattern succeeds on both
    assert.equal(
      nsisGlobRegex.test(hyphenFilename),
      true,
      '*setup.exe must match hyphenated setup filename'
    )
    assert.equal(
      nsisGlobRegex.test(underscoreFilename),
      true,
      '*setup.exe must match underscore setup filename'
    )
  })
})
