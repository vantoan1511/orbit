import { describe, it } from 'node:test'
import assert from 'node:assert/strict'
import fs from 'node:fs'
import path from 'node:path'
import {
  parseSemver,
  compareSemver,
  computeMakeLatest,
  parseReleaseArgs,
  resolveBaseBranch,
  buildPrBranchName,
  buildPrBody
} from '../../../scripts/lib/releaseHelpers.mjs'

describe('Shared Release Helpers & Semver Contracts', () => {
  it('parseSemver and compareSemver handle standard, v-prefixed, and prerelease versions', () => {
    assert.deepEqual(parseSemver('0.18.3'), {
      major: 0,
      minor: 18,
      patch: 3,
      prerelease: null
    })
    assert.deepEqual(parseSemver('v0.20.0-rc.1'), {
      major: 0,
      minor: 20,
      patch: 0,
      prerelease: 'rc.1'
    })
    assert.equal(parseSemver('not-a-version'), null)

    assert.equal(compareSemver('v0.18.3', 'v0.19.1'), -1)
    assert.equal(compareSemver('v0.19.1', '0.19.1'), 0)
    assert.equal(compareSemver('0.20.0', '0.20.0-beta.1'), 1)
    assert.equal(compareSemver('0.20.0-alpha.2', '0.20.0-alpha.10'), -1)
    assert.equal(compareSemver('0.20.0-alpha.1', '0.20.0-alpha.beta'), -1)
  })

  it('computeMakeLatest returns false for older hotfix patches and prereleases, and honors explicit overrides', () => {
    assert.deepEqual(computeMakeLatest('0.18.3', ['v0.19.1', 'v0.18.2']), {
      makeLatest: 'false',
      isPrerelease: false
    })
    assert.deepEqual(computeMakeLatest('0.19.2', ['v0.19.1', 'v0.20.0-beta.1']), {
      makeLatest: 'true',
      isPrerelease: false
    })
    assert.deepEqual(computeMakeLatest('0.20.0-rc.1', ['v0.19.1']), {
      makeLatest: 'false',
      isPrerelease: true
    })
    assert.deepEqual(computeMakeLatest('0.18.3', ['v0.19.1'], 'true'), {
      makeLatest: 'true',
      isPrerelease: false
    })
    assert.deepEqual(computeMakeLatest('0.19.2', ['v0.19.1'], 'false'), {
      makeLatest: 'false',
      isPrerelease: false
    })
  })

  it('parseReleaseArgs and resolveBaseBranch support current-branch default, --base override, --direct, and reject unsafe combinations', () => {
    assert.deepEqual(parseReleaseArgs(['0.18.3', '--base', 'hotfix/v0.18.2']), {
      help: false,
      dryRun: false,
      skipPr: false,
      direct: false,
      tag: false,
      baseBranch: 'hotfix/v0.18.2',
      targetVersionArg: '0.18.3'
    })
    assert.deepEqual(parseReleaseArgs(['0.18.3', '--base=release/v0.18', '--dry-run']), {
      help: false,
      dryRun: true,
      skipPr: false,
      direct: false,
      tag: false,
      baseBranch: 'release/v0.18',
      targetVersionArg: '0.18.3'
    })
    assert.deepEqual(parseReleaseArgs(['0.18.3', '--direct', '--tag']), {
      help: false,
      dryRun: false,
      skipPr: false,
      direct: true,
      tag: true,
      baseBranch: null,
      targetVersionArg: '0.18.3'
    })

    assert.throws(() => parseReleaseArgs(['0.18.3', '--tag']), /--tag.*--direct|--no-pr/i)
    assert.throws(() => parseReleaseArgs(['--base']), /--base/i)

    assert.equal(
      resolveBaseBranch({ explicitBase: null, currentBranch: 'release/v0.18', direct: false }),
      'release/v0.18'
    )
    assert.equal(
      resolveBaseBranch({
        explicitBase: 'hotfix/v0.18.2',
        currentBranch: 'main',
        direct: false
      }),
      'hotfix/v0.18.2'
    )
    assert.throws(
      () => resolveBaseBranch({ explicitBase: null, currentBranch: '', direct: false }),
      /detached HEAD|base branch/i
    )
    assert.throws(
      () => resolveBaseBranch({ explicitBase: null, currentBranch: 'main', direct: true }),
      /main|master/i
    )
    assert.throws(
      () => resolveBaseBranch({ explicitBase: null, currentBranch: 'master', direct: true }),
      /main|master/i
    )

    assert.equal(buildPrBranchName('0.18.3'), 'chore/release-v0.18.3')
    assert.ok(buildPrBody('0.18.3', '0.18.2', 'hotfix/v0.18.2').includes('hotfix/v0.18.2'))
  })

  it('scripts/release.js uses releaseHelpers, supports --base/--direct/--tag, and does not hardcode --base main', () => {
    const releasePath = path.resolve('scripts/release.js')
    const content = fs.readFileSync(releasePath, 'utf8')

    assert.ok(!content.includes('--base main'), 'scripts/release.js must not hardcode --base main')
    assert.ok(
      content.includes('./lib/releaseHelpers.mjs'),
      'scripts/release.js must import ./lib/releaseHelpers.mjs'
    )
    assert.ok(content.includes('--base'), 'scripts/release.js must document/support --base')
    assert.ok(content.includes('--direct'), 'scripts/release.js must document/support --direct')
    assert.ok(content.includes('--tag'), 'scripts/release.js must document/support --tag')
    assert.ok(
      content.includes("'checkout', '-b', branchName, baseBranch"),
      'scripts/release.js must branch from baseBranch when explicit --base differs from currentBranch'
    )
  })

  it('buildPrBody targets the provided baseBranch instead of hardcoding main', () => {
    const body = buildPrBody('0.18.3', '0.18.2', 'hotfix/v0.18.2')
    assert.ok(body.includes('Upon merging this PR into `hotfix/v0.18.2`:'))
    assert.ok(!body.includes('into `main`'))
  })

  it('release.yml supports release/**, hotfix/**, v* tags, existing-tag skip guard, and action-gh-release@v2 make_latest', () => {
    const workflowPath = path.resolve('.github/workflows/release.yml')
    const content = fs.readFileSync(workflowPath, 'utf8')

    assert.match(
      content,
      /branches:\s*\n\s*-\s*main\s*\n\s*-\s*['"]release\/\*\*['"]\s*\n\s*-\s*['"]hotfix\/\*\*['"]/
    )
    assert.match(content, /tags:\s*\n\s*-\s*['"]v\*['"]/)
    assert.ok(content.includes('Determine Release Metadata & Eligibility'))
    assert.ok(content.includes('scripts/lib/releaseHelpers.mjs'))
    assert.ok(content.includes("steps.meta.outputs.should_release == 'true'"))
    assert.ok(content.includes('softprops/action-gh-release@v2'))
    assert.ok(content.includes('make_latest:'))
    assert.ok(content.includes('prerelease:'))
    assert.ok(content.includes('INPUT_MAKE_LATEST: ${{ inputs.make_latest }}'))
  })

  it('winget.yml resolves exact tag from workflow_run head_sha and skips older/prerelease versions unless manually specified', () => {
    const workflowPath = path.resolve('.github/workflows/winget.yml')
    const content = fs.readFileSync(workflowPath, 'utf8')

    assert.ok(content.includes('github.event.workflow_run.head_sha'))
    assert.ok(content.includes('/contents/package.json?ref='))
    assert.ok(content.includes('/releases/tags/'))
    assert.ok(content.includes('should_publish'))
    assert.ok(content.includes("steps.resolve-tag.outputs.should_publish == 'true'"))
    assert.ok(content.includes('INPUT_TAG: ${{ inputs.tag }}'))
  })
})
