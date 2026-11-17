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

export const RELEASE_CATEGORY_ORDER = [
  '🚀 Features',
  '🐛 Bug Fixes',
  '📚 Documentation',
  '🛠 Maintenance & Refactoring',
  '🧰 Other Changes'
]

const CATEGORY_PREFIX_RULES = [
  { category: '🚀 Features', prefixes: ['feat', 'feature', 'enhancement'] },
  { category: '🐛 Bug Fixes', prefixes: ['fix', 'bug', 'bugfix'] },
  { category: '📚 Documentation', prefixes: ['docs', 'doc', 'documentation'] },
  {
    category: '🛠 Maintenance & Refactoring',
    prefixes: ['chore', 'refactor', 'perf', 'ci', 'build', 'test', 'style']
  }
]

function pickHighestTagBelow(targetVersion, targetIsPrerelease, tags) {
  let bestTag = null
  for (const rawTag of tags) {
    const tag = typeof rawTag === 'string' ? rawTag.trim() : ''
    if (!tag) continue
    const parsedTag = parseSemver(tag)
    if (!parsedTag) continue
    if (!targetIsPrerelease && parsedTag.prerelease) continue
    if (compareSemver(tag, targetVersion) >= 0) continue
    if (!bestTag || compareSemver(tag, bestTag) > 0) {
      bestTag = tag
    }
  }
  return bestTag
}

/**
 * Resolve the highest semver tag strictly below `targetVersion`, preferring merged tags.
 * @param {string} targetVersion
 * @param {string[]} [mergedTags=[]]
 * @param {string[]} [allTags=[]]
 * @returns {string | null}
 */
export function resolvePreviousTag(targetVersion, mergedTags = [], allTags = []) {
  const parsedTarget = parseSemver(targetVersion)
  if (!parsedTarget) {
    throw new Error(`Invalid target semver version: "${targetVersion}"`)
  }
  const targetIsPrerelease = Boolean(parsedTarget.prerelease)
  return (
    pickHighestTagBelow(targetVersion, targetIsPrerelease, mergedTags) ??
    pickHighestTagBelow(targetVersion, targetIsPrerelease, allTags)
  )
}

/**
 * Return true when `body` contains a populated `## What's Changed` section.
 * @param {string | null | undefined} body
 * @returns {boolean}
 */
export function hasWhatsChangedSection(body) {
  if (typeof body !== 'string' || !body.trim()) return false
  const lines = body.split(/\r?\n/)
  const headingIdx = lines.findIndex((line) => /^##\s+What's Changed\s*$/i.test(line.trim()))
  if (headingIdx === -1) return false

  for (let i = headingIdx + 1; i < lines.length; i++) {
    const trimmed = lines[i].trim()
    if (!trimmed) continue
    if (/^\*\*Full Changelog\*\*:/i.test(trimmed)) break
    if (/^[*-]\s+\S/.test(trimmed)) return true
  }
  return false
}

/**
 * Map a conventional commit subject prefix to a `.github/release.yml` category title.
 * @param {string} subject
 * @returns {string}
 */
export function categorizeCommitPrefix(subject) {
  if (typeof subject !== 'string') return '🧰 Other Changes'
  const match = subject.trim().match(/^([a-zA-Z]+)(?:\([^)]*\))?!?:\s*/)
  if (!match) return '🧰 Other Changes'

  const type = match[1].toLowerCase()
  for (const rule of CATEGORY_PREFIX_RULES) {
    if (rule.prefixes.some((prefix) => type === prefix || type.startsWith(prefix))) {
      return rule.category
    }
  }
  return '🧰 Other Changes'
}

/**
 * Parse `git log --format=%H%x09%an%x09%s` output lines into commit log entries.
 * @param {string} stdout
 * @returns {Array<{ sha: string, author: string, subject: string }>}
 */
export function parseGitLogLines(stdout) {
  if (typeof stdout !== 'string' || !stdout.trim()) return []
  const entries = []
  for (const rawLine of stdout.split(/\r?\n/)) {
    const line = rawLine.trim()
    if (!line) continue
    const [sha = '', author = '', ...subjectParts] = line.split('\t')
    const subject = subjectParts.join('\t').trim()
    if (!subject) continue
    entries.push({
      sha: sha.trim(),
      author: author.trim(),
      subject
    })
  }
  return entries
}

/**
 * Parse a raw commit log entry into structured release note metadata.
 * @param {{ sha?: string, subject: string, author?: string }} entry
 */
export function parseCommitEntry(entry) {
  const sha = typeof entry.sha === 'string' ? entry.sha.trim() : ''
  const subject = typeof entry.subject === 'string' ? entry.subject.trim() : ''
  const author =
    typeof entry.author === 'string' && entry.author.trim() ? entry.author.trim() : 'unknown'

  const prMatch = subject.match(/\s*\(#(\d+)\)\s*$/)
  const prNumber = prMatch ? Number.parseInt(prMatch[1], 10) : null
  const cleanSubject = prMatch ? subject.slice(0, prMatch.index).trim() : subject
  const category = categorizeCommitPrefix(cleanSubject)
  const isReleaseCommit = /^chore(?:\([^)]*\))?:\s*release\s+v?\d+\.\d+\.\d+/i.test(cleanSubject)

  return {
    sha,
    subject,
    cleanSubject,
    author,
    prNumber,
    category,
    isReleaseCommit
  }
}

/**
 * Build markdown release notes from commit history when GitHub's auto-generated notes are empty.
 * @param {{
 *   targetTag: string,
 *   previousTag: string | null,
 *   repository: string,
 *   commits: Array<{ sha?: string, subject: string, author?: string }>,
 *   includeReleaseCommit?: boolean
 * }} options
 * @returns {string}
 */
export function buildFallbackReleaseNotes({
  targetTag,
  previousTag,
  repository,
  commits,
  includeReleaseCommit = false
}) {
  const grouped = new Map(RELEASE_CATEGORY_ORDER.map((category) => [category, []]))

  for (const rawCommit of commits || []) {
    const parsed = parseCommitEntry(rawCommit)
    if (!parsed.cleanSubject) continue
    if (!includeReleaseCommit && parsed.isReleaseCommit) continue

    const bucket = grouped.get(parsed.category) ?? grouped.get('🧰 Other Changes')
    const shortSha = parsed.sha ? parsed.sha.slice(0, 7) : ''
    const line = parsed.prNumber
      ? `* ${parsed.cleanSubject} by ${parsed.author} in https://github.com/${repository}/pull/${parsed.prNumber}`
      : shortSha
        ? `* ${parsed.cleanSubject} by ${parsed.author} (${shortSha})`
        : `* ${parsed.cleanSubject} by ${parsed.author}`
    bucket.push(line)
  }

  const sections = ["## What's Changed"]
  let hasEntries = false

  for (const category of RELEASE_CATEGORY_ORDER) {
    const items = grouped.get(category)
    if (!items || items.length === 0) continue
    hasEntries = true
    sections.push(`### ${category}`)
    sections.push(...items)
  }

  if (!hasEntries) {
    sections.push('* No notable changes recorded.')
  }

  if (previousTag && repository) {
    sections.push('')
    sections.push(
      `**Full Changelog**: https://github.com/${repository}/compare/${previousTag}...${targetTag}`
    )
  }

  return sections.join('\n')
}

/**
 * Choose GitHub-generated release notes when populated, otherwise return commit-based fallback notes.
 * @param {{ generatedNotes: string | null | undefined, fallbackNotes: string }} options
 * @returns {{ body: string, source: 'github' | 'fallback' }}
 */
export function selectReleaseNotes({ generatedNotes, fallbackNotes }) {
  if (hasWhatsChangedSection(generatedNotes)) {
    return {
      source: 'github',
      body: generatedNotes
    }
  }

  const normalizedFallback = fallbackNotes.endsWith('\n') ? fallbackNotes : `${fallbackNotes}\n`
  return {
    source: 'fallback',
    body: normalizedFallback
  }
}
