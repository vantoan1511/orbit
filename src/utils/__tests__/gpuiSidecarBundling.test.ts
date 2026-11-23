import { describe, it } from 'node:test'
import assert from 'node:assert/strict'
import fs from 'node:fs'
import os from 'node:os'
import path from 'node:path'
import {
  GPUI_CARGO_BIN_NAME,
  GPUI_PACKAGE_NAME,
  GPUI_SIDECAR_BASENAME,
  IMAGE_SUBSYSTEM_WINDOWS_CUI,
  IMAGE_SUBSYSTEM_WINDOWS_GUI,
  TAURI_EXTERNAL_BIN_ENTRY,
  computeFileSha256,
  hasCaseInsensitiveFilenameCollision,
  isWindowsTargetTriple,
  parse7zListingHasEntry,
  parseMsiFileTableHasEntry,
  parseNsisScriptHasSidecar,
  parsePeSubsystem,
  readPeSubsystem,
  resolveBuildProfile,
  resolveSidecarPaths,
  resolveTargetTriple
} from '../../../scripts/lib/gpuiSidecar.ts'
import {
  buildCargoArgs,
  shouldBuildFrontendInParallel,
  shouldUseExplicitCargoTarget,
  stageGpuiSidecarFile
} from '../../../scripts/build-gpui-sidecar.ts'
import { verifyStagedSidecarArtifacts } from '../../../scripts/verify-gpui-bundle.ts'

function createMinimalPeBinary(subsystem: number, pe32Plus = true): Buffer {
  const peOffset = 0x80
  const optionalHeaderOffset = peOffset + 24
  const optionalHeaderSize = pe32Plus ? 240 : 224
  const buffer = Buffer.alloc(optionalHeaderOffset + optionalHeaderSize)

  // DOS header magic ("MZ") and e_lfanew pointer to PE signature
  buffer[0] = 0x4d
  buffer[1] = 0x5a
  buffer.writeUInt32LE(peOffset, 0x3c)

  // PE signature ("PE\0\0")
  buffer.write('PE\0\0', peOffset, 'ascii')

  // COFF header SizeOfOptionalHeader (at peOffset + 20)
  buffer.writeUInt16LE(optionalHeaderSize, peOffset + 20)

  // Optional header magic (0x020B = PE32+, 0x010B = PE32) and Subsystem (at +68)
  buffer.writeUInt16LE(pe32Plus ? 0x020b : 0x010b, optionalHeaderOffset)
  buffer.writeUInt16LE(subsystem, optionalHeaderOffset + 68)

  return buffer
}

describe('GPUI Sidecar & Bundle Helper Library (scripts/lib/gpuiSidecar.ts)', () => {
  it('exports expected canonical GPUI sidecar constants', () => {
    assert.equal(GPUI_PACKAGE_NAME, 'orbit-gpui')
    assert.equal(GPUI_CARGO_BIN_NAME, 'orbit')
    assert.equal(GPUI_SIDECAR_BASENAME, 'orbit-gpui')
    assert.equal(TAURI_EXTERNAL_BIN_ENTRY, 'binaries/orbit-gpui')
  })

  it('resolveBuildProfile defaults to release, honors --profile flags, and detects TAURI_ENV_DEBUG', () => {
    assert.equal(resolveBuildProfile([], {}), 'release')
    assert.equal(resolveBuildProfile(['--profile', 'debug'], {}), 'debug')
    assert.equal(resolveBuildProfile(['--profile=debug'], {}), 'debug')
    assert.equal(
      resolveBuildProfile(['--profile', 'release'], { TAURI_ENV_DEBUG: 'true' }),
      'release'
    )
    assert.equal(resolveBuildProfile(['--profile=release'], { TAURI_ENV_DEBUG: 'true' }), 'release')
    assert.equal(resolveBuildProfile([], { TAURI_ENV_DEBUG: 'true' }), 'debug')
    assert.equal(resolveBuildProfile([], { TAURI_ENV_DEBUG: '1' }), 'debug')
    assert.equal(resolveBuildProfile([], { TAURI_ENV_DEBUG: 'false' }), 'release')
    assert.throws(() => resolveBuildProfile(['--profile', 'invalid'], {}), /invalid build profile/i)
    assert.throws(() => resolveBuildProfile(['--profile'], {}), /missing value for --profile/i)
  })

  it('resolveTargetTriple honors --target flags and TAURI_ENV_TARGET_TRIPLE env var', () => {
    assert.equal(
      resolveTargetTriple(['--target', 'x86_64-pc-windows-msvc'], {}),
      'x86_64-pc-windows-msvc'
    )
    assert.equal(resolveTargetTriple(['--target=aarch64-apple-darwin'], {}), 'aarch64-apple-darwin')
    assert.equal(
      resolveTargetTriple([], { TAURI_ENV_TARGET_TRIPLE: 'x86_64-unknown-linux-gnu' }),
      'x86_64-unknown-linux-gnu'
    )
    assert.equal(
      resolveTargetTriple(['--target', 'aarch64-pc-windows-msvc'], {
        TAURI_ENV_TARGET_TRIPLE: 'x86_64-pc-windows-msvc'
      }),
      'aarch64-pc-windows-msvc'
    )
    assert.throws(() => resolveTargetTriple(['--target'], {}), /missing value for --target/i)
  })

  it('isWindowsTargetTriple and resolveSidecarPaths return exact platform paths', () => {
    assert.equal(isWindowsTargetTriple('x86_64-pc-windows-msvc'), true)
    assert.equal(isWindowsTargetTriple('aarch64-pc-windows-msvc'), true)
    assert.equal(isWindowsTargetTriple('aarch64-apple-darwin'), false)
    assert.equal(isWindowsTargetTriple('x86_64-unknown-linux-gnu'), false)

    const root = path.resolve('/workspace/orbit')

    const winPaths = resolveSidecarPaths(root, 'release', 'x86_64-pc-windows-msvc')
    assert.equal(winPaths.cargoBinaryPath, path.join(root, 'target', 'release', 'orbit.exe'))
    assert.equal(
      winPaths.stagedSidecarPath,
      path.join(root, 'src-tauri', 'binaries', 'orbit-gpui-x86_64-pc-windows-msvc.exe')
    )
    assert.equal(winPaths.targetSidecarPath, path.join(root, 'target', 'release', 'orbit-gpui.exe'))
    assert.equal(winPaths.installedBinaryName, 'orbit-gpui.exe')

    const macPaths = resolveSidecarPaths(root, 'debug', 'aarch64-apple-darwin')
    assert.equal(macPaths.cargoBinaryPath, path.join(root, 'target', 'debug', 'orbit'))
    assert.equal(
      macPaths.stagedSidecarPath,
      path.join(root, 'src-tauri', 'binaries', 'orbit-gpui-aarch64-apple-darwin')
    )
    assert.equal(macPaths.targetSidecarPath, path.join(root, 'target', 'debug', 'orbit-gpui'))
    assert.equal(macPaths.installedBinaryName, 'orbit-gpui')

    const linuxPaths = resolveSidecarPaths(root, 'release', 'x86_64-unknown-linux-gnu')
    assert.equal(linuxPaths.cargoBinaryPath, path.join(root, 'target', 'release', 'orbit'))
    assert.equal(
      linuxPaths.stagedSidecarPath,
      path.join(root, 'src-tauri', 'binaries', 'orbit-gpui-x86_64-unknown-linux-gnu')
    )
    assert.equal(linuxPaths.installedBinaryName, 'orbit-gpui')
  })

  it('hasCaseInsensitiveFilenameCollision detects Windows collisions and confirms orbit-gpui.exe is collision-free', () => {
    assert.equal(hasCaseInsensitiveFilenameCollision('orbit.exe', 'Orbit.exe'), true)
    assert.equal(hasCaseInsensitiveFilenameCollision('ORBIT.EXE', 'orbit.exe'), true)
    assert.equal(hasCaseInsensitiveFilenameCollision('orbit-gpui.exe', 'Orbit.exe'), false)
    assert.equal(hasCaseInsensitiveFilenameCollision('orbit-gpui.exe', 'orbit-app.exe'), false)
    assert.equal(hasCaseInsensitiveFilenameCollision('orbit-gpui.exe', 'ORBIT-GPUI.EXE'), true)
  })

  it('parse7zListingHasEntry matches exact filename entries and rejects missing/partial matches', () => {
    const sampleListing = [
      '2026-11-21 12:00:00 ....A     12345678      4567890  orbit-app.exe',
      '2026-11-21 12:00:00 ....A     23456789      8901234  orbit-gpui.exe',
      '2026-11-21 12:00:00 ....A         1024          512  uninstall.exe'
    ].join('\n')

    assert.equal(parse7zListingHasEntry(sampleListing, 'orbit-gpui.exe'), true)
    assert.equal(parse7zListingHasEntry(sampleListing, 'ORBIT-GPUI.EXE'), true)
    assert.equal(parse7zListingHasEntry(sampleListing, 'orbit.exe'), false)

    const partialOnlyListing = [
      '2026-11-21 12:00:00 ....A     12345678      4567890  orbit-gpui.exe.bak',
      '2026-11-21 12:00:00 ....A     12345678      4567890  my-orbit-gpui.exe'
    ].join('\n')

    assert.equal(parse7zListingHasEntry(partialOnlyListing, 'orbit-gpui.exe'), false)
    assert.equal(parse7zListingHasEntry('', 'orbit-gpui.exe'), false)
  })

  it('parseMsiFileTableHasEntry matches plain and WiX short|long FileName entries', () => {
    assert.equal(
      parseMsiFileTableHasEntry(
        ['orbitapp.exe|orbit-app.exe', 'orbitgp.exe|orbit-gpui.exe'],
        'orbit-gpui.exe'
      ),
      true
    )
    assert.equal(
      parseMsiFileTableHasEntry(['orbit-app.exe', 'orbit-gpui.exe'], 'orbit-gpui.exe'),
      true
    )
    assert.equal(parseMsiFileTableHasEntry(['orbitgp.exe|ORBIT-GPUI.EXE'], 'orbit-gpui.exe'), true)
    assert.equal(
      parseMsiFileTableHasEntry(
        ['orbitapp.exe|orbit-app.exe', 'orbitgp.exe|orbit-gpui.exe.bak'],
        'orbit-gpui.exe'
      ),
      false
    )
    assert.equal(parseMsiFileTableHasEntry(['orbit-app.exe'], 'orbit-gpui.exe'), false)
    assert.equal(parseMsiFileTableHasEntry([], 'orbit-gpui.exe'), false)
  })

  it('parseNsisScriptHasSidecar detects bundled sidecar File directive and rejects missing/partial matches', () => {
    const validNsi = `
      Section Install
        File /a "/oname=orbit-gpui.exe" "D:\\Projects\\orbit\\target\\release\\orbit-gpui.exe"
      SectionEnd
    `
    const quotedPlainNsi = `
      Section Install
        File "D:\\Projects\\orbit\\target\\release\\orbit-gpui.exe"
      SectionEnd
    `
    const missingNsi = `
      Section Install
        File "D:\\Projects\\orbit\\target\\release\\orbit-app.exe"
      SectionEnd
    `
    const partialNsi = `
      Section Install
        File "/oname=my-orbit-gpui.exe.bak" "D:\\Projects\\orbit\\target\\release\\my-orbit-gpui.exe.bak"
      SectionEnd
    `

    assert.equal(parseNsisScriptHasSidecar(validNsi, 'orbit-gpui.exe'), true)
    assert.equal(parseNsisScriptHasSidecar(quotedPlainNsi, 'orbit-gpui.exe'), true)
    assert.equal(parseNsisScriptHasSidecar(missingNsi, 'orbit-gpui.exe'), false)
    assert.equal(parseNsisScriptHasSidecar(partialNsi, 'orbit-gpui.exe'), false)
  })

  it('computeFileSha256 computes deterministic lowercase hex SHA-256 digest', () => {
    const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), 'orbit-gpui-sha-'))
    try {
      const filePath = path.join(tmpDir, 'sample.bin')
      fs.writeFileSync(filePath, 'orbit-gpui-test-payload')
      const hash1 = computeFileSha256(filePath)
      const hash2 = computeFileSha256(filePath)
      assert.equal(hash1, hash2)
      assert.match(hash1, /^[a-f0-9]{64}$/)
    } finally {
      fs.rmSync(tmpDir, { recursive: true, force: true })
    }
  })

  it('parsePeSubsystem parses PE32 and PE32+ GUI and CUI subsystems and rejects invalid/truncated buffers', () => {
    assert.equal(
      parsePeSubsystem(createMinimalPeBinary(IMAGE_SUBSYSTEM_WINDOWS_GUI, true)),
      IMAGE_SUBSYSTEM_WINDOWS_GUI
    )
    assert.equal(
      parsePeSubsystem(createMinimalPeBinary(IMAGE_SUBSYSTEM_WINDOWS_CUI, true)),
      IMAGE_SUBSYSTEM_WINDOWS_CUI
    )
    assert.equal(
      parsePeSubsystem(createMinimalPeBinary(IMAGE_SUBSYSTEM_WINDOWS_GUI, false)),
      IMAGE_SUBSYSTEM_WINDOWS_GUI
    )
    assert.equal(
      parsePeSubsystem(createMinimalPeBinary(IMAGE_SUBSYSTEM_WINDOWS_CUI, false)),
      IMAGE_SUBSYSTEM_WINDOWS_CUI
    )

    // Rejects non-PE, empty, truncated, and bad-magic buffers
    assert.equal(parsePeSubsystem(new Uint8Array(0)), null)
    assert.equal(parsePeSubsystem(Buffer.from('not-a-pe-binary')), null)
    assert.equal(
      parsePeSubsystem(createMinimalPeBinary(IMAGE_SUBSYSTEM_WINDOWS_GUI, true).subarray(0, 128)),
      null
    )

    const badMagic = createMinimalPeBinary(IMAGE_SUBSYSTEM_WINDOWS_GUI, true)
    badMagic.writeUInt16LE(0x9999, 0x80 + 24)
    assert.equal(parsePeSubsystem(badMagic), null)

    const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), 'orbit-gpui-pe-'))
    try {
      const pePath = path.join(tmpDir, 'orbit-gpui.exe')
      fs.writeFileSync(pePath, createMinimalPeBinary(IMAGE_SUBSYSTEM_WINDOWS_GUI, true))
      assert.equal(readPeSubsystem(pePath), IMAGE_SUBSYSTEM_WINDOWS_GUI)
      assert.equal(readPeSubsystem(path.join(tmpDir, 'missing.exe')), null)
    } finally {
      fs.rmSync(tmpDir, { recursive: true, force: true })
    }
  })
})

describe('GPUI Sidecar Build & Staging Script (scripts/build-gpui-sidecar.ts)', () => {
  it('buildCargoArgs builds expected cargo arguments for debug, release, and cross-compile targets', () => {
    assert.deepEqual(buildCargoArgs('debug', 'x86_64-pc-windows-msvc', false), [
      'build',
      '-p',
      'orbit-gpui'
    ])
    assert.deepEqual(buildCargoArgs('release', 'x86_64-pc-windows-msvc', false), [
      'build',
      '-p',
      'orbit-gpui',
      '--release'
    ])
    assert.deepEqual(buildCargoArgs('release', 'aarch64-pc-windows-msvc', true), [
      'build',
      '-p',
      'orbit-gpui',
      '--release',
      '--target',
      'aarch64-pc-windows-msvc'
    ])
    assert.deepEqual(buildCargoArgs('debug', 'aarch64-apple-darwin', true), [
      'build',
      '-p',
      'orbit-gpui',
      '--target',
      'aarch64-apple-darwin'
    ])
  })

  it('shouldUseExplicitCargoTarget enables --target for CLI flags and cross-compiling TAURI_ENV_TARGET_TRIPLE', () => {
    const host = 'x86_64-pc-windows-msvc'
    assert.equal(shouldUseExplicitCargoTarget([], {}, host), false)
    assert.equal(shouldUseExplicitCargoTarget([], { TAURI_ENV_TARGET_TRIPLE: host }, host), false)
    assert.equal(
      shouldUseExplicitCargoTarget(
        [],
        { TAURI_ENV_TARGET_TRIPLE: 'aarch64-pc-windows-msvc' },
        host
      ),
      true
    )
    assert.equal(
      shouldUseExplicitCargoTarget(['--target', 'x86_64-pc-windows-msvc'], {}, host),
      true
    )
  })

  it('stageGpuiSidecarFile copies binary to nested directory, verifies SHA-256, and rejects missing or empty files', () => {
    const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), 'orbit-gpui-stage-'))
    try {
      const sourcePath = path.join(tmpDir, 'target', 'release', 'orbit.exe')
      const destPath = path.join(
        tmpDir,
        'src-tauri',
        'binaries',
        'orbit-gpui-x86_64-pc-windows-msvc.exe'
      )
      fs.mkdirSync(path.dirname(sourcePath), { recursive: true })
      fs.writeFileSync(sourcePath, 'MZ-fake-pe-binary-content')

      const result = stageGpuiSidecarFile(sourcePath, destPath)
      assert.ok(fs.existsSync(destPath), 'staged sidecar file must exist')
      assert.equal(result.bytes, fs.statSync(sourcePath).size)
      assert.equal(result.sha256, computeFileSha256(sourcePath))
      assert.equal(result.sha256, computeFileSha256(destPath))

      const missingSource = path.join(tmpDir, 'does-not-exist.exe')
      assert.throws(() => stageGpuiSidecarFile(missingSource, destPath), /not found/i)

      const emptySource = path.join(tmpDir, 'empty.exe')
      fs.writeFileSync(emptySource, '')
      assert.throws(() => stageGpuiSidecarFile(emptySource, destPath), /empty|0 bytes/i)
    } finally {
      fs.rmSync(tmpDir, { recursive: true, force: true })
    }
  })

  it('stageGpuiSidecarFile skips overwriting destination and preserves mtime when size and SHA-256 already match', () => {
    const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), 'orbit-gpui-stage-idempotent-'))
    try {
      const sourcePath = path.join(tmpDir, 'target', 'debug', 'orbit.exe')
      const destPath = path.join(
        tmpDir,
        'src-tauri',
        'binaries',
        'orbit-gpui-x86_64-pc-windows-msvc.exe'
      )
      fs.mkdirSync(path.dirname(sourcePath), { recursive: true })
      fs.writeFileSync(sourcePath, 'MZ-initial-gpui-binary')

      const first = stageGpuiSidecarFile(sourcePath, destPath)
      assert.equal(first.skippedCopy, false)

      // Set an older deterministic mtime on destPath to verify copy is skipped
      const pastDate = new Date('2025-01-01T00:00:00.000Z')
      fs.utimesSync(destPath, pastDate, pastDate)
      const beforeMtimeMs = fs.statSync(destPath).mtimeMs

      const second = stageGpuiSidecarFile(sourcePath, destPath)
      assert.equal(second.skippedCopy, true)
      assert.equal(second.sha256, first.sha256)
      assert.equal(second.bytes, first.bytes)
      assert.equal(fs.statSync(destPath).mtimeMs, beforeMtimeMs)

      // Modify source content -> must overwrite and report skippedCopy === false
      fs.writeFileSync(sourcePath, 'MZ-updated-gpui-binary-payload')
      const third = stageGpuiSidecarFile(sourcePath, destPath)
      assert.equal(third.skippedCopy, false)
      assert.notEqual(third.sha256, first.sha256)
      assert.equal(third.sha256, computeFileSha256(destPath))
    } finally {
      fs.rmSync(tmpDir, { recursive: true, force: true })
    }
  })

  it('shouldBuildFrontendInParallel returns true only when --with-frontend is provided', () => {
    assert.equal(shouldBuildFrontendInParallel([]), false)
    assert.equal(shouldBuildFrontendInParallel(['--profile', 'release']), false)
    assert.equal(shouldBuildFrontendInParallel(['--with-frontend']), true)
    assert.equal(shouldBuildFrontendInParallel(['--profile', 'release', '--with-frontend']), true)
  })
})

describe('GPUI Packaged Bundle Verification Script (scripts/verify-gpui-bundle.ts)', () => {
  it('verifyStagedSidecarArtifacts passes when all three artifacts exist with matching hashes and fails on missing, empty, or stale files', () => {
    const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), 'orbit-gpui-verify-'))
    try {
      const triple = 'x86_64-pc-windows-msvc'
      const paths = resolveSidecarPaths(tmpDir, 'release', triple)
      const payload = createMinimalPeBinary(IMAGE_SUBSYSTEM_WINDOWS_GUI)

      for (const filePath of [
        paths.cargoBinaryPath,
        paths.stagedSidecarPath,
        paths.targetSidecarPath
      ]) {
        fs.mkdirSync(path.dirname(filePath), { recursive: true })
        fs.writeFileSync(filePath, payload)
      }

      const okResult = verifyStagedSidecarArtifacts(tmpDir, 'release', triple)
      assert.equal(okResult.ok, true, okResult.message)
      assert.ok(okResult.details.length >= 4)

      // Stale target sidecar -> hash mismatch
      fs.writeFileSync(paths.targetSidecarPath, 'MZ-stale-gpui-binary')
      const staleResult = verifyStagedSidecarArtifacts(tmpDir, 'release', triple)
      assert.equal(staleResult.ok, false)
      assert.match(staleResult.message, /mismatch|stale/i)

      // Restore target sidecar, make staged sidecar empty
      fs.writeFileSync(paths.targetSidecarPath, payload)
      fs.writeFileSync(paths.stagedSidecarPath, '')
      const emptyResult = verifyStagedSidecarArtifacts(tmpDir, 'release', triple)
      assert.equal(emptyResult.ok, false)
      assert.match(emptyResult.message, /empty|0 bytes/i)

      // Missing cargo binary
      fs.rmSync(paths.cargoBinaryPath)
      const missingResult = verifyStagedSidecarArtifacts(tmpDir, 'release', triple)
      assert.equal(missingResult.ok, false)
      assert.match(missingResult.message, /not found|missing/i)
    } finally {
      fs.rmSync(tmpDir, { recursive: true, force: true })
    }
  })

  it('verifyStagedSidecarArtifacts fails on release Windows binary with console subsystem (CUI) and allows console subsystem for debug builds', () => {
    const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), 'orbit-gpui-subsystem-'))
    try {
      const triple = 'x86_64-pc-windows-msvc'
      const cuiPayload = createMinimalPeBinary(IMAGE_SUBSYSTEM_WINDOWS_CUI)

      const releasePaths = resolveSidecarPaths(tmpDir, 'release', triple)
      for (const filePath of [
        releasePaths.cargoBinaryPath,
        releasePaths.stagedSidecarPath,
        releasePaths.targetSidecarPath
      ]) {
        fs.mkdirSync(path.dirname(filePath), { recursive: true })
        fs.writeFileSync(filePath, cuiPayload)
      }

      const releaseResult = verifyStagedSidecarArtifacts(tmpDir, 'release', triple)
      assert.equal(releaseResult.ok, false)
      assert.match(releaseResult.message, /IMAGE_SUBSYSTEM_WINDOWS_CUI/i)
      assert.match(releaseResult.message, /IMAGE_SUBSYSTEM_WINDOWS_GUI/i)

      const debugPaths = resolveSidecarPaths(tmpDir, 'debug', triple)
      for (const filePath of [
        debugPaths.cargoBinaryPath,
        debugPaths.stagedSidecarPath,
        debugPaths.targetSidecarPath
      ]) {
        fs.mkdirSync(path.dirname(filePath), { recursive: true })
        fs.writeFileSync(filePath, cuiPayload)
      }

      const debugResult = verifyStagedSidecarArtifacts(tmpDir, 'debug', triple)
      assert.equal(debugResult.ok, true, debugResult.message)
    } finally {
      fs.rmSync(tmpDir, { recursive: true, force: true })
    }
  })
})

describe('Tauri & Package Configuration Contracts for GPUI Sidecar', () => {
  it('src-tauri/tauri.conf.json configures bundle.externalBin and beforeBuildCommand for GPUI sidecar', () => {
    const config = JSON.parse(fs.readFileSync(path.resolve('src-tauri/tauri.conf.json'), 'utf8'))

    assert.ok(
      Array.isArray(config.bundle?.externalBin),
      'bundle.externalBin must be an array in src-tauri/tauri.conf.json'
    )
    assert.ok(
      config.bundle.externalBin.includes(TAURI_EXTERNAL_BIN_ENTRY),
      `bundle.externalBin must include '${TAURI_EXTERNAL_BIN_ENTRY}'`
    )
    assert.match(
      String(config.build?.beforeBuildCommand ?? ''),
      /scripts\/build-gpui-sidecar\.ts/,
      'build.beforeBuildCommand must invoke scripts/build-gpui-sidecar.ts'
    )
  })

  it('package.json defines gpui:build:dev, gpui:build:release, and compiles GPUI before tauri:dev', () => {
    const pkg = JSON.parse(fs.readFileSync(path.resolve('package.json'), 'utf8'))
    const scripts = pkg.scripts ?? {}

    assert.equal(
      scripts['gpui:build:dev'],
      'node --experimental-strip-types scripts/build-gpui-sidecar.ts --profile debug'
    )
    assert.equal(
      scripts['gpui:build:release'],
      'node --experimental-strip-types scripts/build-gpui-sidecar.ts --profile release'
    )
    assert.match(
      String(scripts['tauri:dev'] ?? ''),
      /gpui:build:dev.*tauri dev/,
      'tauri:dev must run gpui:build:dev before launching tauri dev'
    )
  })

  it('.gitignore ignores /src-tauri/binaries/ so staged executables are never committed', () => {
    const gitignore = fs.readFileSync(path.resolve('.gitignore'), 'utf8')
    assert.match(gitignore, /^\/src-tauri\/binaries\/?\r?$/m)
  })

  it('src-tauri/windows/nsis/installer.nsi retains {{#each binaries}} in both Install and Uninstall sections', () => {
    const nsi = fs.readFileSync(path.resolve('src-tauri/windows/nsis/installer.nsi'), 'utf8')
    const matches = nsi.match(/\{\{#each\s+binaries\}\}/g) ?? []
    assert.ok(
      matches.length >= 2,
      `installer.nsi must contain {{#each binaries}} in both Install and Uninstall sections (found ${matches.length})`
    )
  })

  it('src-gpui/src/main.rs sets windows_subsystem = "windows" for non-debug builds', () => {
    const mainRs = fs.readFileSync(path.resolve('src-gpui/src/main.rs'), 'utf8')
    assert.match(
      mainRs,
      /^#!\[cfg_attr\(not\(debug_assertions\),\s*windows_subsystem\s*=\s*"windows"\)\]/m,
      'src-gpui/src/main.rs must set #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]'
    )
  })
})

describe('CI & Release Workflow Contracts for GPUI Sidecar', () => {
  it('.github/workflows/ci.yml builds GPUI sidecar before workspace clippy and workspace unit tests', () => {
    const ciContent = fs.readFileSync(path.resolve('.github/workflows/ci.yml'), 'utf8')
    const buildGpuiIdx = ciContent.indexOf('npm run gpui:build:dev')
    const workspaceClippyIdx = ciContent.indexOf(
      'cargo clippy --workspace --all-targets -- -D warnings'
    )
    const workspaceTestIdx = ciContent.indexOf('cargo test --workspace')

    assert.ok(buildGpuiIdx !== -1, 'ci.yml must run npm run gpui:build:dev')
    assert.ok(
      workspaceClippyIdx !== -1,
      'ci.yml must run cargo clippy --workspace --all-targets -- -D warnings'
    )
    assert.ok(workspaceTestIdx !== -1, 'ci.yml must run cargo test --workspace')
    assert.ok(
      buildGpuiIdx < workspaceClippyIdx,
      'ci.yml must run npm run gpui:build:dev before Rust Workspace Clippy'
    )
    assert.ok(
      buildGpuiIdx < workspaceTestIdx,
      'ci.yml must run npm run gpui:build:dev before Rust Workspace Tests'
    )
    assert.ok(
      workspaceClippyIdx < workspaceTestIdx,
      'ci.yml must run Rust Workspace Clippy before Rust Workspace Tests'
    )
  })

  it('.github/workflows/release.yml verifies GPUI sidecar in NSIS and MSI installers before Create Release', () => {
    const releaseContent = fs.readFileSync(path.resolve('.github/workflows/release.yml'), 'utf8')
    const verifyIdx = releaseContent.indexOf('scripts/verify-gpui-bundle.ts')
    const createReleaseIdx = releaseContent.indexOf('softprops/action-gh-release@v2')

    assert.ok(verifyIdx !== -1, 'release.yml must invoke scripts/verify-gpui-bundle.ts')
    assert.ok(createReleaseIdx !== -1, 'release.yml must use softprops/action-gh-release@v2')
    assert.ok(
      verifyIdx < createReleaseIdx,
      'release.yml must verify GPUI bundle BEFORE Create Release'
    )
    assert.match(
      releaseContent,
      /scripts\/verify-gpui-bundle\.ts[^\r\n]*--nsis[^\r\n]*--msi/,
      'release.yml must pass both --nsis and --msi to scripts/verify-gpui-bundle.ts'
    )
    assert.ok(
      releaseContent.includes('MSI_INSTALLER_PATH'),
      'release.yml must locate MSI_INSTALLER_PATH for verification'
    )
  })
})
