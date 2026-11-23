import { describe, it } from 'node:test'
import assert from 'node:assert/strict'
import fs from 'node:fs'
import path from 'node:path'
import viteConfig from '../../../vite.config.ts'

describe('Rust Build Profile & Crate Type Contracts', () => {
  it('Cargo.toml configures lean dev debuginfo while preserving release ThinLTO', () => {
    const cargoToml = fs.readFileSync(path.resolve('Cargo.toml'), 'utf8')

    assert.match(
      cargoToml,
      /\[profile\.dev\][\s\S]*?debug\s*=\s*"line-tables-only"/,
      'Cargo.toml [profile.dev] must configure debug = "line-tables-only"'
    )
    assert.match(
      cargoToml,
      /\[profile\.dev\.package\."\*\"\][\s\S]*?debug\s*=\s*0/,
      'Cargo.toml [profile.dev.package."*"] must configure debug = 0'
    )
    assert.match(
      cargoToml,
      /\[profile\.release\][\s\S]*?lto\s*=\s*"thin"/,
      'Cargo.toml [profile.release] must retain lto = "thin"'
    )
  })

  it('src-tauri/Cargo.toml builds only rlib crate-type for desktop', () => {
    const tauriCargoToml = fs.readFileSync(path.resolve('src-tauri/Cargo.toml'), 'utf8')

    assert.match(
      tauriCargoToml,
      /\[lib\][\s\S]*?crate-type\s*=\s*\[\s*"rlib"\s*\]/,
      'src-tauri/Cargo.toml [lib] must set crate-type = ["rlib"]'
    )
    assert.doesNotMatch(
      tauriCargoToml,
      /staticlib|cdylib/,
      'src-tauri/Cargo.toml must not build unused mobile staticlib or cdylib crate types'
    )
  })
})

function flattenPluginNames(plugins: unknown): string[] {
  if (!Array.isArray(plugins)) return []
  const names: string[] = []
  for (const entry of plugins) {
    if (Array.isArray(entry)) {
      names.push(...flattenPluginNames(entry))
    } else if (entry && typeof entry === 'object' && 'name' in entry) {
      const name = (entry as { name?: unknown }).name
      if (typeof name === 'string') {
        names.push(name)
      }
    }
  }
  return names
}

describe('Tauri, Vite & ESLint Build Optimization Contracts', () => {
  it('src-tauri/tauri.conf.json runs beforeBuildCommand with --with-frontend and retains lzma compression', () => {
    const tauriConf = JSON.parse(fs.readFileSync(path.resolve('src-tauri/tauri.conf.json'), 'utf8'))

    assert.equal(
      tauriConf.build?.beforeBuildCommand,
      'node --experimental-strip-types scripts/build-gpui-sidecar.ts --with-frontend'
    )
    assert.equal(tauriConf.bundle?.windows?.nsis?.compression, 'lzma')
  })

  it('vite.config.ts conditionally enables vueDevTools only in serve mode', () => {
    assert.equal(
      typeof viteConfig,
      'function',
      'vite.config.ts must export a defineConfig(({ command }) => ...) function'
    )

    const buildConfig = viteConfig({ command: 'build', mode: 'production' })
    const serveConfig = viteConfig({ command: 'serve', mode: 'development' })

    const buildPlugins = flattenPluginNames(buildConfig.plugins)
    const servePlugins = flattenPluginNames(serveConfig.plugins)

    assert.equal(
      buildPlugins.some((name) => name.toLowerCase().includes('devtools')),
      false,
      `vueDevTools must be excluded during build, found: ${buildPlugins.join(', ')}`
    )
    assert.equal(
      servePlugins.some((name) => name.toLowerCase().includes('devtools')),
      true,
      `vueDevTools must be included during serve, found: ${servePlugins.join(', ')}`
    )
  })

  it('package.json enables ESLint --cache on lint and defines lint:check without --fix', () => {
    const pkg = JSON.parse(fs.readFileSync(path.resolve('package.json'), 'utf8'))
    const scripts = pkg.scripts ?? {}

    assert.match(String(scripts.lint ?? ''), /--cache/, 'scripts.lint must include --cache')
    assert.match(
      String(scripts['lint:check'] ?? ''),
      /--cache/,
      'scripts["lint:check"] must include --cache'
    )
    assert.match(
      String(scripts['lint:check'] ?? ''),
      /--max-warnings\s+0/,
      'scripts["lint:check"] must include --max-warnings 0'
    )
    assert.doesNotMatch(
      String(scripts['lint:check'] ?? ''),
      /--fix/,
      'scripts["lint:check"] must not pass --fix'
    )
  })
})

describe('GitHub Actions CI & Release Workflow Optimization Contracts', () => {
  it('.github/workflows/ci.yml splits frontend (ubuntu-latest) and rust (windows-latest) into parallel jobs', () => {
    const ciYml = fs.readFileSync(path.resolve('.github/workflows/ci.yml'), 'utf8')

    const frontendIdx = ciYml.indexOf('\n  frontend:')
    const rustIdx = ciYml.indexOf('\n  rust:')

    assert.ok(frontendIdx !== -1, 'ci.yml must define a frontend job')
    assert.ok(rustIdx !== -1, 'ci.yml must define a rust job')

    const frontendBlock =
      frontendIdx < rustIdx ? ciYml.slice(frontendIdx, rustIdx) : ciYml.slice(frontendIdx)
    const rustBlock =
      rustIdx > frontendIdx ? ciYml.slice(rustIdx) : ciYml.slice(rustIdx, frontendIdx)

    assert.match(
      frontendBlock,
      /runs-on:\s*ubuntu-latest/,
      'frontend job must run on ubuntu-latest'
    )
    assert.match(frontendBlock, /npm run lint:check/, 'frontend job must run npm run lint:check')
    assert.doesNotMatch(frontendBlock, /needs:/, 'frontend job must run in parallel without needs:')

    assert.match(rustBlock, /runs-on:\s*windows-latest/, 'rust job must run on windows-latest')
    assert.doesNotMatch(rustBlock, /needs:/, 'rust job must run in parallel without needs:')
    assert.doesNotMatch(rustBlock, /\bnpm ci\b/, 'rust job must not run npm ci')
    assert.match(
      rustBlock,
      /mozilla-actions\/sccache-action/,
      'rust job must install sccache via mozilla-actions/sccache-action'
    )
    assert.match(rustBlock, /RUSTC_WRAPPER:\s*sccache/, 'rust job must set RUSTC_WRAPPER: sccache')
    assert.match(
      rustBlock,
      /SCCACHE_GHA_ENABLED:\s*['"]?true['"]?/,
      'rust job must enable SCCACHE_GHA_ENABLED'
    )
    assert.match(
      rustBlock,
      /CARGO_INCREMENTAL:\s*['"]?0['"]?/,
      'rust job must disable CARGO_INCREMENTAL for sccache'
    )
    assert.match(
      rustBlock,
      /lld-link\.exe/,
      'rust job must detect and conditionally configure lld-link.exe'
    )
    assert.match(
      rustBlock,
      /CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_LINKER/,
      'rust job must set CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_LINKER when lld-link is available'
    )
  })

  it('.github/workflows/release.yml configures sccache and conditional lld-link before tauri:build', () => {
    const releaseYml = fs.readFileSync(path.resolve('.github/workflows/release.yml'), 'utf8')

    const sccacheActionIdx = releaseYml.indexOf('mozilla-actions/sccache-action')
    const lldIdx = releaseYml.indexOf('lld-link.exe')
    const tauriBuildIdx = releaseYml.indexOf('npm run tauri:build')

    assert.ok(sccacheActionIdx !== -1, 'release.yml must use mozilla-actions/sccache-action')
    assert.ok(lldIdx !== -1, 'release.yml must detect lld-link.exe')
    assert.ok(tauriBuildIdx !== -1, 'release.yml must run npm run tauri:build')
    assert.ok(
      sccacheActionIdx < tauriBuildIdx,
      'release.yml must configure sccache before npm run tauri:build'
    )
    assert.ok(
      lldIdx < tauriBuildIdx,
      'release.yml must configure conditional lld-link before npm run tauri:build'
    )
    assert.match(
      releaseYml,
      /RUSTC_WRAPPER:\s*sccache/,
      'release.yml must set RUSTC_WRAPPER: sccache'
    )
    assert.match(
      releaseYml,
      /SCCACHE_GHA_ENABLED:\s*['"]?true['"]?/,
      'release.yml must enable SCCACHE_GHA_ENABLED'
    )
    assert.match(
      releaseYml,
      /CARGO_INCREMENTAL:\s*['"]?0['"]?/,
      'release.yml must disable CARGO_INCREMENTAL for sccache'
    )
  })
})
