import test from 'node:test'
import assert from 'node:assert/strict'
import fs from 'node:fs'
import path from 'node:path'
import os from 'node:os'
import { spawnSync } from 'node:child_process'
import {
  parseInstallerExecutionLevel,
  extractManifestXml
} from '../scripts/lib/installerManifest.ts'

test('parseInstallerExecutionLevel parses asInvoker', () => {
  const xml = `
    <assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
      <trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
        <security>
          <requestedPrivileges>
            <requestedExecutionLevel level="asInvoker" uiAccess="false"/>
          </requestedPrivileges>
        </security>
      </trustInfo>
    </assembly>
  `
  assert.equal(parseInstallerExecutionLevel(xml), 'asInvoker')
})

test('parseInstallerExecutionLevel parses highestAvailable', () => {
  const xml = `
    <assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
      <trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
        <security>
          <requestedPrivileges>
            <requestedExecutionLevel level="highestAvailable" uiAccess="false"/>
          </requestedPrivileges>
        </security>
      </trustInfo>
    </assembly>
  `
  assert.equal(parseInstallerExecutionLevel(xml), 'highestAvailable')
})

test('parseInstallerExecutionLevel parses requireAdministrator', () => {
  const xml = `
    <assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
      <trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
        <security>
          <requestedPrivileges>
            <requestedExecutionLevel level="requireAdministrator" uiAccess="false"/>
          </requestedPrivileges>
        </security>
      </trustInfo>
    </assembly>
  `
  assert.equal(parseInstallerExecutionLevel(xml), 'requireAdministrator')
})

test('parseInstallerExecutionLevel returns unknown for missing or garbled manifest', () => {
  assert.equal(parseInstallerExecutionLevel(''), 'unknown')
  assert.equal(parseInstallerExecutionLevel('<assembly></assembly>'), 'unknown')
  assert.equal(
    parseInstallerExecutionLevel('<requestedExecutionLevel level="someOtherValue" />'),
    'unknown'
  )
})

test('extractManifestXml finds manifest embedded in binary noise', () => {
  const manifest = `<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
  <trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
    <security>
      <requestedPrivileges>
        <requestedExecutionLevel level="asInvoker" uiAccess="false"/>
      </requestedPrivileges>
    </security>
  </trustInfo>
</assembly>`

  const prefix = Buffer.from([0x4d, 0x5a, 0x90, 0x00, 0x03, 0x00, 0x00, 0x00])
  const manifestBytes = Buffer.from(manifest, 'utf-8')
  const suffix = Buffer.from([0x00, 0x50, 0x45, 0x00, 0xff, 0xfe, 0xaa, 0x55])

  const exeBuffer = Buffer.concat([prefix, manifestBytes, suffix])
  const extracted = extractManifestXml(new Uint8Array(exeBuffer))

  assert.ok(extracted, 'manifest XML must be extracted from binary')
  assert.match(extracted, /<requestedExecutionLevel level="asInvoker"/)
})

test('extractManifestXml returns null if manifest pattern is not found', () => {
  const randomBytes = new Uint8Array([0x01, 0x02, 0x03, 0x04])
  assert.equal(extractManifestXml(randomBytes), null)
})

const VERIFY_SCRIPT_PATH = path.resolve(
  import.meta.dirname,
  '../scripts/verify-installer-elevation.ts'
)

function createDummyExe(manifestXml: string): Buffer {
  return Buffer.concat([
    Buffer.from([0x4d, 0x5a, 0x90, 0x00]),
    Buffer.from(manifestXml, 'utf-8'),
    Buffer.from([0x00, 0x00])
  ])
}

test('verify script exits zero for asInvoker fixture', () => {
  const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), 'orbit-installer-test-'))
  const fixturePath = path.join(tmpDir, 'orbit_setup_asInvoker.exe')
  try {
    const manifest = `<assembly><requestedPrivileges><requestedExecutionLevel level="asInvoker"/></requestedPrivileges></assembly>`
    fs.writeFileSync(fixturePath, createDummyExe(manifest))

    const result = spawnSync(
      process.execPath,
      ['--experimental-strip-types', VERIFY_SCRIPT_PATH, fixturePath],
      { encoding: 'utf-8' }
    )

    assert.equal(
      result.status,
      0,
      `Expected exit code 0, got ${result.status}. Stderr: ${result.stderr}`
    )
  } finally {
    fs.rmSync(tmpDir, { recursive: true, force: true })
  }
})

test('verify script exits non-zero for highestAvailable fixture', () => {
  const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), 'orbit-installer-test-'))
  const fixturePath = path.join(tmpDir, 'orbit_setup_highestAvailable.exe')
  try {
    const manifest = `<assembly><requestedPrivileges><requestedExecutionLevel level="highestAvailable"/></requestedPrivileges></assembly>`
    fs.writeFileSync(fixturePath, createDummyExe(manifest))

    const result = spawnSync(
      process.execPath,
      ['--experimental-strip-types', VERIFY_SCRIPT_PATH, fixturePath],
      { encoding: 'utf-8' }
    )

    assert.notEqual(result.status, 0, 'Expected non-zero exit code for highestAvailable level')
    assert.match(
      result.stderr,
      /highestAvailable/,
      'Stderr should mention highestAvailable execution level'
    )
    assert.match(
      result.stderr,
      /updater-backward-compat\.md/,
      'Stderr should reference gotchas doc'
    )
  } finally {
    fs.rmSync(tmpDir, { recursive: true, force: true })
  }
})

test('verify script properly handles paths containing spaces without failing open', () => {
  const baseTmpDir = fs.mkdtempSync(path.join(os.tmpdir(), 'orbit spaces test-'))
  const spacedDir = path.join(baseTmpDir, 'sub directory with spaces')
  fs.mkdirSync(spacedDir, { recursive: true })

  try {
    const asInvokerPath = path.join(spacedDir, 'orbit setup spaced asInvoker.exe')
    const highestPath = path.join(spacedDir, 'orbit setup spaced highest.exe')

    fs.writeFileSync(
      asInvokerPath,
      createDummyExe(
        '<assembly><requestedPrivileges><requestedExecutionLevel level="asInvoker"/></requestedPrivileges></assembly>'
      )
    )
    fs.writeFileSync(
      highestPath,
      createDummyExe(
        '<assembly><requestedPrivileges><requestedExecutionLevel level="highestAvailable"/></requestedPrivileges></assembly>'
      )
    )

    // asInvoker in spaced path should pass
    const passResult = spawnSync(
      process.execPath,
      ['--experimental-strip-types', VERIFY_SCRIPT_PATH, asInvokerPath],
      { encoding: 'utf-8' }
    )
    assert.equal(
      passResult.status,
      0,
      `Expected exit code 0 for spaced asInvoker path. Stderr: ${passResult.stderr}`
    )

    // highestAvailable in spaced path must fail (no fail-open)
    const failResult = spawnSync(
      process.execPath,
      ['--experimental-strip-types', VERIFY_SCRIPT_PATH, highestPath],
      { encoding: 'utf-8' }
    )
    assert.notEqual(failResult.status, 0, 'Expected non-zero exit code for spaced highestAvailable')
    assert.match(failResult.stderr, /highestAvailable/)
  } finally {
    fs.rmSync(baseTmpDir, { recursive: true, force: true })
  }
})
