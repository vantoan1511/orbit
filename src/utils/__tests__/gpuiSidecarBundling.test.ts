import { describe, it } from 'node:test'
import assert from 'node:assert/strict'
import fs from 'node:fs'
import os from 'node:os'
import path from 'node:path'
import {
  GPUI_CARGO_BIN_NAME,
  GPUI_PACKAGE_NAME,
  GPUI_SIDECAR_BASENAME,
  TAURI_EXTERNAL_BIN_ENTRY,
  computeFileSha256,
  hasCaseInsensitiveFilenameCollision,
  isWindowsTargetTriple,
  parse7zListingHasEntry,
  parseMsiFileTableHasEntry,
  parseNsisScriptHasSidecar,
  resolveBuildProfile,
  resolveSidecarPaths,
  resolveTargetTriple
} from '../../../scripts/lib/gpuiSidecar.ts'
import {
  buildCargoArgs,
  shouldUseExplicitCargoTarget,
  stageGpuiSidecarFile
} from '../../../scripts/build-gpui-sidecar.ts'
import { verifyStagedSidecarArtifacts } from '../../../scripts/verify-gpui-bundle.ts'

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
})

describe('GPUI Packaged Bundle Verification Script (scripts/verify-gpui-bundle.ts)', () => {
  it('verifyStagedSidecarArtifacts passes when all three artifacts exist with matching hashes and fails on missing, empty, or stale files', () => {
    const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), 'orbit-gpui-verify-'))
    try {
      const triple = 'x86_64-pc-windows-msvc'
      const paths = resolveSidecarPaths(tmpDir, 'release', triple)
      const payload = 'MZ-orbit-gpui-release-binary-v1'

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
      assert.ok(okResult.details.length >= 3)

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
})

describe('CI & Release Workflow Contracts for GPUI Sidecar', () => {
  it('.github/workflows/ci.yml builds GPUI sidecar before backend tests and runs GPUI unit tests', () => {
    const ciContent = fs.readFileSync(path.resolve('.github/workflows/ci.yml'), 'utf8')
    const buildGpuiIdx = ciContent.indexOf('npm run gpui:build:dev')
    const backendTestIdx = ciContent.indexOf('cargo test --manifest-path src-tauri/Cargo.toml')
    const gpuiTestIdx = ciContent.indexOf('cargo test -p orbit-gpui')

    assert.ok(buildGpuiIdx !== -1, 'ci.yml must run npm run gpui:build:dev')
    assert.ok(
      backendTestIdx !== -1,
      'ci.yml must run cargo test --manifest-path src-tauri/Cargo.toml'
    )
    assert.ok(gpuiTestIdx !== -1, 'ci.yml must run cargo test -p orbit-gpui')
    assert.ok(
      buildGpuiIdx < backendTestIdx,
      'ci.yml must run npm run gpui:build:dev before Backend Unit Tests'
    )
    assert.ok(
      backendTestIdx < gpuiTestIdx,
      'ci.yml must run cargo test -p orbit-gpui after Backend Unit Tests'
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
