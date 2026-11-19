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
  buildPrBody,
  updateCargoManifestVersion,
  resolvePreviousTag,
  hasWhatsChangedSection,
  categorizeCommitPrefix,
  parseGitLogLines,
  parseCommitEntry,
  buildFallbackReleaseNotes,
  selectReleaseNotes
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

  it('updateCargoManifestVersion updates [workspace.package] version in root Cargo.toml and rejects missing version', () => {
    const rootManifest = [
      '[workspace]',
      'members = ["crates/orbit-core", "src-tauri", "src-gpui"]',
      'resolver = "2"',
      '',
      '[workspace.package]',
      'version = "0.19.1"',
      'edition = "2024"',
      '',
      '[workspace.dependencies]',
      'serde = "1.0"',
      ''
    ].join('\n')

    const updated = updateCargoManifestVersion(rootManifest, '0.19.2')
    assert.equal(updated, rootManifest.replace('version = "0.19.1"', 'version = "0.19.2"'))

    const inheritedMemberManifest = [
      '[package]',
      'name = "orbit-app"',
      'version.workspace = true',
      'edition.workspace = true',
      ''
    ].join('\n')

    assert.throws(() => updateCargoManifestVersion(inheritedMemberManifest, '0.19.2'), /version/i)

    const prBody = buildPrBody('0.19.2', '0.19.1', 'main')
    assert.ok(prBody.includes('- `Cargo.toml`'))
    assert.ok(!prBody.includes('`src-tauri/Cargo.toml`'))
  })

  it('root Cargo.toml centralizes [workspace.package], [workspace.dependencies], [workspace.lints.clippy] and clippy.toml allows test unwrap/expect', () => {
    const pkg = JSON.parse(fs.readFileSync(path.resolve('package.json'), 'utf8'))
    const cargoToml = fs.readFileSync(path.resolve('Cargo.toml'), 'utf8')
    const clippyTomlPath = path.resolve('clippy.toml')

    assert.ok(
      cargoToml.includes('[workspace.package]'),
      'Cargo.toml must define [workspace.package]'
    )
    assert.ok(
      cargoToml.includes(`version = "${pkg.version}"`),
      `Cargo.toml [workspace.package] version must match package.json (${pkg.version})`
    )
    assert.ok(
      cargoToml.includes('[workspace.dependencies]'),
      'Cargo.toml must define [workspace.dependencies]'
    )
    assert.ok(
      cargoToml.includes('[workspace.lints.clippy]'),
      'Cargo.toml must define [workspace.lints.clippy]'
    )
    assert.ok(cargoToml.includes('unwrap_used = "deny"'))
    assert.ok(cargoToml.includes('expect_used = "deny"'))

    assert.ok(fs.existsSync(clippyTomlPath), 'clippy.toml must exist at workspace root')
    const clippyToml = fs.readFileSync(clippyTomlPath, 'utf8')
    assert.ok(clippyToml.includes('allow-unwrap-in-tests = true'))
    assert.ok(clippyToml.includes('allow-expect-in-tests = true'))
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

  it('resolvePreviousTag picks highest merged semver tag strictly below targetVersion, skips prereleases for stable targets, and falls back to allTags when mergedTags is empty', () => {
    assert.equal(
      resolvePreviousTag('0.19.2', ['v0.19.0', 'v0.19.1'], ['v0.19.0', 'v0.19.1', 'v0.20.0']),
      'v0.19.1'
    )
    assert.equal(resolvePreviousTag('0.19.2', ['v0.19.1', 'v0.19.2-rc.1'], []), 'v0.19.1')
    assert.equal(resolvePreviousTag('0.20.0-rc.2', ['v0.19.2', 'v0.20.0-rc.1'], []), 'v0.20.0-rc.1')
    assert.equal(resolvePreviousTag('0.19.2', [], ['v0.19.0', 'v0.19.1', 'v0.20.0']), 'v0.19.1')
    assert.equal(resolvePreviousTag('0.1.0', [], []), null)
  })

  it("hasWhatsChangedSection detects populated What's Changed sections and rejects Full Changelog-only bodies", () => {
    assert.equal(
      hasWhatsChangedSection(
        '**Full Changelog**: https://github.com/vantoan1511/orbit/compare/v0.19.1...v0.19.2'
      ),
      false
    )
    assert.equal(
      hasWhatsChangedSection("## What's Changed\n### 🚀 Features\n* feat: foo (#1)\n"),
      true
    )
    assert.equal(
      hasWhatsChangedSection(
        "## What's Changed\n\n**Full Changelog**: https://github.com/vantoan1511/orbit/compare/v0.19.1...v0.19.2"
      ),
      false
    )
    assert.equal(hasWhatsChangedSection(''), false)
    assert.equal(hasWhatsChangedSection(null), false)
  })

  it('buildFallbackReleaseNotes categorizes commits per .github/release.yml, extracts PR links, excludes chore: release v*, and appends Full Changelog', () => {
    assert.equal(categorizeCommitPrefix('feat(gpui): add tabs'), '🚀 Features')
    assert.equal(categorizeCommitPrefix('fix: crash'), '🐛 Bug Fixes')
    assert.equal(categorizeCommitPrefix('docs: update readme'), '📚 Documentation')
    assert.equal(categorizeCommitPrefix('perf: eliminate lag'), '🛠 Maintenance & Refactoring')
    assert.equal(categorizeCommitPrefix('misc: something'), '🧰 Other Changes')

    const parsedLines = parseGitLogLines('383bb41\tNGUYEN VAN TOAN\tperf: eliminate lag (#443)\n')
    assert.deepEqual(parsedLines, [
      { sha: '383bb41', author: 'NGUYEN VAN TOAN', subject: 'perf: eliminate lag (#443)' }
    ])

    const parsedEntry = parseCommitEntry({
      sha: '383bb41abcdef',
      subject: 'perf: eliminate lag (#443)',
      author: 'NGUYEN VAN TOAN'
    })
    assert.equal(parsedEntry.prNumber, 443)
    assert.equal(parsedEntry.cleanSubject, 'perf: eliminate lag')
    assert.equal(parsedEntry.category, '🛠 Maintenance & Refactoring')
    assert.equal(parsedEntry.isReleaseCommit, false)

    const notes = buildFallbackReleaseNotes({
      targetTag: 'v0.19.2',
      previousTag: 'v0.19.1',
      repository: 'vantoan1511/orbit',
      commits: [
        { sha: '383bb41', subject: 'perf: eliminate lag (#443)', author: 'NGUYEN VAN TOAN' },
        { sha: 'e32acd0', subject: 'chore: release v0.19.2', author: 'NGUYEN VAN TOAN' },
        { sha: 'abc1234', subject: 'feat(gpui): add tabs', author: 'Alice' }
      ]
    })

    assert.ok(notes.includes("## What's Changed"))
    assert.ok(notes.includes('### 🚀 Features'))
    assert.ok(notes.includes('* feat(gpui): add tabs by Alice (abc1234)'))
    assert.ok(notes.includes('### 🛠 Maintenance & Refactoring'))
    assert.ok(
      notes.includes(
        '* perf: eliminate lag by NGUYEN VAN TOAN in https://github.com/vantoan1511/orbit/pull/443'
      )
    )
    assert.ok(!notes.includes('chore: release v0.19.2'))
    assert.ok(
      notes.endsWith(
        '**Full Changelog**: https://github.com/vantoan1511/orbit/compare/v0.19.1...v0.19.2'
      )
    )
  })

  it('selectReleaseNotes prefers GitHub generated notes when populated and falls back otherwise', () => {
    const githubNotes =
      "## What's Changed\n* feat: x by @alice in https://github.com/vantoan1511/orbit/pull/1\n"
    const fallbackNotes = "## What's Changed\n### 🚀 Features\n* feat: x by Alice (abc1234)"

    assert.deepEqual(selectReleaseNotes({ generatedNotes: githubNotes, fallbackNotes }), {
      source: 'github',
      body: githubNotes
    })

    const emptyGithubNotes =
      '**Full Changelog**: https://github.com/vantoan1511/orbit/compare/v0.19.1...v0.19.2'
    assert.deepEqual(selectReleaseNotes({ generatedNotes: emptyGithubNotes, fallbackNotes }), {
      source: 'fallback',
      body: `${fallbackNotes}\n`
    })
  })

  it('release.yml generates hybrid release notes with commit-based fallback and passes body_path to action-gh-release@v2', () => {
    const workflowPath = path.resolve('.github/workflows/release.yml')
    const content = fs.readFileSync(workflowPath, 'utf8')

    assert.match(
      content,
      /- name: Generate Release Notes[\s\S]*?if: steps\.meta\.outputs\.should_release == 'true'/
    )
    assert.ok(content.includes('resolvePreviousTag'))
    assert.ok(content.includes('parseGitLogLines'))
    assert.ok(content.includes('buildFallbackReleaseNotes'))
    assert.ok(content.includes('selectReleaseNotes'))
    assert.ok(content.includes('./scripts/lib/releaseHelpers.mjs'))
    assert.ok(content.includes('/releases/generate-notes'))
    assert.ok(content.includes('release_notes_path'))
    assert.ok(content.includes('body_path: ${{ steps.notes.outputs.release_notes_path }}'))
    assert.ok(
      !content.includes('generate_release_notes: true'),
      'release.yml must not leave generate_release_notes: true enabled when using body_path'
    )
  })
})
