import { readFileSync, writeFileSync } from 'fs'
import { execSync, execFileSync } from 'child_process'
import {
  parseSemver,
  parseReleaseArgs,
  resolveBaseBranch,
  buildPrBranchName,
  buildPrBody,
  updateCargoManifestVersion
} from './lib/releaseHelpers.mjs'

function run(command, options = {}) {
  console.log(`> ${command}`)
  return execSync(command, { stdio: 'inherit', ...options })
}

function runFile(file, args, options = {}) {
  console.log(`> ${file} ${args.join(' ')}`)
  return execFileSync(file, args, { stdio: 'inherit', ...options })
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
  updateFile(filePath, (content) => updateCargoManifestVersion(content, newVersion))
}

function printHelp() {
  console.log(`
Usage: node scripts/release.js [version] [options]

Arguments:
  [version]           Target semver version (e.g. 0.18.3 or v0.18.3). If omitted, bumps current patch.

Options:
  --base <branch>     Target base branch for PR creation (default: current checked-out branch).
  --direct            Commit and push directly on the current release/* or hotfix/* branch (refuses main/master).
  --tag               Create and push git tag v<version> (requires --direct or --no-pr).
  --no-pr             Skip creating a GitHub Pull Request.
  --dry-run           Preview changes and commands without committing, pushing, or creating PR.
  -h, --help          Display this help message.
`)
}

function main() {
  let cli
  try {
    cli = parseReleaseArgs(process.argv.slice(2))
  } catch (error) {
    console.error(`Error: ${error.message}`)
    process.exit(1)
  }

  if (cli.help) {
    printHelp()
    process.exit(0)
  }

  const currentBranch = execSync('git branch --show-current', { encoding: 'utf8' }).trim()
  let baseBranch
  try {
    baseBranch = resolveBaseBranch({
      explicitBase: cli.baseBranch,
      currentBranch,
      direct: cli.direct
    })
  } catch (error) {
    console.error(`Error: ${error.message}`)
    process.exit(1)
  }

  const packageJson = JSON.parse(readFileSync('package.json', 'utf8'))
  const currentVersion = packageJson.version
  let targetVersion = cli.targetVersionArg

  if (!targetVersion) {
    const semver = parseSemver(currentVersion)
    if (!semver) {
      console.error(
        `Current version (${currentVersion}) is not standard semver. Please provide a version: npm run release <version>`
      )
      process.exit(1)
    }
    targetVersion = `${semver.major}.${semver.minor}.${semver.patch + 1}`
    console.log(
      `No version specified. Auto-bumping patch version from ${currentVersion} to ${targetVersion}`
    )
  }

  if (targetVersion.startsWith('v')) {
    targetVersion = targetVersion.slice(1)
  }

  if (!parseSemver(targetVersion)) {
    console.error(`Error: Invalid semver version string: "${targetVersion}"`)
    process.exit(1)
  }

  console.log(
    `\n=== Preparing Orbit Release v${targetVersion}${cli.dryRun ? ' (DRY RUN)' : ''} ===\n`
  )

  const branchName = cli.direct ? baseBranch : buildPrBranchName(targetVersion)
  const commitMsg = `chore: release v${targetVersion}`
  const prBody = buildPrBody(targetVersion, currentVersion, baseBranch)
  const tagName = `v${targetVersion}`

  const statusOutput = execSync('git status --porcelain', { encoding: 'utf8' }).trim()
  if (statusOutput.length > 0) {
    console.error(
      'Error: Git working tree has uncommitted changes. Please stash or commit them before releasing.'
    )
    process.exit(1)
  }

  if (cli.direct) {
    console.log(`1. Using current branch ${branchName} (--direct mode)...`)
  } else {
    console.log(`1. Checking out PR prep branch ${branchName} (base: ${baseBranch})...`)
    const checkoutArgs =
      cli.baseBranch && baseBranch !== currentBranch
        ? ['checkout', '-b', branchName, baseBranch]
        : ['checkout', '-b', branchName]
    try {
      runFile('git', checkoutArgs)
    } catch {
      console.log(`Branch ${branchName} might already exist, checking out...`)
      runFile('git', ['checkout', branchName])
    }
  }

  console.log(`\n2. Updating version numbers to ${targetVersion}...`)

  updateFile('package.json', (content) => {
    const pkg = JSON.parse(content)
    pkg.version = targetVersion
    return JSON.stringify(pkg, null, 2) + '\n'
  })

  updateFile('src-tauri/tauri.conf.json', (content) => {
    const config = JSON.parse(content)
    config.version = targetVersion
    return JSON.stringify(config, null, 2) + '\n'
  })

  updateCargoVersion('Cargo.toml', targetVersion)

  console.log('\n3. Updating lock files...')
  run('npm install')
  run('cargo check')

  console.log('\n4. Running typecheck...')
  run('npm run type-check')

  if (cli.dryRun) {
    console.log('\n[DRY RUN] Skipping git commit, push, tag, and PR creation.')
    console.log('[DRY RUN] Would execute:')
    console.log(
      '  git add package.json package-lock.json src-tauri/tauri.conf.json Cargo.toml Cargo.lock'
    )
    console.log(`  git commit -m "${commitMsg}"`)
    console.log(`  git push origin ${branchName}`)
    if (cli.tag) {
      console.log(`  git tag ${tagName}`)
      console.log(`  git push origin ${tagName}`)
    }
    if (!cli.direct && !cli.skipPr) {
      console.log(
        `  gh pr create --title "${commitMsg}" --base ${baseBranch} --head ${branchName} --label chore --body ...`
      )
    }
    console.log(`\nDry run completed for v${targetVersion}!`)
    return
  }

  console.log('\n5. Staging modified release files and committing...')
  run('git add package.json package-lock.json src-tauri/tauri.conf.json Cargo.toml Cargo.lock')
  runFile('git', ['commit', '-m', commitMsg])

  console.log(`\n6. Pushing branch ${branchName} to origin...`)
  runFile('git', ['push', 'origin', branchName])

  if (cli.tag) {
    console.log(`\n7. Creating and pushing tag ${tagName}...`)
    runFile('git', ['tag', tagName])
    runFile('git', ['push', 'origin', tagName])
  }

  if (cli.direct || cli.skipPr) {
    if (cli.skipPr) {
      console.log('\nSkipping GitHub Pull Request creation (--no-pr flag set).')
    }
    console.log(`\nRelease v${targetVersion} prepared and pushed successfully!`)
    return
  }

  console.log(`\n7. Creating GitHub Pull Request (base: ${baseBranch})...`)
  const prArgs = [
    'pr',
    'create',
    '--title',
    commitMsg,
    '--base',
    baseBranch,
    '--head',
    branchName,
    '--label',
    'chore',
    '--body',
    prBody
  ]
  try {
    runFile('gh', prArgs)
  } catch {
    console.log('Retrying PR creation with fallback without labels...')
    try {
      runFile('gh', [
        'pr',
        'create',
        '--title',
        commitMsg,
        '--base',
        baseBranch,
        '--head',
        branchName,
        '--body',
        prBody
      ])
    } catch {
      console.warn(
        '\nWarning: Failed to create PR automatically via gh cli. You can create it manually on GitHub.'
      )
    }
  }

  console.log(`\nRelease v${targetVersion} prepared and PR opened successfully!`)
}

main()
