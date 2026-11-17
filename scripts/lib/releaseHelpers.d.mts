export interface SemverParts {
  major: number
  minor: number
  patch: number
  prerelease: string | null
}

export interface MakeLatestResult {
  makeLatest: 'true' | 'false'
  isPrerelease: boolean
}

export interface ReleaseCliArgs {
  help: boolean
  dryRun: boolean
  skipPr: boolean
  direct: boolean
  tag: boolean
  baseBranch: string | null
  targetVersionArg: string | null
}

export interface ResolveBaseBranchOptions {
  explicitBase: string | null
  currentBranch: string
  direct: boolean
}

export interface CommitLogEntry {
  sha?: string
  subject: string
  author?: string
}

export interface ParsedCommitEntry {
  sha: string
  subject: string
  cleanSubject: string
  author: string
  prNumber: number | null
  category: string
  isReleaseCommit: boolean
}

export interface FallbackReleaseNotesOptions {
  targetTag: string
  previousTag: string | null
  repository: string
  commits: CommitLogEntry[]
  includeReleaseCommit?: boolean
}

export interface ReleaseNotesSelection {
  body: string
  source: 'github' | 'fallback'
}

export const RELEASE_CATEGORY_ORDER: readonly string[]

export function parseSemver(versionStr: string): SemverParts | null
export function compareSemver(a: string, b: string): -1 | 0 | 1
export function computeMakeLatest(
  targetVersion: string,
  existingTags?: string[],
  override?: 'auto' | 'true' | 'false'
): MakeLatestResult
export function parseReleaseArgs(args: string[]): ReleaseCliArgs
export function resolveBaseBranch(options: ResolveBaseBranchOptions): string
export function buildPrBranchName(targetVersion: string): string
export function buildPrBody(
  targetVersion: string,
  currentVersion: string,
  baseBranch: string
): string
export function resolvePreviousTag(
  targetVersion: string,
  mergedTags?: string[],
  allTags?: string[]
): string | null
export function hasWhatsChangedSection(body: string | null | undefined): boolean
export function categorizeCommitPrefix(subject: string): string
export function parseGitLogLines(
  stdout: string
): Array<{ sha: string; author: string; subject: string }>
export function parseCommitEntry(entry: CommitLogEntry): ParsedCommitEntry
export function buildFallbackReleaseNotes(options: FallbackReleaseNotesOptions): string
export function selectReleaseNotes(options: {
  generatedNotes: string | null | undefined
  fallbackNotes: string
}): ReleaseNotesSelection
