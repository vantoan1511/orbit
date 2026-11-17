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
