const SEMVER_REGEX = /^v?(\d+)\.(\d+)\.(\d+)(?:-([0-9A-Za-z.-]+))?$/
const NUMERIC_ID_REGEX = /^\d+$/

/**
 * Parse a semantic version string (with optional leading `v`).
 * @param {string} versionStr
 */
export function parseSemver(versionStr) {
  if (typeof versionStr !== 'string') return null
  const match = versionStr.trim().match(SEMVER_REGEX)
  if (!match) return null
  return {
    major: Number.parseInt(match[1], 10),
    minor: Number.parseInt(match[2], 10),
    patch: Number.parseInt(match[3], 10),
    prerelease: match[4] || null
  }
}

function comparePrerelease(preA, preB) {
  if (preA === preB) return 0
  if (!preA) return 1
  if (!preB) return -1

  const partsA = preA.split('.')
  const partsB = preB.split('.')
  const len = Math.max(partsA.length, partsB.length)

  for (let i = 0; i < len; i++) {
    const a = partsA[i]
    const b = partsB[i]
    if (a === undefined) return -1
    if (b === undefined) return 1

    const aNum = NUMERIC_ID_REGEX.test(a)
    const bNum = NUMERIC_ID_REGEX.test(b)

    if (aNum && bNum) {
      const diff = Number.parseInt(a, 10) - Number.parseInt(b, 10)
      if (diff !== 0) return diff < 0 ? -1 : 1
    } else if (aNum) {
      // Numeric identifiers always have lower precedence than non-numeric identifiers
      return -1
    } else if (bNum) {
      return 1
    } else if (a !== b) {
      return a < b ? -1 : 1
    }
  }

  return 0
}

/**
 * Compare two semver strings according to SemVer 2.0.0 precedence.
 * @param {string} a
 * @param {string} b
 * @returns {-1 | 0 | 1}
 */
export function compareSemver(a, b) {
  const parsedA = parseSemver(a)
  const parsedB = parseSemver(b)
  if (!parsedA) throw new Error(`Invalid semver string: "${a}"`)
  if (!parsedB) throw new Error(`Invalid semver string: "${b}"`)

  if (parsedA.major !== parsedB.major) return parsedA.major < parsedB.major ? -1 : 1
  if (parsedA.minor !== parsedB.minor) return parsedA.minor < parsedB.minor ? -1 : 1
  if (parsedA.patch !== parsedB.patch) return parsedA.patch < parsedB.patch ? -1 : 1

  return comparePrerelease(parsedA.prerelease, parsedB.prerelease)
}

/**
 * Determine GitHub Release `make_latest` and `prerelease` flags for a target version.
 * @param {string} targetVersion
 * @param {string[]} existingTags
 * @param {'auto' | 'true' | 'false'} [override='auto']
 */
export function computeMakeLatest(targetVersion, existingTags = [], override = 'auto') {
  const parsedTarget = parseSemver(targetVersion)
  if (!parsedTarget) {
    throw new Error(`Invalid target semver version: "${targetVersion}"`)
  }

  const isPrerelease = Boolean(parsedTarget.prerelease)
  if (override === 'true' || override === 'false') {
    return { makeLatest: override, isPrerelease }
  }

  if (isPrerelease) {
    return { makeLatest: 'false', isPrerelease: true }
  }

  for (const tag of existingTags) {
    const parsedTag = parseSemver(tag)
    if (!parsedTag || parsedTag.prerelease) continue
    if (compareSemver(tag, targetVersion) > 0) {
      return { makeLatest: 'false', isPrerelease: false }
    }
  }

  return { makeLatest: 'true', isPrerelease: false }
}

/**
 * Parse CLI arguments for `scripts/release.js`.
 * @param {string[]} args
 */
export function parseReleaseArgs(args) {
  let help = false
  let dryRun = false
  let skipPr = false
  let direct = false
  let tag = false
  let baseBranch = null
  let targetVersionArg = null

  for (let i = 0; i < args.length; i++) {
    const arg = args[i]
    if (arg === '--help' || arg === '-h') {
      help = true
    } else if (arg === '--dry-run') {
      dryRun = true
    } else if (arg === '--no-pr') {
      skipPr = true
    } else if (arg === '--direct') {
      direct = true
    } else if (arg === '--tag') {
      tag = true
    } else if (arg === '--base') {
      const next = args[i + 1]
      if (!next || next.startsWith('-')) {
        throw new Error('Missing value for --base <branch> option.')
      }
      baseBranch = next
      i++
    } else if (arg.startsWith('--base=')) {
      const value = arg.slice('--base='.length).trim()
      if (!value) {
        throw new Error('Missing value for --base=<branch> option.')
      }
      baseBranch = value
    } else if (arg.startsWith('-')) {
      throw new Error(`Unknown option: ${arg}`)
    } else if (!targetVersionArg) {
      targetVersionArg = arg
    } else {
      throw new Error(`Unexpected extra argument: ${arg}`)
    }
  }

  if (tag && !direct && !skipPr) {
    throw new Error('--tag can only be used together with --direct or --no-pr.')
  }

  return {
    help,
    dryRun,
    skipPr,
    direct,
    tag,
    baseBranch,
    targetVersionArg
  }
}

/**
 * Resolve the target base branch for PR mode or validate current branch for --direct mode.
 * @param {{ explicitBase: string | null, currentBranch: string, direct: boolean }} options
 */
export function resolveBaseBranch({ explicitBase, currentBranch, direct }) {
  const trimmedExplicit = typeof explicitBase === 'string' ? explicitBase.trim() : ''
  const trimmedCurrent = typeof currentBranch === 'string' ? currentBranch.trim() : ''

  if (direct) {
    if (!trimmedCurrent) {
      throw new Error(
        'Cannot use --direct from a detached HEAD. Check out a release/* or hotfix/* branch first.'
      )
    }
    if (trimmedCurrent === 'main' || trimmedCurrent === 'master') {
      throw new Error(
        `Refusing --direct release on protected branch "${trimmedCurrent}". Use PR mode or check out a release/* or hotfix/* branch.`
      )
    }
    return trimmedCurrent
  }

  if (trimmedExplicit) {
    return trimmedExplicit
  }

  if (!trimmedCurrent) {
    throw new Error(
      'Cannot determine base branch from detached HEAD. Pass --base <branch> or check out a branch.'
    )
  }

  return trimmedCurrent
}

/**
 * Build the PR preparation branch name so it does not prematurely trigger release workflows.
 * @param {string} targetVersion
 */
export function buildPrBranchName(targetVersion) {
  return `chore/release-v${targetVersion}`
}

/**
 * Build the release preparation PR description targeting `baseBranch`.
 * @param {string} targetVersion
 * @param {string} currentVersion
 * @param {string} baseBranch
 */
export function buildPrBody(targetVersion, currentVersion, baseBranch) {
  return `## Summary

Release preparation for Orbit **v${targetVersion}**.

## Changes

- Bumps version from \`${currentVersion}\` to \`${targetVersion}\` in:
  - \`package.json\`
  - \`src-tauri/tauri.conf.json\`
  - \`src-tauri/Cargo.toml\`
- Synchronizes lockfiles (\`package-lock.json\` and \`Cargo.lock\`).
- Verified TypeScript checks via \`npm run type-check\`.

## Automated Release Process

Upon merging this PR into \`${baseBranch}\`:
1. The GitHub Actions \`Release\` workflow will build Windows desktop binaries and installers (\`.exe\` NSIS & \`.msi\`).
2. Installer elevation level (\`asInvoker\`) will be verified automatically.
3. Git tag \`v${targetVersion}\` and GitHub Release notes will be published (with automatic semver \`make_latest\` gating).
4. Automated WinGet package submission workflow will be dispatched (skipped automatically for prereleases or older hotfix versions).
`
}
