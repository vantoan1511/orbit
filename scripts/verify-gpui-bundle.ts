import { spawnSync } from 'node:child_process'
import fs from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'
import {
  IMAGE_SUBSYSTEM_WINDOWS_CUI,
  IMAGE_SUBSYSTEM_WINDOWS_GUI,
  computeFileSha256,
  isWindowsTargetTriple,
  parse7zListingHasEntry,
  parseFlagValue,
  parseMsiFileTableHasEntry,
  parseNsisScriptHasSidecar,
  readPeSubsystem,
  resolveBuildProfile,
  resolveSidecarPaths,
  resolveTargetTriple,
  type BuildProfile,
  type BundleVerificationResult
} from './lib/gpuiSidecar.ts'

export interface VerifyPackagedBundleOptions {
  workspaceRoot: string
  profile: BuildProfile
  targetTriple: string
  nsisPath?: string | null
  msiPath?: string | null
  nsisScriptPath?: string | null
}

function validateNonEmptyFile(
  filePath: string,
  label: string
): { ok: true; size: number; sha256: string } | { ok: false; error: string } {
  if (!fs.existsSync(filePath)) {
    return { ok: false, error: `${label} not found at: ${filePath}` }
  }
  const stat = fs.statSync(filePath)
  if (!stat.isFile()) {
    return { ok: false, error: `${label} is not a regular file: ${filePath}` }
  }
  if (stat.size === 0) {
    return { ok: false, error: `${label} is empty (0 bytes): ${filePath}` }
  }
  return { ok: true, size: stat.size, sha256: computeFileSha256(filePath) }
}

function verifyMatchingSidecarCopy(
  filePath: string,
  label: string,
  detailLabel: string,
  expectedSha256: string,
  cargoBinaryPath: string,
  details: string[]
): string | null {
  const check = validateNonEmptyFile(filePath, label)
  if (!check.ok) {
    return check.error
  }
  if (check.sha256 !== expectedSha256) {
    return `${label} hash mismatch (stale binary): ${filePath} (${check.sha256}) != ${cargoBinaryPath} (${expectedSha256})`
  }
  details.push(`${detailLabel}: ${filePath} (${check.size} bytes, hash matches)`)
  return null
}

export function verifyStagedSidecarArtifacts(
  workspaceRoot: string,
  profile: BuildProfile,
  targetTriple: string
): BundleVerificationResult {
  const paths = resolveSidecarPaths(workspaceRoot, profile, targetTriple)
  const details: string[] = []

  const cargoCheck = validateNonEmptyFile(paths.cargoBinaryPath, 'Compiled GPUI binary')
  if (!cargoCheck.ok) {
    return { ok: false, message: cargoCheck.error, details }
  }
  details.push(
    `Compiled GPUI binary: ${paths.cargoBinaryPath} (${cargoCheck.size} bytes, sha256=${cargoCheck.sha256.slice(0, 12)}...)`
  )

  const copies: Array<[string, string, string]> = [
    [paths.stagedSidecarPath, 'Staged Tauri externalBin sidecar', 'Staged externalBin sidecar'],
    [paths.targetSidecarPath, 'Tauri target sidecar copy', 'Tauri target sidecar copy']
  ]
  for (const [copyPath, label, detailLabel] of copies) {
    const err = verifyMatchingSidecarCopy(
      copyPath,
      label,
      detailLabel,
      cargoCheck.sha256,
      paths.cargoBinaryPath,
      details
    )
    if (err) {
      return { ok: false, message: err, details }
    }
  }

  if (profile === 'release' && isWindowsTargetTriple(targetTriple)) {
    const subsystem = readPeSubsystem(paths.cargoBinaryPath)
    if (subsystem !== IMAGE_SUBSYSTEM_WINDOWS_GUI) {
      const actualLabel =
        subsystem === IMAGE_SUBSYSTEM_WINDOWS_CUI
          ? `IMAGE_SUBSYSTEM_WINDOWS_CUI (${subsystem})`
          : subsystem === null
            ? 'invalid or unreadable PE header'
            : `unexpected subsystem (${subsystem})`
      return {
        ok: false,
        message: `Compiled GPUI release binary ${paths.cargoBinaryPath} has ${actualLabel}; expected IMAGE_SUBSYSTEM_WINDOWS_GUI (${IMAGE_SUBSYSTEM_WINDOWS_GUI}) so Windows does not open an extra console window.`,
        details
      }
    }
    details.push(
      `PE subsystem verified: IMAGE_SUBSYSTEM_WINDOWS_GUI (${IMAGE_SUBSYSTEM_WINDOWS_GUI})`
    )
  }

  return {
    ok: true,
    message: `Verified GPUI sidecar artifacts for ${profile} (${targetTriple}).`,
    details
  }
}

function find7zExecutable(): string | null {
  const candidates =
    process.platform === 'win32'
      ? ['7z', 'C:\\Program Files\\7-Zip\\7z.exe', 'C:\\Program Files (x86)\\7-Zip\\7z.exe']
      : ['7z', '7zz']

  for (const candidate of candidates) {
    if (candidate.includes(path.sep) && !fs.existsSync(candidate)) {
      continue
    }
    const probe = spawnSync(candidate, ['i'], { encoding: 'utf8' })
    if (!probe.error && probe.status === 0) {
      return candidate
    }
  }

  return null
}

function findRenderedNsisScript(workspaceRoot: string, profile: BuildProfile): string | null {
  const nsisRoot = path.join(workspaceRoot, 'target', profile, 'nsis')
  if (!fs.existsSync(nsisRoot)) {
    return null
  }
  for (const entry of fs.readdirSync(nsisRoot, { withFileTypes: true })) {
    if (!entry.isDirectory()) {
      continue
    }
    const candidate = path.join(nsisRoot, entry.name, 'installer.nsi')
    if (fs.existsSync(candidate)) {
      return candidate
    }
  }
  return null
}

function verifyNsisInstaller(
  nsisPath: string,
  expectedFileName: string,
  nsisScriptPath: string | null,
  details: string[]
): string | null {
  if (!fs.existsSync(nsisPath)) {
    return `NSIS installer not found at: ${nsisPath}`
  }

  let verifiedViaRenderedScript = false
  if (nsisScriptPath && fs.existsSync(nsisScriptPath)) {
    const content = fs.readFileSync(nsisScriptPath, 'utf8')
    if (!parseNsisScriptHasSidecar(content, expectedFileName)) {
      return `Rendered NSIS script ${nsisScriptPath} does not include sidecar '${expectedFileName}'.`
    }
    details.push(`Rendered NSIS script references '${expectedFileName}': ${nsisScriptPath}`)
    verifiedViaRenderedScript = true
  }

  const sevenZip = find7zExecutable()
  if (!sevenZip) {
    if (verifiedViaRenderedScript && process.env.GITHUB_ACTIONS !== 'true') {
      details.push(
        `NSIS archive inspection skipped locally (7-Zip not installed); verified via rendered installer.nsi (${nsisPath})`
      )
      return null
    }
    return '7-Zip (`7z`) is required to inspect NSIS installer contents but was not found on PATH or in Program Files.'
  }

  const listing = spawnSync(sevenZip, ['l', '-ba', nsisPath], { encoding: 'utf8' })
  if (listing.error || listing.status !== 0) {
    return `Failed to inspect NSIS archive ${nsisPath} with ${sevenZip}: ${listing.stderr || listing.error?.message || `exit ${listing.status}`}`
  }

  if (!parse7zListingHasEntry(listing.stdout, expectedFileName)) {
    return `NSIS installer ${path.basename(nsisPath)} does not contain required GPUI sidecar '${expectedFileName}'.`
  }
  details.push(`NSIS archive contains '${expectedFileName}': ${nsisPath}`)

  return null
}

function verifyMsiInstaller(
  msiPath: string,
  expectedFileName: string,
  details: string[]
): string | null {
  if (!fs.existsSync(msiPath)) {
    return `MSI installer not found at: ${msiPath}`
  }

  if (process.platform !== 'win32') {
    return `MSI File table inspection via WindowsInstaller.Installer COM requires Windows (current platform: ${process.platform}).`
  }

  const psScript = [
    '$ErrorActionPreference = "Stop"',
    '$installer = New-Object -ComObject WindowsInstaller.Installer',
    '$db = $installer.GetType().InvokeMember("OpenDatabase", "InvokeMethod", $null, $installer, @($env:ORBIT_VERIFY_MSI_PATH, 0))',
    '$view = $db.GetType().InvokeMember("OpenView", "InvokeMethod", $null, $db, @("SELECT FileName FROM File"))',
    '$view.GetType().InvokeMember("Execute", "InvokeMethod", $null, $view, $null) | Out-Null',
    'while ($true) {',
    '  $record = $view.GetType().InvokeMember("Fetch", "InvokeMethod", $null, $view, $null)',
    '  if ($null -eq $record) { break }',
    '  $name = $record.GetType().InvokeMember("StringData", "GetProperty", $null, $record, @(1))',
    '  Write-Output $name',
    '}',
    '$view.GetType().InvokeMember("Close", "InvokeMethod", $null, $view, $null) | Out-Null'
  ].join('\n')

  const result = spawnSync('powershell', ['-NoProfile', '-NonInteractive', '-Command', psScript], {
    encoding: 'utf8',
    env: {
      ...process.env,
      ORBIT_VERIFY_MSI_PATH: path.resolve(msiPath)
    }
  })

  if (result.error || result.status !== 0) {
    return `Failed to query MSI File table for ${msiPath}: ${result.stderr || result.error?.message || `exit ${result.status}`}`
  }

  const fileNames = result.stdout
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter((line) => line.length > 0)

  if (!parseMsiFileTableHasEntry(fileNames, expectedFileName)) {
    return `MSI installer ${path.basename(msiPath)} File table does not contain required GPUI sidecar '${expectedFileName}'.`
  }
  details.push(`MSI File table contains '${expectedFileName}': ${msiPath}`)

  return null
}

export function verifyPackagedBundle(
  options: VerifyPackagedBundleOptions
): BundleVerificationResult {
  const artifactsResult = verifyStagedSidecarArtifacts(
    options.workspaceRoot,
    options.profile,
    options.targetTriple
  )
  if (!artifactsResult.ok) {
    return artifactsResult
  }

  const paths = resolveSidecarPaths(options.workspaceRoot, options.profile, options.targetTriple)
  const details = [...artifactsResult.details]

  if (options.nsisPath) {
    const resolvedScript =
      options.nsisScriptPath ?? findRenderedNsisScript(options.workspaceRoot, options.profile)
    const nsisErr = verifyNsisInstaller(
      path.resolve(options.nsisPath),
      paths.installedBinaryName,
      resolvedScript,
      details
    )
    if (nsisErr) {
      return { ok: false, message: nsisErr, details }
    }
  }

  if (options.msiPath) {
    const msiErr = verifyMsiInstaller(
      path.resolve(options.msiPath),
      paths.installedBinaryName,
      details
    )
    if (msiErr) {
      return { ok: false, message: msiErr, details }
    }
  }

  return {
    ok: true,
    message: `Verified GPUI sidecar '${paths.installedBinaryName}' across build artifacts and packaged installers.`,
    details
  }
}

function runCli(): void {
  const args = process.argv.slice(2)
  const workspaceRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..')
  const profile = resolveBuildProfile(args, process.env)
  const targetTriple = resolveTargetTriple(args, process.env)
  const nsisPath = parseFlagValue(args, '--nsis')
  const msiPath = parseFlagValue(args, '--msi')
  const nsisScriptPath = parseFlagValue(args, '--nsis-script')

  const result = verifyPackagedBundle({
    workspaceRoot,
    profile,
    targetTriple,
    nsisPath,
    msiPath,
    nsisScriptPath
  })

  for (const detail of result.details) {
    console.log(`  - ${detail}`)
  }

  if (!result.ok) {
    console.error(`[ERROR] GPUI bundle verification failed: ${result.message}`)
    process.exit(1)
  }

  console.log(`[PASS] ${result.message}`)
  process.exit(0)
}

const isMain =
  process.argv[1] && path.resolve(process.argv[1]) === path.resolve(fileURLToPath(import.meta.url))

if (isMain) {
  runCli()
}
