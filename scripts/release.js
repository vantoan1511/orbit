import { readFileSync, writeFileSync } from 'fs'
import { execSync } from 'child_process'

function run(command, options = {}) {
  console.log(`> ${command}`)
  return execSync(command, { stdio: 'inherit', ...options })
}

function updateFile(filePath, updater) {
  const content = readFileSync(filePath, 'utf8')
  const newContent = updater(content)
  if (content === newContent) {
    console.log(`No changes for ${filePath}`)
  } else {
    writeFileSync(filePath, newContent, 'utf8')
    console.log(`Updated ${filePath}`)
  }
}

function updateCargoVersion(filePath, newVersion) {
  updateFile(filePath, (content) => {
    return content.replace(
      /(\[package\][\s\S]*?\bversion\s*=\s*")([^"]+)(")/,
      `$1${newVersion}$3`
    )
  })
}

function parseSemver(versionStr) {
  const match = versionStr.match(/^(\d+)\.(\d+)\.(\d+)(?:-([0-9A-Za-z.-]+))?$/)
  if (!match) return null
  return {
    major: parseInt(match[1], 10),
    minor: parseInt(match[2], 10),
    patch: parseInt(match[3], 10),
    prerelease: match[4] || null
  }
}

function buildPrBody(targetVersion, currentVersion) {
  return `## Summary

Release preparation for Orbit **v${targetVersion}**.

## Changes

- Bumps version from \`${currentVersion}\` to \`${targetVersion}\` in:
  - \`neutralino.config.json\`
  - \`package.json\`
  - \`core/engine/Cargo.toml\`
  - \`core/updater/Cargo.toml\`
- Synchronizes lockfiles (\`package-lock.json\` and \`core/Cargo.lock\`).
- Verified TypeScript checks via \`npm run type-check\`.

## Automated Release Process

Upon merging this PR into \`main\`:
1. The GitHub Actions \`Release\` workflow will build desktop binaries and packages.
2. An update zip package and installer executable will be generated.
3. \`update-manifest.json\` will be updated and committed.
4. Git tag \`v${targetVersion}\` and GitHub Release notes will be published.
5. Automated WinGet package submission workflow will be dispatched.
`
}

function main() {
  const args = process.argv.slice(2)
  if (args.includes('--help') || args.includes('-h')) {
    console.log(`
Usage: node scripts/release.js [version] [options]

Arguments:
  [version]       Target semver version (e.g. 0.10.0 or v0.10.0). If omitted, bumps current patch.

Options:
  --dry-run       Preview changes and commands without committing, pushing, or creating PR.
  --no-pr         Skip creating a GitHub Pull Request.
  -h, --help      Display this help message.
`)
    process.exit(0)
  }

  const isDryRun = args.includes('--dry-run')
  const skipPr = args.includes('--no-pr')
  const versionArg = args.find((arg) => !arg.startsWith('--') && !arg.startsWith('-'))
  let targetVersion = versionArg

  const packageJson = JSON.parse(readFileSync('package.json', 'utf8'))
  const currentVersion = packageJson.version

  if (!targetVersion) {
    const semver = parseSemver(currentVersion)
    if (!semver) {
      console.error(`Current version (${currentVersion}) is not standard semver. Please provide a version: npm run release <version>`)
      process.exit(1)
    }
    targetVersion = `${semver.major}.${semver.minor}.${semver.patch + 1}`
    console.log(`No version specified. Auto-bumping patch version from ${currentVersion} to ${targetVersion}`)
  }

  // Remove leading 'v' if provided
  if (targetVersion.startsWith('v')) {
    targetVersion = targetVersion.slice(1)
  }

  if (!parseSemver(targetVersion)) {
    console.error(`Error: Invalid semver version string: "${targetVersion}"`)
    process.exit(1)
  }

  console.log(`\n=== Preparing Orbit Release v${targetVersion}${isDryRun ? ' (DRY RUN)' : ''} ===\n`)

  const branchName = `release/v${targetVersion}`
  const commitMsg = `chore: release v${targetVersion}`
  const prBody = buildPrBody(targetVersion, currentVersion)

  // Ensure git working tree is clean
  const statusOutput = execSync('git status --porcelain', { encoding: 'utf8' }).trim()
  if (statusOutput.length > 0) {
    console.error('Error: Git working tree has uncommitted changes. Please stash or commit them before releasing.')
    process.exit(1)
  }

  console.log(`1. Checking out branch ${branchName}...`)
  try {
    run(`git checkout -b ${branchName}`)
  } catch {
    console.log(`Branch ${branchName} might already exist, checking out...`)
    run(`git checkout ${branchName}`)
  }

  console.log(`\n2. Updating version numbers to ${targetVersion}...`)

  // 1. package.json
  updateFile('package.json', (content) => {
    const pkg = JSON.parse(content)
    pkg.version = targetVersion
    return JSON.stringify(pkg, null, 2) + '\n'
  })

  // 2. neutralino.config.json
  updateFile('neutralino.config.json', (content) => {
    const config = JSON.parse(content)
    config.version = targetVersion
    return JSON.stringify(config, null, 2) + '\n'
  })

  // 3. core/engine/Cargo.toml
  updateCargoVersion('core/engine/Cargo.toml', targetVersion)

  // 4. core/updater/Cargo.toml
  updateCargoVersion('core/updater/Cargo.toml', targetVersion)

  console.log('\n3. Updating lock files...')
  run('npm install')
  run('cargo check', { cwd: 'core' })

  console.log('\n4. Running typecheck...')
  run('npm run type-check')

  if (isDryRun) {
    console.log('\n[DRY RUN] Skipping git commit, push, and PR creation.')
    console.log(`[DRY RUN] Would execute:`)
    console.log(`  git add package.json package-lock.json neutralino.config.json core/engine/Cargo.toml core/updater/Cargo.toml core/Cargo.lock`)
    console.log(`  git commit -m "${commitMsg}"`)
    console.log(`  git push origin ${branchName}`)
    if (!skipPr) {
      console.log(`  gh pr create --title "${commitMsg}" --base main --head ${branchName} --label chore --body ...`)
    }
    console.log(`\nDry run completed for v${targetVersion}!`)
    return
  }

  console.log('\n5. Staging modified release files and committing...')
  run('git add package.json package-lock.json neutralino.config.json core/engine/Cargo.toml core/updater/Cargo.toml core/Cargo.lock')
  run(`git commit -m "${commitMsg}"`)

  console.log('\n6. Pushing branch to origin...')
  run(`git push origin ${branchName}`)

  if (skipPr) {
    console.log('\nSkipping GitHub Pull Request creation (--no-pr flag set).')
    console.log(`\nRelease v${targetVersion} prepared and pushed successfully!`)
    return
  }

  console.log('\n7. Creating GitHub Pull Request...')
  try {
    const prCommand = `gh pr create --title "${commitMsg}" --base main --head ${branchName} --label chore --body "${prBody.replace(/"/g, '\\"')}"`
    run(prCommand)
  } catch {
    console.log('Retrying PR creation with fallback without labels...')
    try {
      run(`gh pr create --title "${commitMsg}" --base main --head ${branchName} --body "${prBody.replace(/"/g, '\\"')}"`)
    } catch {
      console.warn('\nWarning: Failed to create PR automatically via gh cli. You can create it manually on GitHub.')
    }
  }

  console.log(`\nRelease v${targetVersion} prepared and PR opened successfully!`)
}

main()
