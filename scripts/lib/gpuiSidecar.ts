import { execFileSync } from 'node:child_process'
import crypto from 'node:crypto'
import fs from 'node:fs'
import path from 'node:path'

export type BuildProfile = 'debug' | 'release'

export interface SidecarPaths {
  /** Workspace Cargo output path: target/{debug|release}/orbit[.exe] */
  cargoBinaryPath: string
  /** Tauri externalBin staged path: src-tauri/binaries/orbit-gpui-{targetTriple}[.exe] */
  stagedSidecarPath: string
  /** Tauri build-script copied path: target/{debug|release}/orbit-gpui[.exe] */
  targetSidecarPath: string
  /** Final installed executable filename inside NSIS/MSI: orbit-gpui.exe | orbit-gpui */
  installedBinaryName: string
}

export interface BundleVerificationResult {
  ok: boolean
  message: string
  details: string[]
}

export const GPUI_PACKAGE_NAME = 'orbit-gpui' as const
export const GPUI_CARGO_BIN_NAME = 'orbit' as const
export const GPUI_SIDECAR_BASENAME = 'orbit-gpui' as const
export const TAURI_EXTERNAL_BIN_ENTRY = 'binaries/orbit-gpui' as const

export function parseFlagValue(args: readonly string[], flagName: string): string | null {
  const prefix = `${flagName}=`
  for (let i = 0; i < args.length; i += 1) {
    const arg = args[i]
    if (arg === flagName) {
      const next = args[i + 1]
      if (!next || next.startsWith('--')) {
        throw new Error(`Missing value for ${flagName}`)
      }
      return next.trim()
    }
    if (arg && arg.startsWith(prefix)) {
      const value = arg.slice(prefix.length).trim()
      if (!value) {
        throw new Error(`Missing value for ${flagName}`)
      }
      return value
    }
  }
  return null
}

export function resolveBuildProfile(
  args: readonly string[],
  env: NodeJS.ProcessEnv = process.env
): BuildProfile {
  const explicit = parseFlagValue(args, '--profile')
  if (explicit !== null) {
    if (explicit === 'debug' || explicit === 'release') {
      return explicit
    }
    throw new Error(`Invalid build profile '${explicit}'. Expected 'debug' or 'release'.`)
  }

  const tauriDebug = (env.TAURI_ENV_DEBUG ?? '').trim().toLowerCase()
  if (tauriDebug === 'true' || tauriDebug === '1') {
    return 'debug'
  }

  return 'release'
}

export function parseRustcHostTriple(rustcVerboseVersionOutput: string): string | null {
  for (const line of rustcVerboseVersionOutput.split(/\r?\n/)) {
    const trimmed = line.trim()
    if (trimmed.startsWith('host:')) {
      const triple = trimmed.slice('host:'.length).trim()
      if (triple.length > 0) {
        return triple
      }
    }
  }
  return null
}

export function detectHostTargetTriple(): string {
  const output = execFileSync('rustc', ['-vV'], { encoding: 'utf8' })
  const triple = parseRustcHostTriple(output)
  if (!triple) {
    throw new Error('Failed to parse host target triple from `rustc -vV` output.')
  }
  return triple
}

export function resolveExplicitTargetTriple(
  args: readonly string[],
  env: NodeJS.ProcessEnv = process.env
): string | null {
  const fromArgs = parseFlagValue(args, '--target')
  if (fromArgs) {
    return fromArgs
  }

  const fromEnv = (env.TAURI_ENV_TARGET_TRIPLE ?? '').trim()
  return fromEnv.length > 0 ? fromEnv : null
}

export function resolveTargetTriple(
  args: readonly string[],
  env: NodeJS.ProcessEnv = process.env
): string {
  return resolveExplicitTargetTriple(args, env) ?? detectHostTargetTriple()
}

export function isWindowsTargetTriple(targetTriple: string): boolean {
  return targetTriple.toLowerCase().includes('windows')
}

export function resolveSidecarPaths(
  workspaceRoot: string,
  profile: BuildProfile,
  targetTriple: string
): SidecarPaths {
  const ext = isWindowsTargetTriple(targetTriple) ? '.exe' : ''
  const cargoBinaryName = `${GPUI_CARGO_BIN_NAME}${ext}`
  const installedBinaryName = `${GPUI_SIDECAR_BASENAME}${ext}`
  const stagedSidecarName = `${GPUI_SIDECAR_BASENAME}-${targetTriple}${ext}`

  return {
    cargoBinaryPath: path.join(workspaceRoot, 'target', profile, cargoBinaryName),
    stagedSidecarPath: path.join(workspaceRoot, 'src-tauri', 'binaries', stagedSidecarName),
    targetSidecarPath: path.join(workspaceRoot, 'target', profile, installedBinaryName),
    installedBinaryName
  }
}

export function hasCaseInsensitiveFilenameCollision(nameA: string, nameB: string): boolean {
  return path.basename(nameA).toLowerCase() === path.basename(nameB).toLowerCase()
}

export function parse7zListingHasEntry(listingOutput: string, expectedFileName: string): boolean {
  const target = expectedFileName.trim().toLowerCase()
  if (!target) {
    return false
  }

  for (const rawLine of listingOutput.split(/\r?\n/)) {
    const line = rawLine.trim()
    if (!line) {
      continue
    }
    const tokens = line.split(/\s+/)
    const lastToken = tokens[tokens.length - 1]
    if (!lastToken) {
      continue
    }
    const normalized = lastToken.replace(/\\/g, '/')
    const entryName = path.posix.basename(normalized).toLowerCase()
    if (entryName === target) {
      return true
    }
  }

  return false
}

export function parseMsiFileTableHasEntry(
  fileNames: readonly string[],
  expectedFileName: string
): boolean {
  const target = expectedFileName.trim().toLowerCase()
  if (!target) {
    return false
  }

  for (const entry of fileNames) {
    const parts = entry
      .split('|')
      .map((part) => part.trim().toLowerCase())
      .filter((part) => part.length > 0)
    if (parts.includes(target)) {
      return true
    }
  }

  return false
}

function escapeRegExp(value: string): string {
  return value.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
}

export function parseNsisScriptHasSidecar(nsisContent: string, expectedFileName: string): boolean {
  const target = expectedFileName.trim()
  if (!target) {
    return false
  }

  const escaped = escapeRegExp(target)
  const onamePattern = new RegExp(`["']/oname=${escaped}["']`, 'i')
  if (onamePattern.test(nsisContent)) {
    return true
  }

  const fileDirectivePattern = new RegExp(
    `^\\s*File\\b[^\\r\\n]*(?:[\\\\/"'])${escaped}["']?\\s*$`,
    'im'
  )
  return fileDirectivePattern.test(nsisContent)
}

export function computeFileSha256(filePath: string): string {
  const hash = crypto.createHash('sha256')
  const data = fs.readFileSync(filePath)
  hash.update(data)
  return hash.digest('hex')
}
