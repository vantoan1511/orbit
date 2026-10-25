import fs from 'node:fs'
import path from 'node:path'
import {
  extractManifestXml,
  parseInstallerExecutionLevel,
  type InstallerExecutionLevel
} from './lib/installerManifest.ts'

const GOTCHAS_DOC = '.agents/references/gotchas/updater-backward-compat.md'

export interface VerificationResult {
  ok: boolean
  level: InstallerExecutionLevel
  message: string
}

export function verifyInstallerFile(installerPath: string): VerificationResult {
  if (!fs.existsSync(installerPath)) {
    return {
      ok: false,
      level: 'unknown',
      message: `Installer file not found at: ${installerPath}`
    }
  }

  const fileBytes = fs.readFileSync(installerPath)
  const manifestXml = extractManifestXml(fileBytes)

  if (!manifestXml) {
    return {
      ok: false,
      level: 'unknown',
      message: `Failed to find embedded application manifest in ${installerPath}. Ensure the binary is a valid Windows PE executable.`
    }
  }

  const level = parseInstallerExecutionLevel(manifestXml)

  if (level !== 'asInvoker') {
    return {
      ok: false,
      level,
      message:
        `Forbidden installer requestedExecutionLevel '${level}' in ${installerPath}.\n` +
        `The NSIS installer must have requestedExecutionLevel 'asInvoker' (configured via bundle.windows.nsis.installMode: "currentUser").\n` +
        `Legacy Orbit clients (<= 0.17.0) invoke updates via CreateProcess which cannot handle elevation prompts (fails with OS error 740).\n` +
        `See: ${GOTCHAS_DOC}`
    }
  }

  return {
    ok: true,
    level,
    message: `Verified installer ${path.basename(installerPath)} has requestedExecutionLevel 'asInvoker' (elevation-free).`
  }
}

function runCli(): void {
  const args = process.argv.slice(2)
  if (args.length === 0) {
    console.error(
      'Usage: node --experimental-strip-types scripts/verify-installer-elevation.ts <path-to-setup.exe>'
    )
    process.exit(1)
  }

  const installerPath = path.resolve(args[0])
  const result = verifyInstallerFile(installerPath)

  if (!result.ok) {
    console.error(`[ERROR] Verification failed: ${result.message}`)
    process.exit(1)
  }

  console.log(`[PASS] ${result.message}`)
  process.exit(0)
}

// Execute when invoked directly from CLI
if (
  process.argv[1] &&
  path.resolve(process.argv[1]) ===
    path.resolve(new URL(import.meta.url).pathname.replace(/^\/([A-Za-z]:)/, '$1'))
) {
  runCli()
}
