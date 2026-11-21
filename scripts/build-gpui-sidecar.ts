import { spawnSync } from 'node:child_process'
import fs from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'
import {
  GPUI_CARGO_BIN_NAME,
  GPUI_PACKAGE_NAME,
  computeFileSha256,
  detectHostTargetTriple,
  isWindowsTargetTriple,
  resolveBuildProfile,
  resolveExplicitTargetTriple,
  resolveSidecarPaths,
  resolveTargetTriple,
  type BuildProfile
} from './lib/gpuiSidecar.ts'

export interface StageSidecarResult {
  sourcePath: string
  destPath: string
  bytes: number
  sha256: string
}

export function buildCargoArgs(
  profile: BuildProfile,
  targetTriple: string,
  explicitTarget: boolean
): string[] {
  const args = ['build', '-p', GPUI_PACKAGE_NAME]
  if (profile === 'release') {
    args.push('--release')
  }
  if (explicitTarget) {
    args.push('--target', targetTriple)
  }
  return args
}

export function stageGpuiSidecarFile(sourcePath: string, destPath: string): StageSidecarResult {
  if (!fs.existsSync(sourcePath)) {
    throw new Error(`Compiled GPUI binary not found at: ${sourcePath}`)
  }

  const sourceStat = fs.statSync(sourcePath)
  if (!sourceStat.isFile()) {
    throw new Error(`Compiled GPUI binary path is not a regular file: ${sourcePath}`)
  }
  if (sourceStat.size === 0) {
    throw new Error(`Compiled GPUI binary is empty (0 bytes): ${sourcePath}`)
  }

  fs.mkdirSync(path.dirname(destPath), { recursive: true })
  fs.copyFileSync(sourcePath, destPath)

  if (process.platform !== 'win32') {
    fs.chmodSync(destPath, sourceStat.mode | 0o111)
  }

  const destStat = fs.statSync(destPath)
  const sourceHash = computeFileSha256(sourcePath)
  const destHash = computeFileSha256(destPath)

  if (destStat.size !== sourceStat.size || destHash !== sourceHash) {
    throw new Error(
      `Staged GPUI sidecar integrity mismatch between ${sourcePath} (${sourceStat.size} bytes, ${sourceHash}) and ${destPath} (${destStat.size} bytes, ${destHash})`
    )
  }

  return {
    sourcePath,
    destPath,
    bytes: destStat.size,
    sha256: destHash
  }
}

export function shouldUseExplicitCargoTarget(
  args: readonly string[],
  env: NodeJS.ProcessEnv,
  hostTriple: string
): boolean {
  if (args.some((arg) => arg === '--target' || arg.startsWith('--target='))) {
    return true
  }
  const explicitFromEnv = resolveExplicitTargetTriple(args, env)
  return explicitFromEnv !== null && explicitFromEnv !== hostTriple
}

function runCli(): void {
  const args = process.argv.slice(2)
  const workspaceRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..')
  const profile = resolveBuildProfile(args, process.env)
  const hostTriple = detectHostTargetTriple()
  const explicitTarget = shouldUseExplicitCargoTarget(args, process.env, hostTriple)
  const targetTriple = resolveTargetTriple(args, process.env)
  const paths = resolveSidecarPaths(workspaceRoot, profile, targetTriple)

  const ext = isWindowsTargetTriple(targetTriple) ? '.exe' : ''
  const cargoOutputPath = explicitTarget
    ? path.join(workspaceRoot, 'target', targetTriple, profile, `${GPUI_CARGO_BIN_NAME}${ext}`)
    : paths.cargoBinaryPath

  const cargoArgs = buildCargoArgs(profile, targetTriple, explicitTarget)
  const explicitTargetNote = resolveExplicitTargetTriple(args, process.env)
    ? ` (target: ${targetTriple})`
    : ` (host: ${targetTriple})`

  console.log(`[gpui-sidecar] Compiling ${GPUI_PACKAGE_NAME} [${profile}]${explicitTargetNote}...`)
  console.log(`[gpui-sidecar] Running: cargo ${cargoArgs.join(' ')}`)

  const result = spawnSync('cargo', cargoArgs, {
    cwd: workspaceRoot,
    stdio: 'inherit'
  })

  if (result.error) {
    console.error(`[ERROR] Failed to execute cargo: ${result.error.message}`)
    process.exit(1)
  }
  if (result.status !== 0) {
    console.error(`[ERROR] cargo build failed with exit code ${result.status ?? 1}`)
    process.exit(result.status ?? 1)
  }

  const staged = stageGpuiSidecarFile(cargoOutputPath, paths.stagedSidecarPath)
  console.log(
    `[PASS] Staged GPUI sidecar -> ${path.relative(workspaceRoot, staged.destPath)} (${staged.bytes} bytes, sha256=${staged.sha256.slice(0, 12)}...)`
  )
}

const isMain =
  process.argv[1] && path.resolve(process.argv[1]) === path.resolve(fileURLToPath(import.meta.url))

if (isMain) {
  runCli()
}
